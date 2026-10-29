//! Original intent is data across ownership loss; every lookup admits a fresh reader.
pub use super::super::super::super::{
    host,
    programs::{validated_second_program, RetentionProgramP1},
    workflow,
};
pub use super::*;
pub use worth_query_host::facade::application_entry::{
    PerformedWorkflowInstanceStart, PublishedWorkflowInstanceRef, WorkflowProgressOutcome,
};
pub use worth_query_host::facade::application_installation::{
    WorthQueryApplicationProgramRoster, WorthQueryCheckpointCapturePolicy,
};
#[path = "reacquisition/successors.rs"]
pub mod successors;

pub fn support_successor(application: &mut GovernedRuntime) {
    let supported = workflow::runtime::install_workflow_spec_for::<
        RetentionProgramP1,
        GovernedDocumentWorkflow,
    >(
        application.runtime().installed_schema(),
        application
            .supported_program::<RetentionProgramP1>()
            .unwrap()
            .installed_program(),
        workflow::runtime::workflow_resources(),
    );
    application
        .support_workflow_spec(supported)
        .unwrap_or_else(|(_supported, denial)| {
            panic!("the vocabulary belongs to a rostered program: {denial:?}")
        });
}
pub fn carry(
    application: &GovernedRuntime,
    revision: &worth_query_host::facade::declaration::application_program::ApplicationProgramRevision,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    let programs = application.runtime().request(&principal, &scope).programs();
    let comparison = programs.compare(revision).unwrap();
    let prepared = programs.adopt(&comparison).unwrap()
        .workflow(worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::carry_compatible()).prepare(1_024).unwrap();
    assert!(matches!(prepared.publish(application.runtime(), &principal, &scope), worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPublicationOutcome::Performed(_)));
}
pub fn assert_start_actor(view: &WorthQueryWorkflowObservation, expected: (u32, u64, u32)) {
    match view.audit() {
        WorthQueryWorkflowAudit::Visible { start, .. } => {
            let actor = start.entity_id();
            assert_eq!(
                (
                    actor.partition_value(),
                    actor.local_slot_value(),
                    actor.generation_value()
                ),
                expected
            );
        }
        _ => panic!("independent admitted audit reader must disclose the original actor"),
    }
}
pub fn started(outcome: WorkflowInstanceStartOutcome) -> PerformedWorkflowInstanceStart {
    match outcome {
        WorkflowInstanceStartOutcome::Started(done) => done,
        other => panic!("genuine start must publish: {other:?}"),
    }
}
pub fn live(view: WorthQueryWorkflowInstanceRead) -> PublishedWorkflowInstanceRef {
    match view {
        WorthQueryWorkflowInstanceRead::Live { instance, .. } => instance,
        _ => panic!("expected a genuinely live locator"),
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
