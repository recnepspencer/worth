//! Preserve the responsible ordered-custody boundary in planning outcomes.

use super::super::{PlanningContext, ResolvedPlanningBasis};
use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitFailure, PhysicalRecoveryOrderedReleaseDenial,
    PhysicalRecoveryOutcome, PhysicalRecoveryPlanningDenial,
};

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
