//! Original creation evidence, not discovery permission or an instance registry.
pub use super::document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime},
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::DOCUMENT_IDENTITY,
    schema::{Document, DocumentRetentionSchema},
    workflow::{
        publish_definition, terminal_definition, WorkflowInstanceStartInput,
        WorkflowInstanceStartIntent, WorkflowInstanceStartOperation,
    },
};
pub use worth_query_host::facade::application_entry::{
    PerformedWorkflowInstanceStart, PublishedWorkflowDefinitionRef,
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorthQueryApplicationRequestExt,
    WorthQueryWorkflowInstanceDiscoveryIntent, WorthQueryWorkflowInstanceStartRequest,
};

pub type Prepared<'a> = WorthQueryWorkflowInstanceStartRequest<
    'a,
    DocumentRetentionSchema,
    WorkflowInstanceStartOperation,
    WorkflowInstanceStartInput,
    Document,
>;
pub type Locator = WorthQueryWorkflowInstanceDiscoveryIntent<DocumentRetentionSchema>;

pub fn prepare(
    application: &DocumentWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    key: u64,
) -> Prepared<'_> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(definition.branch())
        .mutate(WorkflowInstanceStartIntent { input: input() })
        .without_source()
        .idempotency(&key)
        .prepare_workflow_instance_start(application, definition)
        .expect("authentic installed start prepares")
}

pub fn input() -> WorkflowInstanceStartInput {
    WorkflowInstanceStartInput {
        document_identity: DOCUMENT_IDENTITY.to_owned(),
    }
}

pub fn definition(
    application: &DocumentWorkflowRuntime,
    terminal: &str,
    predecessor: WorkflowDefinitionExpectedPredecessor,
    key: u64,
) -> PublishedWorkflowDefinitionRef {
    match publish_definition(application, terminal_definition(terminal), predecessor, key).unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the genuine workflow definition must publish: {other:?}"),
    }
}

pub fn started(outcome: WorkflowInstanceStartOutcome) -> PerformedWorkflowInstanceStart {
    match outcome {
        WorkflowInstanceStartOutcome::Started(performed) => performed,
        other => panic!("native start must land: {other:?}"),
    }
}

// Compiler body: SDK getter evidence outlives a consumed prepared request and
// needs no Input:Clone/Send/Sync or Schema:Clone/Eq bounds.
#[allow(dead_code)]
pub fn retain_sdk<
    S: worth_query_host::facade::declaration::application_schema::ApplicationSchema,
    O,
    I,
    C,
>(
    prepared: WorthQueryWorkflowInstanceStartRequest<'_, S, O, I, C>,
) -> WorthQueryWorkflowInstanceDiscoveryIntent<S> {
    let original = prepared.discovery_intent();
    drop(prepared);
    assert_eq!(original, original.clone());
    original
}
