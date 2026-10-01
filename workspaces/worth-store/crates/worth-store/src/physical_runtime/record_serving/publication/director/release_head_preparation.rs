//! Fixes the one-key head-tree effect from the fenced post-reservation root,
//! before C9 encodes the descriptor's WAL member.

use worth_store_physical_format::{
    DurablePhysicalRootManifest, PersistedReleaseCustodyHeadEffectV1, ReleaseCustodyHeadDenial,
    ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadTransitionLimitsV1,
    ReleaseCustodyHeadTransitionV1,
};

use super::RecordPublicationDirector;
use crate::physical_runtime::record_serving::{
    access::release_custody_head::read_release_head_path, publication::PreparedReleaseHeadBasis,
    PreparedPhysicalRootProjection, RecordAppendDenial, RecordAppendError,
};

const MAX_HEAD_PATH_NODES: u16 = 16;
const MAX_HEAD_NEW_BLOCKS: u16 = 36;

pub(super) fn release_head_reservation_bytes(
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
) -> Option<u64> {
    u64::from(format.page_size().bytes())
        .checked_mul(u64::from(MAX_HEAD_PATH_NODES + MAX_HEAD_NEW_BLOCKS))
}

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn released_head_capacity_charge(
        &self,
    ) -> Option<crate::physical_runtime::durability::ReleaseHeadCapacityCharge> {
        let frame_bytes = u64::from(self.format.declaration().page_size().bytes());
        let path_and_nodes = release_head_reservation_bytes(self.format.declaration())?;
        Some(
            crate::physical_runtime::durability::ReleaseHeadCapacityCharge::new(
                frame_bytes,
                path_and_nodes,
                path_and_nodes,
                0,
                worth_store_physical_format::ReleaseCustodyHeadEntryV1::ENCODED_BYTES as u64,
            ),
        )
    }

    pub(super) fn plan_released_head_for_wal(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        current_root: &DurablePhysicalRootManifest,
        basis: PreparedReleaseHeadBasis,
        prepared_root: &mut PreparedPhysicalRootProjection,
    ) -> Result<(), RecordAppendError> {
        let [descriptor_record] = prepared_root.records.as_slice() else {
            return Err(damaged());
        };
        if prepared_root.source_root != *current_root
            || basis.descriptor().base().source_root_generation() != current_root.generation()
        {
            return Err(damaged());
        }
        let next = basis
            .next_entry(*descriptor_record)
            .map_err(|_| damaged())?;
        let limits = ReleaseCustodyHeadTransitionLimitsV1::new(
            MAX_HEAD_PATH_NODES,
            MAX_HEAD_NEW_BLOCKS,
            release_head_reservation_bytes(self.format.declaration()).ok_or_else(damaged)?,
        )
        .ok_or_else(damaged)?;
        let source_path = read_release_head_path(
            allocation,
            self.residency.clone(),
            self.format,
            self.access,
            current_root,
            basis.key(),
            limits,
        )?;
        let successor_generation = current_root
            .generation()
            .checked_add(1)
            .ok_or_else(damaged)?;
        let transition = ReleaseCustodyHeadTransitionV1::plan(
            current_root.release_custody_head_root(),
            current_root.next_release_custody_head_block(),
            &source_path,
            ReleaseCustodyHeadMutationV1::Upsert {
                expected_prior: basis.prior(),
                next,
            },
            successor_generation,
            current_root.tree_identity(),
            self.format.declaration(),
            limits,
        )
        .map_err(head_denial)?;
        let effect = PersistedReleaseCustodyHeadEffectV1::new_upsert(
            current_root.tree_identity(),
            current_root.generation(),
            basis.source_basis(),
            source_path,
            transition,
            self.format.declaration(),
            limits,
        )
        .map_err(|_| damaged())?;
        prepared_root
            .set_release_head_effect(effect)
            .ok_or(RecordAppendError::Denied(
                RecordAppendDenial::PhysicalPressure,
            ))?;
        Ok(())
    }
}

fn head_denial(denial: ReleaseCustodyHeadDenial) -> RecordAppendError {
    match denial {
        ReleaseCustodyHeadDenial::Budget | ReleaseCustodyHeadDenial::Capacity => {
            RecordAppendError::Denied(RecordAppendDenial::PhysicalPressure)
        }
        _ => damaged(),
    }
}

fn damaged() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
}
