//! Genuine public workflow publication through the Home's installed vocabulary.
use super::*;
use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
};

pub(super) fn publish(
    home: &memory_home::WorkflowHome,
    key: u64,
) -> WorkflowDefinitionPublicationOutcome {
    let runtime = home.runtime().runtime();
    let scope = document::operator_identity::request_scope();
    let principal =
        document::operator_identity::authenticate_operator(runtime.installed_schema(), &scope);
    let installed = home
        .runtime()
        .with_host_advancement(|phase| {
            home.vocabulary()
                .workflow_spec_on(&phase, runtime.current_world())
                .unwrap()
        })
        .unwrap();
    let contract = installed
        .bind_definition(document::workflow::reviewed_document_definition("approved"))
        .unwrap();
    runtime
        .request(&principal, &scope)
        .on_branch(runtime.current_world())
        .mutate(document::workflow::authoring_intent())
        .without_source()
        .idempotency(&key)
        .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
        .unwrap()
        .execute()
}
