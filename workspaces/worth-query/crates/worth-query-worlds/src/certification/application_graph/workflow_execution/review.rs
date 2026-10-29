//! Installed evidence floors and authored approval obligations use the same lane.

pub use super::super::document_retention_model::{
    retention_entry::ExplicitReviewedSetRetentionIntent,
    workflow::{
        accept_assessment, approve_instance, reviewed_document_definition_for_binding,
        settle_assessment, WorkflowGrantStatusInput, WorkflowGrantStatusIntent,
    },
};
pub use super::*;
pub use worth_query_host::facade::{
    application_entry::WorkflowApprovalDecision,
    primary_graph::WorthQueryApplicationAttemptDenialKind,
};

pub fn approved<Binding: ApplicationMutationBinding<DocumentRetentionSchema>>() -> (
    DocumentWorkflowRuntime,
    PublishedWorkflowInstanceRef,
    RequiredWorkflowOperation,
) {
    let application = publish_workflow_on_first_program();
    let published = match publish_definition(
        &application,
        reviewed_document_definition_for_binding::<Binding>("done"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        92001,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("review definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, published, 92002).unwrap() {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("review instance did not start: {other:?}"),
    };
    let proposal =
        match propose_authoring_instance_with_retention(&application, instance.clone(), 92003, 8)
            .unwrap()
        {
            WorkflowProposalOutcome::Published(performed) => performed.proposal().clone(),
            other => panic!("review proposal did not publish: {other:?}"),
        };
    for (settlement, acceptance) in [(92004, 92005), (92006, 92007)] {
        let evidence = settle_assessment(&application, instance.clone(), settlement);
        assert!(matches!(
            accept_assessment(&application, instance.clone(), &evidence, acceptance).unwrap(),
            WorkflowProgressOutcome::Completed(_)
        ));
    }
    assert!(matches!(
        advance_instance(&application, instance.clone(), 92008).unwrap(),
        WorkflowProgressOutcome::Completed(_)
    ));
    let approval = match advance_instance(&application, instance.clone(), 92009).unwrap() {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("authored review was bypassed: {other:?}"),
    };
    assert!(matches!(
        approve_instance(
            &application,
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            92010
        )
        .unwrap(),
        WorkflowProgressOutcome::Completed(_)
    ));
    let required = match advance_instance(&application, instance.clone(), 92011).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("reviewed effect did not become executable: {other:?}"),
    };
    (application, instance, required)
}
