//! A genuine approved effect is corrected without reopening its completed instance.
pub use super::*;
pub use worth_query_host::facade::{application_entry::*, runtime::ExecutionAllocationPolicy};
pub use worth_query_testkit_document_retention::facade::document_retention_model::{
    correctable_retention::{CorrectableRetentionBinding, CorrectableRetentionIntent},
    schema::SetRetentionInput,
    workflow::reviewed_document_definition_for_binding,
};
#[path = "correction/authored_divergence.rs"]
pub mod authored_divergence;

pub fn approved_source() -> (
    DocumentWorkflowRuntime,
    PublishedWorkflowInstanceRef,
    RequiredWorkflowOperation,
    worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt,
) {
    let application = worth_query_testkit_document_retention::facade::document_retention_model::host::publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        reviewed_document_definition_for_binding::<CorrectableRetentionBinding>("applied"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        9_180,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("definition must publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition, 9_181).unwrap() {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("instance must start: {other:?}"),
    };
    let (proposal, approval, _) =
        journey::approval_requirement(&application, instance.clone(), 9_180);
    assert!(matches!(
        approve_instance(
            &application,
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            9_189
        ),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let required = match advance_instance(&application, instance.clone(), 9_190).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("approved operation must be required: {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let input = SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 8,
    };
    let source = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(CorrectableRetentionIntent {
            input: input.clone(),
        })
        .without_source()
        .idempotency(&9_191_u64)
        .for_workflow_operation(&application, &required)
        .unwrap()
        .advanced()
        .execute_in_program(
            application.program_runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: source, ..
    } = source
    else {
        panic!("approved source must perform: {source:?}");
    };
    assert_eq!(read_retention(runtime, instance.branch()), 8);
    (application, instance, required, source)
}

pub fn correct_source(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    source: &worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    let request = application
        .runtime()
        .request(&principal, &scope)
        .on_branch(instance.branch());
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(source, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("9193")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("source must have an admitted ordinary inverse");
    };
    assert!(matches!(
        prepared.publish().unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(
        read_retention(application.runtime(), instance.branch()),
        SEED_RETENTION
    );
}
