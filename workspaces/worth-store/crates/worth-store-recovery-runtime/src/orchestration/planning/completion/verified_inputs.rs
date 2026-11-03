//! Complete selected-input evidence before execution-product construction.

use super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningState};

pub(in crate::orchestration::planning) fn verify_inputs(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningState,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    context = super::rewrite_materialization::install(context, basis)?;
    context = super::source_copy::verify(context, basis)?;
    context = super::blob_expiry::verify(context, basis)?;
    context = super::blob_reclaim::verify_wal_drops(context, basis)?;
    super::blob_reclaim::verify_selected_custody(context, basis)
}
