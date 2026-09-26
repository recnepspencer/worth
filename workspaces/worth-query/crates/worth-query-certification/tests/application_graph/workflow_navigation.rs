use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
    WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationAttemptDenialKind;

use super::bounded_dimension_model::{
    dimension_entry::PART_IDENTITY,
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    workflow::{
        accept_assessment, advance_instance, assessment_join_terminal_definition,
        propose_authoring_instance, publish_definition, settle_assessment, start_instance,
        WorkflowAdvanceInput, WorkflowAdvanceIntent,
    },
};

#[test]
fn back_is_one_published_transition_and_reconstructs_without_replaying_an_effect() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        assessment_join_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        2_310,
    )
    .expect("navigation definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("expected published definition: {other:?}"),
    };
    let instance = match start_instance(&application, definition.definition().clone(), 2_311)
        .expect("navigation instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected started instance: {other:?}"),
    };
    assert!(matches!(
        propose_authoring_instance(&application, instance.clone(), 2_312),
        Ok(WorkflowProposalOutcome::Published(_))
    ));

    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let navigate = |key: u64| {
        runtime
            .request(&principal, &scope)
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    part_identity: PART_IDENTITY.to_owned(),
                },
            })
            .without_source()
            .idempotency(&key)
            .prepare_workflow_navigate_back(&application, instance.clone())
            .expect("Back request must receive fresh admission")
            .execute()
    };

    assert!(
        matches!(
            navigate(2_313),
            Err(WorkflowProgressOutcome::PreparationDenied(denial))
                if denial.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported
        ),
        "Back cannot cross the performed proposal operation"
    );
    let first = settle_assessment(&application, instance.clone(), 2_314);
    assert_eq!(first.required().node_path(), "checks/first");
    assert!(matches!(
        accept_assessment(&application, instance.clone(), &first, 2_315),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    assert!(matches!(
        advance_instance(&application, instance.clone(), 2_316),
        Ok(WorkflowProgressOutcome::AwaitingAssessment(required))
            if required.node_path() == "checks/second"
    ));

    let before = runtime.workflow_instance_progress_counters();
    let back = navigate(2_317).expect("Back must publish its own transition");
    assert_eq!(back.node_path(), "checks/second");
    assert!(!back.replayed());
    assert_eq!(
        runtime
            .workflow_instance_progress_counters()
            .incremental_advances(),
        before.incremental_advances() + 1
    );
    let duplicate = navigate(2_317).expect("the same Back intent must replay its result");
    assert_eq!(duplicate.transition(), back.transition());
    assert!(duplicate.replayed());
    assert!(
        matches!(
            navigate(2_318),
            Err(WorkflowProgressOutcome::PreparationDenied(denial))
                if denial.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported
        ),
        "a fresh Back intent cannot cross the performed proposal operation"
    );

    runtime.release_workflow_instance_progress_for_test();
    let cold_duplicate = navigate(2_317).expect("cold replay must reconstruct the Back identity");
    assert_eq!(cold_duplicate.transition(), back.transition());
    assert!(cold_duplicate.replayed());
    match advance_instance(&application, instance, 2_319)
        .expect("cold reconstruction must admit the unchanged first assessment")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/first");
            assert!(
                performed.assessment_evidence().is_none(),
                "navigation reuses the published evidence rather than fabricating a second fact"
            );
        }
        other => panic!("expected retained first assessment after cold Back: {other:?}"),
    }
}
