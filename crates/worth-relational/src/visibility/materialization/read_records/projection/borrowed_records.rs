use super::VisibilityProjectionView;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionAccess;
use crate::visibility::snapshot_states::SnapshotStateBasis;
use worth_foundational::facade::AuthoritativeRecordAspectState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalBorrowedRecordReadDenial {
    ExactBasisRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationalEntityMetadata {
    pub entity_id: EntityId,
    pub kind_id: KindId,
    pub lifecycle: RecordLifecycleState,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationalRelationMetadata {
    pub relation_id: RelationId,
    pub kind_id: KindId,
    pub lifecycle: RecordLifecycleState,
    pub source: EntityId,
    pub target: EntityId,
}

impl VisibilityProjectionView<'_> {
    /// One owner-issued Work bound for the exact borrowed entity probe below.
    /// The selected schema count is read from the pinned root without a walk.
    pub fn exact_entity_state_read_work_bound(&self) -> Option<u64> {
        let SnapshotStateBasis::Exact(basis) = &self.basis else {
            return None;
        };
        Some(
            38_u64.saturating_add(kind_lookup_work(
                basis
                    .root()
                    .schema_authority()
                    .registry()
                    .entity_kinds
                    .len(),
            )),
        )
    }

    /// One owner-issued Work bound for the exact borrowed relation probe.
    pub fn exact_relation_metadata_read_work_bound(&self) -> Option<u64> {
        let SnapshotStateBasis::Exact(basis) = &self.basis else {
            return None;
        };
        Some(
            39_u64.saturating_add(kind_lookup_work(
                basis
                    .root()
                    .schema_authority()
                    .registry()
                    .relation_kinds
                    .len(),
            )),
        )
    }

    /// Borrow the exact selected root's state without resolving owned kind names
    /// or materializing a read record. A pinned old exact root remains eligible.
    pub fn with_exact_entity_state<T>(
        &self,
        entity: EntityId,
        project: impl FnOnce(RelationalEntityMetadata, Option<&AuthoritativeRecordAspectState>) -> T,
    ) -> Result<Option<T>, RelationalBorrowedRecordReadDenial> {
        let SnapshotStateBasis::Exact(basis) = &self.basis else {
            return Err(RelationalBorrowedRecordReadDenial::ExactBasisRequired);
        };
        let root = basis.root();
        let Some(partition) = root.get_partition(entity.partition_id) else {
            return Ok(None);
        };
        let Some(slot) = partition.entity_arena.get_slot(entity.slot_index()) else {
            return Ok(None);
        };
        if !slot.is_live()
            || (!entity.generation.is_zero() && slot.generation() != entity.generation_value())
        {
            return Ok(None);
        }
        let Some(kind_id) = slot.kind_id() else {
            return Ok(None);
        };
        if !root
            .schema_authority()
            .registry()
            .entity_kinds
            .contains_key(&kind_id)
        {
            return Ok(None);
        }
        let metadata = RelationalEntityMetadata {
            entity_id: EntityId::new(
                entity.partition_id,
                entity.local_slot_value(),
                slot.generation(),
            ),
            kind_id,
            lifecycle: slot.lifecycle(),
        };
        Ok(Some(project(
            metadata,
            slot.extra().authoritative_aspect_state.as_ref(),
        )))
    }

    /// Exact metadata only; names and aspect values stay in the pinned root.
    pub fn exact_relation_metadata(
        &self,
        relation: RelationId,
    ) -> Result<Option<RelationalRelationMetadata>, RelationalBorrowedRecordReadDenial> {
        let SnapshotStateBasis::Exact(basis) = &self.basis else {
            return Err(RelationalBorrowedRecordReadDenial::ExactBasisRequired);
        };
        let root = basis.root();
        let Some(partition) = root.get_partition(relation.partition_id) else {
            return Ok(None);
        };
        let Some(slot) = partition.relation_arena.get_slot(relation.slot_index()) else {
            return Ok(None);
        };
        if !slot.is_live()
            || (!relation.generation.is_zero() && slot.generation() != relation.generation_value())
        {
            return Ok(None);
        }
        let Some(kind_id) = slot.kind_id() else {
            return Ok(None);
        };
        if !root
            .schema_authority()
            .registry()
            .relation_kinds
            .contains_key(&kind_id)
        {
            return Ok(None);
        }
        let Some(endpoints) = &slot.extra().endpoints else {
            return Ok(None);
        };
        Ok(Some(RelationalRelationMetadata {
            relation_id: RelationId::new(
                relation.partition_id,
                relation.local_slot_value(),
                slot.generation(),
            ),
            kind_id,
            lifecycle: slot.lifecycle(),
            source: endpoints.source,
            target: endpoints.target,
        }))
    }
}

fn kind_lookup_work(entries: usize) -> u64 {
    let mut levels = 1_u64;
    let mut minimum = 1_usize;
    while entries > minimum {
        minimum = minimum.saturating_mul(6).saturating_add(5);
        levels = levels.saturating_add(1);
        if minimum == usize::MAX {
            break;
        }
    }
    levels.saturating_mul(11)
}
