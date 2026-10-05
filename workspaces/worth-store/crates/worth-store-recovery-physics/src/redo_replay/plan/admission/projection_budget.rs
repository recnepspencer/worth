//! Debits each admitted C.9 projection from the bounded decode allowance.
//!
//! A member is decoded against its own ceiling and then debited, so an
//! exhausted allowance is reported as that limit with the count that crossed
//! it and never as a malformed projection.

use worth_store_physical_format::{
    PersistedPhysicalRecoveryPayload, PersistedPhysicalRecoveryProjection,
    PhysicalRecoveryProjectionDecodeLimits,
};

use super::super::PhysicalRedoPlanningDenial;
use crate::PhysicalRedoProjectionLimit;

/// What every admitted member's projection may still hold, of what was
/// admitted for all of them.
pub(super) struct ProjectionBudget {
    admitted: PhysicalRecoveryProjectionDecodeLimits,
    remaining: PhysicalRecoveryProjectionDecodeLimits,
}

impl ProjectionBudget {
    pub(super) const fn new(admitted: PhysicalRecoveryProjectionDecodeLimits) -> Self {
        Self {
            admitted,
            remaining: admitted,
        }
    }

    /// One member's projection holds no more entries of any kind than its
    /// canonical redo has bytes.
    pub(super) const fn member_ceiling(
        canonical_redo: &[u8],
    ) -> PhysicalRecoveryProjectionDecodeLimits {
        let bytes = canonical_redo.len() as u64;
        PhysicalRecoveryProjectionDecodeLimits {
            frames: bytes,
            record_identities: bytes,
            placements: bytes,
            segment_updates: bytes,
            manifests: bytes,
            total_entries: bytes.saturating_mul(3),
            inline_allocations: bytes,
        }
    }

    pub(super) fn consume(
        &mut self,
        projection: &PersistedPhysicalRecoveryProjection,
    ) -> Result<(), PhysicalRedoPlanningDenial> {
        use PhysicalRedoProjectionLimit as Limit;
        let frames = match projection.payload() {
            PersistedPhysicalRecoveryPayload::Frames(frames) => frames.len() as u64,
            PersistedPhysicalRecoveryPayload::SourceCopy(_) => 0,
        };
        let placements = projection.placements().len() as u64;
        let segment_updates = projection.segment_updates().len() as u64;
        let manifests = projection.manifests().len() as u64;
        let claimed = projection
            .operation()
            .release_head_tree_claim()
            .map(|claim| {
                claim
                    .entry_count()
                    .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)
            })
            .transpose()?
            .unwrap_or(0);
        let total_entries = [segment_updates, manifests, claimed]
            .into_iter()
            .try_fold(placements, u64::checked_add)
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
        let (admitted, remaining) = (self.admitted, &mut self.remaining);
        debit(
            &mut remaining.frames,
            admitted.frames,
            frames,
            Limit::Frames,
        )?;
        debit(
            &mut remaining.record_identities,
            admitted.record_identities,
            projection.record_identities().len() as u64,
            Limit::RecordIdentities,
        )?;
        debit(
            &mut remaining.placements,
            admitted.placements,
            placements,
            Limit::Placements,
        )?;
        debit(
            &mut remaining.segment_updates,
            admitted.segment_updates,
            segment_updates,
            Limit::SegmentUpdates,
        )?;
        debit(
            &mut remaining.manifests,
            admitted.manifests,
            manifests,
            Limit::Manifests,
        )?;
        debit(
            &mut remaining.total_entries,
            admitted.total_entries,
            total_entries,
            Limit::TotalEntries,
        )?;
        debit(
            &mut remaining.inline_allocations,
            admitted.inline_allocations,
            projection.root_state().inline_allocations().len() as u64,
            Limit::InlineAllocations,
        )
    }
}

/// Takes `count` from what remains, or reports the limit with everything
/// counted against it so far.
fn debit(
    remaining: &mut u64,
    admitted: u64,
    count: u64,
    limit: PhysicalRedoProjectionLimit,
) -> Result<(), PhysicalRedoPlanningDenial> {
    match remaining.checked_sub(count) {
        Some(left) => {
            *remaining = left;
            Ok(())
        }
        None => Err(PhysicalRedoPlanningDenial::ProjectionLimit {
            limit,
            observed: (admitted - *remaining).saturating_add(count),
            admitted,
        }),
    }
}
