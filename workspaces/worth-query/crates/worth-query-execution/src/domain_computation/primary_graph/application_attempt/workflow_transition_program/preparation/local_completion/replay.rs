//! Exact native settlement replay precedes ended-state observation, never fresh admission.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationIdempotencyResolution, WorthQueryApplicationUncommitted,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve<S, O, I, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    admission: &WorthQueryAdmittedApplicationOperation<S, O, I, Scope>,
    required: &RequiredWorkflowOperation,
    binding: WorthQueryApplicationIdempotencyBinding,
    original: &WorthQueryApplicationCommitReceipt,
    receipt_identity: [u8; 32],
) -> Result<Option<WorkflowProgressOutcome>, WorkflowTransitionPreparationDenial>
where
    S: ApplicationSchema,
    I: Clone + Send + Sync + 'static,
{
    let [resolution] = runtime
        .resolve_admitted_application_idempotencies(execution, admission, [binding])
        .map_err(WorkflowTransitionPreparationDenial::IdempotencyInspection)?
        .try_into()
        .expect("one native settlement key");
    let locator = runtime
        .primary_provider
        .graph
        .layout
        .workflow()
        .transition
        .identity
        .clone();
    let receipt = match resolution.into_resolution() {
        WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => Some(receipt),
        WorthQueryApplicationIdempotencyResolution::IntentDrift => return Ok(Some(
            WorkflowProgressOutcome::Application(WorthQueryApplicationUncommitted::Denied(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::idempotency_intent_drift())))),
        WorthQueryApplicationIdempotencyResolution::Unseen => {
            super::super::super::publication::transition_entity_in_receipt(original, required.transition_identity(), &locator)
                .map(|_| original.clone())
        }
    };
    Ok(receipt.map(|receipt| {
        runtime
            .primary_provider
            .graph
            .with_runtime(execution, |relational| {
                super::super::super::publication::project(
                    execution,
                    relational,
                    receipt,
                    required.transition_identity().to_owned(),
                    locator,
                    required.node_path().to_owned(),
                    required.local_action_kind() == Some(WorkflowLocalActionKind::Terminate),
                    None,
                    None,
                    Some(receipt_identity),
                    true,
                )
            })
    }))
}
