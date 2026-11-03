#[path = "completion/blob_expiry.rs"]
mod blob_expiry;
#[path = "completion/blob_reclaim.rs"]
pub(super) mod blob_reclaim;
#[path = "completion/execution_basis.rs"]
mod execution_basis;
#[path = "completion/historical_publication.rs"]
pub(super) mod historical_publication;
#[path = "completion/plan_cost.rs"]
mod plan_cost;
#[path = "completion/planned_recovery.rs"]
mod planned_recovery;
#[path = "completion/planned_storage.rs"]
mod planned_storage;
#[path = "completion/publication_effects.rs"]
mod publication_effects;
#[path = "completion/rewrite_materialization.rs"]
mod rewrite_materialization;
#[path = "completion/source_copy.rs"]
mod source_copy;
#[path = "completion/verified_inputs.rs"]
mod verified_inputs;
pub(super) use verified_inputs::verify_inputs;

use crate::progression::PlannedPhysicalRecovery;

use super::context::PlanningContext;
use super::resolved_basis::ResolvedPlanningBasis;

pub(super) fn complete(
    mut context: PlanningContext,
    mut basis: ResolvedPlanningBasis,
) -> Result<PlannedPhysicalRecovery, crate::entry::PhysicalRecoveryOutcome> {
    let execution;
    (context, execution) = execution_basis::derive(context, basis.state_mut())?;
    let (backing, backing_peak);
    (context, backing, backing_peak) =
        planned_storage::prepare(context, basis.state(), &execution)?;
    let (plan_cost, planning_counters);
    (context, plan_cost, planning_counters) =
        plan_cost::admit(context, basis.state(), &execution, backing_peak)?;
    context = publication_effects::admit(context, planning_counters, &execution.publication)?;
    planned_recovery::construct(
        context,
        basis.into_state(),
        execution,
        backing,
        plan_cost,
        planning_counters,
    )
}
