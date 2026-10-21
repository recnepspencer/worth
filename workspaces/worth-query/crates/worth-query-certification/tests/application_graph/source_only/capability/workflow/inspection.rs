//! Evidence unavailability is not a no-effect claim and does not bypass fresh admission.
use super::{advance_input, host, program::Advance, Spec};
use super::{
    program::{Action, Effect, Program},
    CapabilitySchema,
};
use crate::document_retention_model::operator_identity::request_scope;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::{
    application_installation::WorthQueryWorkflowApplicationRuntime, primary_graph::*,
};
pub fn assert_unperformed(
    owner: &WorthQueryWorkflowOperation<CapabilitySchema, Effect, Program, Action>,
    allowed: &WorthQueryApplicationRequest<'_, '_, '_, CapabilitySchema>,
    denied: &WorthQueryApplicationRequest<'_, '_, '_, CapabilitySchema>,
) {
    assert!(matches!(
        owner.inspect_source(allowed),
        Err(WorthQueryWorkflowSourceInspectionDenial::SourceEvidenceUnavailable)
    ));
    assert!(
        matches!(
            owner.inspect_source(denied),
            Err(WorthQueryWorkflowSourceInspectionDenial::Opening(
                WorthQueryApplicationRecoveryRequestDenial::Request(
                    WorthQueryApplicationRequestMutationDenial::Authorization(_)
                )
            ))
        ),
        "fresh authorization precedes even absence of source evidence"
    );
}
pub fn denied_acceptance(
    workflow: &WorthQueryWorkflowApplicationRuntime<CapabilitySchema, Spec, Program>,
    host: &host::Host<Program>,
    instance: &PublishedWorkflowInstanceRef,
    completed: WorthQueryCompletedWorkflowOperation<CapabilitySchema, Effect, Program, Action>,
) -> WorthQueryCompletedWorkflowOperation<CapabilitySchema, Effect, Program, Action> {
    let scope = request_scope();
    let advancer = host::authenticate(host, 2, &scope);
    let request = workflow
        .runtime()
        .request(&advancer, &scope)
        .on_branch(instance.branch());
    let prepared = request
        .mutate(Advance(advance_input()))
        .without_source()
        .idempotency(&0x9176ca05_u64)
        .prepare_workflow_advance(workflow, instance.clone())
        .expect("actor2 has real advance capability");
    let (completed, cause) = prepared
        .accept_owned_operation(completed)
        .err()
        .expect("advance permission cannot authorize original effect acceptance");
    assert!(
        matches!(cause,WorthQueryWorkflowOperationOwnerAcceptanceDenial::Opening(WorthQueryApplicationRecoveryRequestDenial::Request(WorthQueryApplicationRequestMutationDenial::Authorization(ref denial))) if denial.kind()==WorthQueryOperationAuthorizationDenialKind::Refused(WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing))
    );
    completed
}
