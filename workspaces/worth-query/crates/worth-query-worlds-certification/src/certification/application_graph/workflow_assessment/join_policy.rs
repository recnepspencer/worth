pub use super::*;
pub use worth_query_host::facade::{
    application_contribution::WorthQueryWorkflowAssessmentPosture,
    application_entry::PublishedWorkflowInstanceRef,
    declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy,
};

pub use crate::facade::certification::document_retention_model::host::DocumentWorkflowRuntime;

pub fn failing_join(
    policy: ApplicationWorkflowEvidenceJoinPolicy,
    key: u64,
) -> (DocumentWorkflowRuntime, PublishedWorkflowInstanceRef) {
    const FAILING_RETENTION: u64 = 6;
    let application = publish_workflow_on_first_program();
    assert_eq!(
        settle(set_retention(
            application.program_runtime(),
            application.program_runtime().current_world(),
            FAILING_RETENTION,
            key
        )),
        RetentionVerdict::Performed(FAILING_RETENTION)
    );
    let definition = match publish_definition(
        &application,
        assessment_join_terminal_definition_with_policy(policy),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key + 1,
    )
    .expect("join definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected published definition, got {other:?}"),
    };
    let started = match start_instance(&application, definition.definition().clone(), key + 2)
        .expect("join instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected started instance, got {other:?}"),
    };
    propose_instance(&application, started.instance().clone(), key + 3)
        .expect("proposal must settle");
    for (settlement_offset, acceptance_offset) in [(4, 5), (6, 7)] {
        let settled = settle_assessment(
            &application,
            started.instance().clone(),
            key + settlement_offset,
        );
        assert_eq!(
            settled.posture(),
            WorthQueryWorkflowAssessmentPosture::Failing
        );
        assert!(matches!(
            accept_assessment(
                &application,
                started.instance().clone(),
                &settled,
                key + acceptance_offset
            ),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    (application, started.instance().clone())
}
