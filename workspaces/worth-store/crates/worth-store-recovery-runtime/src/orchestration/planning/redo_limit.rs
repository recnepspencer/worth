//! The recovery limit a refused redo plan ran out of, and its value. A
//! denial that is not an exhausted limit has none.

use worth_store_recovery_physics::{PhysicalRedoPlanningDenial, PhysicalRedoProjectionLimit};

use crate::entry::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure,
};
use crate::orchestration::recovery_budget::RecoveryAllowance;

/// Placements, segment updates and inline allocations are manifest entries,
/// admitted on top of the `manifest_entries` already observed; the blocks a
/// member writes are bounded by its own bytes and are not entries. Physics was
/// handed the whole memory, target and distinct-target limits.
pub(super) fn redo_limit(
    limits: &PhysicalRecoveryLimitDeclaration,
    manifest_entries: u64,
    denial: PhysicalRedoPlanningDenial,
) -> Option<PhysicalRecoveryLimitFailure> {
    use PhysicalRecoveryLimitDimension as Dimension;
    use PhysicalRedoPlanningDenial as Denial;
    use PhysicalRedoProjectionLimit as Projection;
    let refused = |dimension, observed, admitted| {
        RecoveryAllowance::declared(limits, dimension)
            .beside(observed, admitted)
            .map(Into::into)
    };
    match denial {
        Denial::RecoveryMemoryLimit { observed, admitted } => {
            refused(Dimension::RecoveryMemoryBytes, observed, admitted)
        }
        Denial::TargetLimit { observed, admitted } => {
            refused(Dimension::RedoTargets, observed, admitted)
        }
        Denial::DistinctTargetLimit { observed, admitted } => {
            refused(Dimension::DistinctPagesAndExtents, observed, admitted)
        }
        Denial::ProjectionLimit {
            limit, observed, ..
        } => match limit {
            Projection::Frames | Projection::RecordIdentities => {
                RecoveryAllowance::declared(limits, Dimension::RedoTargets).past(observed)
            }
            Projection::Placements
            | Projection::SegmentUpdates
            | Projection::Manifests
            | Projection::TotalEntries
            | Projection::InlineAllocations => {
                RecoveryAllowance::declared(limits, Dimension::ManifestEntries)
                    .past(manifest_entries.checked_add(observed)?)
            }
        },
        Denial::MalformedMember
        | Denial::WrongDomain
        | Denial::InvalidRecordOrder
        | Denial::NonCanonicalTargetOrder
        | Denial::LsnRangeMismatch
        | Denial::InvalidTarget
        | Denial::InvalidRecoveryProjection
        | Denial::UnsupportedRecoveryProjectionVersion(_)
        | Denial::MissingPageObservation
        | Denial::GenerationMismatch
        | Denial::PageDigestMismatch
        | Denial::ProvenNoEffectHasWalAttempt
        | Denial::CounterOverflow
        | Denial::TerminalHeadRetirementUnsupported => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PhysicalRecoveryLimitDimension as Dimension;
    use PhysicalRedoPlanningDenial as Denial;

    fn limits() -> PhysicalRecoveryLimitDeclaration {
        PhysicalRecoveryLimitDeclaration {
            selector_candidates: 1,
            checkpoint_candidates: 1,
            manifest_bytes: 1,
            manifest_entries: 40,
            wal_segments: 1,
            wal_frames: 1,
            wal_bytes: 1,
            redo_targets: 8,
            redo_bytes: 1,
            distinct_pages_and_extents: 8,
            operation_bindings: 1,
            staging_bytes: 1,
            recovery_memory_bytes: 8,
            dirty_frames: 1,
            concurrent_commands: 1,
            publication_effects: 1,
            cleanup_candidates: 1,
            cleanup_bytes: 1,
            observation_bytes: 1,
        }
    }

    fn named(denial: Denial) -> Option<(Dimension, u64, u64)> {
        redo_limit(&limits(), 30, denial)
            .map(|limit| (limit.dimension(), limit.observed(), limit.admitted()))
    }

    #[test]
    fn an_exhausted_redo_limit_is_named_with_what_was_needed() {
        // What was needed is carried as it is, however far past the limit.
        let (observed, admitted) = (12, 8);
        assert_eq!(
            named(Denial::TargetLimit { observed, admitted }),
            Some((Dimension::RedoTargets, 12, 8))
        );
        assert_eq!(
            named(Denial::DistinctTargetLimit { observed, admitted }),
            Some((Dimension::DistinctPagesAndExtents, 12, 8))
        );
        assert_eq!(
            named(Denial::RecoveryMemoryLimit { observed, admitted }),
            Some((Dimension::RecoveryMemoryBytes, 12, 8))
        );
        let projection = |limit| Denial::ProjectionLimit {
            limit,
            observed: 13,
            admitted: 10,
        };
        assert_eq!(
            named(projection(PhysicalRedoProjectionLimit::Frames)),
            Some((Dimension::RedoTargets, 13, 8))
        );
        // Projected entries are admitted on top of the thirty already seen.
        assert_eq!(
            named(projection(PhysicalRedoProjectionLimit::Placements)),
            Some((Dimension::ManifestEntries, 43, 40))
        );
    }

    #[test]
    fn a_member_that_is_not_what_it_claims_exhausted_no_limit() {
        assert_eq!(named(Denial::MalformedMember), None);
        assert_eq!(named(Denial::PageDigestMismatch), None);
        assert_eq!(named(Denial::CounterOverflow), None);
    }
}
