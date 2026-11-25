//! Pre-intent target-step budget admission delegating directly to Native's
//! exact staged cold-reconstruction preflight owner.
//!
//! Runs strictly before Store publishes any durable `TransitionPending` intent,
//! preventing an insufficient index maintenance budget from stranding the
//! predecessor Home on disk.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::application_home_integration::PreparedHomeProgramTransitionTarget;
use worth_relational::facade::{
    indexes::DerivedIndexMaintenanceBudget, runtime::RelationalRuntime,
};
use worth_runtime_world::facade::ProductBranchObservation;

use super::{
    plan::WorthQueryHomeProgramTransitionPlan,
    refusal::WorthQueryHomeProgramTransitionDenial as Denial,
};

/// Admit `resources` through Native's exact staged cold-reconstruction
/// preflight owner before Store reserves or publishes any durable
/// `TransitionPending` intent.
pub(super) fn preflight_fresh_target_budget<Schema: ApplicationSchemaComposition, Program>(
    native: &RelationalRuntime,
    source: &ProductBranchObservation,
    plan: &WorthQueryHomeProgramTransitionPlan,
    target: &PreparedHomeProgramTransitionTarget<Schema, Program>,
    resources: DerivedIndexMaintenanceBudget,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let target_batches = &plan.steps[plan.target as usize].batches;
    native
        .preflight_home_program_target_index_budget(
            request,
            source.basis().relational_basis(),
            target.native_contracts(),
            plan.schema_transition.clone(),
            target_batches,
            resources,
        )
        .map(|_| ())
        .map_err(Denial::Indexes)
}
