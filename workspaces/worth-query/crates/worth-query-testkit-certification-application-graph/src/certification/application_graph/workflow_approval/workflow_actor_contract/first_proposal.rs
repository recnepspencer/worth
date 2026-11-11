//! Actor courts retain the first actual proposal until its original identity is read.
use super::*;
use worth_query_host::facade::application_entry::PerformedWorkflowProposal;

pub fn approval_journey(
    completion: &str,
    key: u64,
) -> (
    DocumentWorkflowRuntime,
    PublishedWorkflowDefinitionRef,
    PublishedWorkflowInstanceRef,
    PerformedWorkflowProposal,
    RequiredWorkflowApproval,
    Vec<worth_relational::facade::identity::EntityId>,
) {
    let application =
        super::super::super::document_retention_model::host::publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        reviewed_document_definition(completion),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("actor definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("actor definition must actually publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition.definition().clone(), key + 1)
        .expect("actor instance prepares")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("actor instance must actually start: {other:?}"),
    };
    let proposal = match propose_instance(&application, instance.clone(), key + 2)
        .expect("actor proposal prepares")
    {
        WorkflowProposalOutcome::Published(performed) => performed,
        other => panic!("actor proposal must actually publish: {other:?}"),
    };
    let (required, evidence, _join) = super::super::journey::approval_requirement_after_proposal(
        &application,
        instance.clone(),
        key,
    );
    (
        application,
        definition.definition().clone(),
        instance,
        proposal,
        required,
        evidence,
    )
}
