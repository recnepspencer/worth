//! Preserve the responsible ordered-custody boundary in planning outcomes.

use worth_store_recovery_physics::{PendingWalReleaseCustodyDenial, PhysicsBound};

use super::super::{PlanningContext, ResolvedPlanningBasis};
use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitFailure, PhysicalRecoveryOrderedReleaseDenial,
    PhysicalRecoveryOutcome, PhysicalRecoveryPlanningDenial,
};
use crate::orchestration::recovery_budget::RecoveryAllowance;

pub(in crate::orchestration::planning::completion::blob_reclaim::selected_release_gate::pending_wal) fn block(
    context: PlanningContext,
    basis: &ResolvedPlanningBasis,
    cause: PhysicalRecoveryOrderedReleaseDenial,
    limit: Option<PhysicalRecoveryLimitFailure>,
) -> PhysicalRecoveryOutcome {
    context.block_with_planning_attempt_denial(
        PhysicalRecoveryBlockKind::SelectedCustody,
        basis.planning_counters(),
        "ordered-release-custody",
        limit,
        PhysicalRecoveryPlanningDenial::OrderedRelease(cause),
    )
}

/// A batch admission refused, and the staging limit it ran past if that is
/// why. Physics was handed what `staging` had left beside the roster and
/// the batches before this one, so recovery needed that much more in all.
pub(in crate::orchestration::planning::completion::blob_reclaim::selected_release_gate::pending_wal) fn batch(
    operation: [u8; 32],
    edge_index: usize,
    cause: PendingWalReleaseCustodyDenial,
    staging: RecoveryAllowance,
) -> (
    PhysicalRecoveryOrderedReleaseDenial,
    Option<PhysicalRecoveryLimitFailure>,
) {
    use PendingWalReleaseCustodyDenial as Refused;
    let limit = match cause {
        Refused::Limit(past) => match past.dimension() {
            PhysicsBound::RetainedBytes => staging
                .beside(past.observed(), past.admitted())
                .map(Into::into),
            // Batch admission is handed retained bytes alone: it cannot run
            // past any other bound.
            PhysicsBound::ResidentBytes
            | PhysicsBound::ManifestEntries
            | PhysicsBound::DistinctPagesAndExtents => None,
        },
        Refused::CheckpointMarker
        | Refused::SourceBinding
        | Refused::ControlBinding
        | Refused::DurableWalFate
        | Refused::PublishedRoot
        | Refused::RetainedSizeOverflow => None,
    };
    let denial = PhysicalRecoveryOrderedReleaseDenial::Batch {
        operation,
        edge_index,
        cause,
    };
    (denial, limit)
}

#[cfg(test)]
mod tests {
    use worth_store_recovery_physics::{
        test_support::physics_limit_for_test, PendingWalReleaseCustodyDenial as Refused,
        PhysicsBound,
    };

    use super::batch;
    use crate::entry::PhysicalRecoveryLimitDimension::StagingBytes;
    use crate::orchestration::recovery_budget::{allowance_for_test, recovery_limit_for_test};

    #[test]
    fn a_batch_past_its_retained_bytes_is_the_staging_limit_beside_what_was_held() {
        // 64 staging bytes; the roster and earlier batches held 40, so the
        // batch was handed 24 and needed 30: recovery needed 70.
        let staging = allowance_for_test(StagingBytes, 64);
        let past = physics_limit_for_test(PhysicsBound::RetainedBytes, 30, 24);
        let (_, limit) = batch([7; 32], 1, Refused::Limit(past), staging);
        assert_eq!(
            limit,
            Some(recovery_limit_for_test(StagingBytes, 70, 64).into())
        );
        let (_, limit) = batch([7; 32], 1, Refused::ControlBinding, staging);
        assert_eq!(limit, None);
    }
}
