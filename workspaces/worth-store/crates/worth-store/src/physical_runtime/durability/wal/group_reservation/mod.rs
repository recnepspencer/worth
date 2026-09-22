mod member_planning;

use worth_proof::NonEmpty;
use worth_store_wal::{
    LogSequenceNumber, WalAppendFrontier, WalSegmentArtifactIdentity, WalSegmentId,
};

use crate::physical_runtime::{
    AdmittedPhysicalDurabilityGroupMember, PhysicalDurabilityGroupMemberBinding,
    PhysicalWalFrameWriteDisposition, WalBarrierMember, WalRangeReservedPhysicalMutation,
};

use self::member_planning::{plan_group, release_reserved_members, wal_artifact};
use super::{PhysicalWalReservationDenial, PhysicalWalRuntimeOwner};

pub(in crate::physical_runtime) struct ReservedPhysicalWalGroupMembers(
    NonEmpty<WalBarrierMember<WalRangeReservedPhysicalMutation>>,
);

impl PhysicalWalRuntimeOwner {
    pub(super) fn reserve_group(
        &self,
        members: NonEmpty<AdmittedPhysicalDurabilityGroupMember>,
    ) -> Result<
        ReservedPhysicalWalGroupMembers,
        (
            NonEmpty<AdmittedPhysicalDurabilityGroupMember>,
            PhysicalWalReservationDenial,
        ),
    > {
        let admitted = self.admit_group_preparations(members)?;
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.sealed {
            return Err((
                restore_admitted(admitted),
                PhysicalWalReservationDenial::InspectionRequired,
            ));
        }
        if state.in_flight {
            return Err((
                restore_admitted(admitted),
                PhysicalWalReservationDenial::AppendInFlight,
            ));
        }
        let byte_limit = state.policy.segment_byte_limit().get().get();
        let first_disposition = if state.segment_count == 0 {
            PhysicalWalFrameWriteDisposition::CreateSegment
        } else {
            PhysicalWalFrameWriteDisposition::AppendExistingSegment
        };
        let group_lsn_start = state
            .frontier
            .last_lsn_end()
            .unwrap_or(LogSequenceNumber::new(LogSequenceNumber::GENESIS.get() + 1));
        let current = plan_group(
            admitted,
            state.frontier,
            group_lsn_start,
            state.active_artifact.clone(),
            first_disposition,
        )?;
        if current.resulting_frontier().valid_prefix_bytes() <= byte_limit {
            state.in_flight = true;
            return Ok(current);
        }
        let requested_on_current = current.resulting_frontier().valid_prefix_bytes();
        let admitted = current.release_after_no_effect();
        if state.segment_count == 0 {
            return Err((
                admitted,
                PhysicalWalReservationDenial::GroupFrameBytesExceedSegmentLimit {
                    admitted: byte_limit,
                    requested: requested_on_current,
                },
            ));
        }
        let inventory_limit = state.policy.segment_inventory_limit().get().get();
        if state.segment_count >= inventory_limit {
            return Err((
                admitted,
                PhysicalWalReservationDenial::SegmentInventoryLimitReached {
                    admitted: inventory_limit,
                    retained: state.segment_count,
                },
            ));
        }
        let Some(next_segment) = state
            .frontier
            .segment()
            .get()
            .checked_add(1)
            .and_then(|value| WalSegmentId::new(value).ok())
        else {
            return Err((
                admitted,
                PhysicalWalReservationDenial::SegmentIdentityExhausted,
            ));
        };
        let next_frontier = WalAppendFrontier::empty(next_segment, state.frontier.generation());
        let artifact = wal_artifact(WalSegmentArtifactIdentity::new(
            next_segment,
            state.frontier.generation(),
        ));
        let admitted = self.admit_group_preparations(admitted)?;
        let rotated = plan_group(
            admitted,
            next_frontier,
            group_lsn_start,
            artifact,
            PhysicalWalFrameWriteDisposition::CreateSegment,
        )?;
        let requested = rotated.resulting_frontier().valid_prefix_bytes();
        if requested > byte_limit {
            return Err((
                rotated.release_after_no_effect(),
                PhysicalWalReservationDenial::GroupFrameBytesExceedSegmentLimit {
                    admitted: byte_limit,
                    requested,
                },
            ));
        }
        state.in_flight = true;
        Ok(rotated)
    }

    fn admit_group_preparations(
        &self,
        members: NonEmpty<AdmittedPhysicalDurabilityGroupMember>,
    ) -> Result<
        Vec<(
            super::preparation_admission::AdmittedWalPreparedMutation,
            PhysicalDurabilityGroupMemberBinding,
        )>,
        (
            NonEmpty<AdmittedPhysicalDurabilityGroupMember>,
            PhysicalWalReservationDenial,
        ),
    > {
        let mut pending = members.into_vec().into_iter();
        let mut admitted = Vec::new();
        while let Some(member) = pending.next() {
            let (prepared, binding) = member.into_parts();
            match self.admit_preparation(prepared) {
                Ok(prepared) => admitted.push((prepared, binding)),
                Err((prepared, cause)) => {
                    let mut restored = restore_admitted_vec(admitted);
                    restored.push(AdmittedPhysicalDurabilityGroupMember::from_parts(
                        prepared, binding,
                    ));
                    restored.extend(pending);
                    return Err((nonempty(restored), cause));
                }
            }
        }
        Ok(admitted)
    }
}

impl ReservedPhysicalWalGroupMembers {
    pub(super) fn from_members(
        members: NonEmpty<WalBarrierMember<WalRangeReservedPhysicalMutation>>,
    ) -> Self {
        Self(members)
    }

    pub(super) fn into_members(
        self,
    ) -> NonEmpty<WalBarrierMember<WalRangeReservedPhysicalMutation>> {
        self.0
    }

    pub(super) fn retained_publication_bytes(&self) -> u64 {
        self.0.as_slice().iter().fold(0, |total, member| {
            let mutation = member.mutation();
            let wal = mutation.encoded_frame().len() as u64;
            let metadata = mutation
                .root_projection()
                .retained_publication_metadata_bytes();
            total.saturating_add(wal).saturating_add(metadata)
        })
    }

    fn resulting_frontier(&self) -> WalAppendFrontier {
        self.0
            .as_slice()
            .last()
            .expect("reserved group membership is nonempty")
            .mutation()
            .resulting_frontier()
    }

    pub(super) fn release_after_no_effect(self) -> NonEmpty<AdmittedPhysicalDurabilityGroupMember> {
        release_reserved_members(self.0)
    }
}

fn restore_admitted(
    admitted: Vec<(
        super::preparation_admission::AdmittedWalPreparedMutation,
        PhysicalDurabilityGroupMemberBinding,
    )>,
) -> NonEmpty<AdmittedPhysicalDurabilityGroupMember> {
    nonempty(restore_admitted_vec(admitted))
}

fn restore_admitted_vec(
    admitted: Vec<(
        super::preparation_admission::AdmittedWalPreparedMutation,
        PhysicalDurabilityGroupMemberBinding,
    )>,
) -> Vec<AdmittedPhysicalDurabilityGroupMember> {
    admitted
        .into_iter()
        .map(|(prepared, binding)| {
            AdmittedPhysicalDurabilityGroupMember::from_parts(prepared.into_prepared(), binding)
        })
        .collect()
}

fn nonempty<T>(members: Vec<T>) -> NonEmpty<T> {
    NonEmpty::try_from_vec(members).unwrap_or_else(|_| {
        unreachable!("group admission and reservation preserve nonempty membership")
    })
}
