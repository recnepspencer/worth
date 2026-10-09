//! Rejoin completed V3 edges into pending or completed ordered custody.

use worth_store_recovery_physics::VerifiedPendingWalReleaseCustody;

use super::{PlanningContext, ResolvedPlanningBasis};
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;

#[path = "ordered.rs"]
mod ordered;

#[path = "historical/completed.rs"]
mod completed;
pub(in crate::orchestration::planning::completion::blob_reclaim::selected_release_gate) use completed::admit_completed_history;

/// Every completed V3 edge is joined through the ordered checkpoint-to-selected
/// walk, which alone replays its release-custody head effect.
pub(super) fn attach(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    claim: &mut VerifiedPendingWalReleaseCustody,
    resident: &mut ResidentAllowance,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    if basis.observed_pages.ordered_releases.is_some() {
        return ordered::attach(context, basis, claim, resident);
    }
    Ok(context)
}
