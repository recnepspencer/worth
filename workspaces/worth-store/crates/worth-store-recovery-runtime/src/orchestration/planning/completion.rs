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
#[path = "completion/publication_effects.rs"]
mod publication_effects;
#[path = "completion/rewrite_materialization.rs"]
mod rewrite_materialization;
#[path = "completion/source_copy.rs"]
mod source_copy;

use crate::progression::PlannedPhysicalRecovery;

use super::context::PlanningContext;
use super::resolved_basis::ResolvedPlanningBasis;

pub(super) fn complete(
    context: PlanningContext,
    mut basis: ResolvedPlanningBasis,
) -> Result<PlannedPhysicalRecovery, crate::entry::PhysicalRecoveryOutcome> {
    let context = rewrite_materialization::install(context, &mut basis)?;
    let context = source_copy::verify(context, &mut basis)?;
    let context = blob_expiry::verify(context, &mut basis)?;
    let context = blob_reclaim::verify(context, &mut basis)?;
    let (context, execution) = execution_basis::derive(context, &mut basis)?;
    let (context, plan_cost, planning_counters) = plan_cost::admit(context, &basis, &execution)?;
    let context = publication_effects::admit(context, planning_counters, &execution.publication)?;
    Ok(planned_recovery::construct(
        context,
        basis,
        execution,
        plan_cost,
        planning_counters,
    ))
}
