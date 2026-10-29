//! Foreign-Spec non-disclosure and authentic native locator provenance.
pub use super::super::super::super::{host, programs::RetentionProgramP1, workflow};
pub use super::*;
pub use worth_query_host::facade::application_entry::{
    PerformedWorkflowInstanceStart, PublishedWorkflowInstanceRef, WorkflowProgressOutcome,
};
pub use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
};

pub fn started(outcome: WorkflowInstanceStartOutcome) -> PerformedWorkflowInstanceStart {
    match outcome {
        WorkflowInstanceStartOutcome::Started(done) => done,
        other => panic!("genuine native start required: {other:?}"),
    }
}
pub fn live(view: WorthQueryWorkflowInstanceRead) -> PublishedWorkflowInstanceRef {
    match view {
        WorthQueryWorkflowInstanceRead::Live { instance, .. } => instance,
        _ => panic!("authentic current successor must be live"),
    }
}
pub fn start_input() -> WorkflowInstanceStartInput {
    WorkflowInstanceStartInput {
        document_identity: DOCUMENT_IDENTITY.to_owned(),
    }
}
pub fn advance(
    application: &GovernedRuntime,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
) -> WorkflowProgressOutcome {
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    application
        .runtime()
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(workflow::WorkflowAdvanceIntent {
            input: workflow::WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .prepare_workflow_advance(application.vocabulary(), instance)
        .unwrap()
        .execute()
}
