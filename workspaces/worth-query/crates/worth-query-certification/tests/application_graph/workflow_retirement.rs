//! Public certification for definition retirement and lineage reopening.

use worth_query_host::facade::{
    application_entry::{
        PerformedWorkflowDefinitionRetirement, PublishedWorkflowDefinitionRef,
        PublishedWorkflowInstanceRef, WorkflowDefinitionExpectedPredecessor,
        WorkflowDefinitionPublicationOutcome, WorkflowDefinitionRetirementOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorthQueryApplicationRequestExt,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
        WorthQueryWorkflowDefinitionRetirementPreparationDenial,
        WorthQueryWorkflowInstanceStartPreparationDenial,
    },
    declaration::application_program::ValidatedWorkflowDefinition,
    primary_graph::WorthQueryApplicationCommitOutcome,
};

use super::bounded_dimension_model::{
    host::{publish_workflow_on_first_program, BoundedDimensionWorkflowRuntime},
    operator_identity::{authenticate_operator, request_scope},
    workflow::{
        advance_instance, authoring_intent, publish_definition, retire_definition, start_instance,
        terminal_definition, ReviewedGeometryWorkflow,
    },
};

#[path = "workflow_retirement/contention.rs"]
mod contention;
#[path = "workflow_retirement/denial.rs"]
mod denial;

#[test]
fn retirement_stops_new_starts_while_pinned_work_completes_and_the_lineage_reopens() {
    let application = publish_workflow_on_first_program();
    let first = expect_published(
        "first revision",
        publish_definition(
            &application,
            terminal_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            401,
        ),
    );
    let waiting = expect_started(start_instance(&application, first.clone(), 402));
    let second = expect_published(
        "second revision",
        publish_definition(
            &application,
            terminal_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(first.clone()),
            403,
        ),
    );

    let retired = expect_retired(retire_definition(&application, second.clone(), 404));
    assert!(!retired.replayed());
    assert_eq!(retired.definition(), &second);
    expect_stale_start(start_instance(&application, second.clone(), 405));
    expect_stale_start(start_instance(&application, first.clone(), 406));

    let replay = expect_retired(retire_definition(&application, second.clone(), 404));
    assert!(
        replay.replayed(),
        "an exact retirement retry recovers its receipt"
    );
    assert_eq!(
        replay.receipt().product_branch(),
        retired.receipt().product_branch()
    );
    expect_stale_retirement(retire_definition(&application, second.clone(), 407));
    expect_stale_retirement(retire_definition(&application, first.clone(), 408));

    match advance_instance(&application, waiting, 409)
        .expect("pinned work prepares against its retained definition")
    {
        WorkflowProgressOutcome::Completed(transition) => {
            assert_eq!(transition.node_path(), "completed");
            assert!(transition.terminal());
        }
        other => panic!("retirement must not strand pinned work, got {other:?}"),
    }

    expect_stale_publication(publish_definition(
        &application,
        terminal_definition("superseded"),
        WorkflowDefinitionExpectedPredecessor::Published(second.clone()),
        410,
    ));
    let reopened = expect_published(
        "reopened lineage",
        publish_definition(
            &application,
            terminal_definition("reopened"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            411,
        ),
    );
    assert_ne!(reopened.entity_id(), first.entity_id());
    assert_ne!(reopened.entity_id(), second.entity_id());
    let started = expect_started(start_instance(&application, reopened.clone(), 412));
    assert_eq!(started.current_node_path(), "reopened");
    expect_stale_start(start_instance(&application, second.clone(), 413));

    let replay_after_reopen = expect_retired(retire_definition(&application, second, 404));
    assert!(
        replay_after_reopen.replayed(),
        "reopening the lineage must not hide an exact retirement replay"
    );
    expect_stale_publication(publish_definition(
        &application,
        terminal_definition("second-reopen"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        414,
    ));
}

fn expect_published(
    context: &str,
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) -> PublishedWorkflowDefinitionRef {
    match result.expect("the workflow publication must prepare") {
        WorkflowDefinitionPublicationOutcome::Published(publication) => {
            publication.definition().clone()
        }
        unexpected => panic!("{context}: expected a publication, got {unexpected:?}"),
    }
}

fn expect_stale_publication(
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) {
    match result.expect("the workflow publication must reach commit comparison") {
        WorkflowDefinitionPublicationOutcome::Application(
            WorthQueryApplicationCommitOutcome::Stale(stale),
        ) => assert!(stale.stale_fact_count() > 0),
        unexpected => panic!("expected a stale publication predecessor, got {unexpected:?}"),
    }
}

fn expect_retired(
    result: Result<
        WorkflowDefinitionRetirementOutcome,
        WorthQueryWorkflowDefinitionRetirementPreparationDenial,
    >,
) -> PerformedWorkflowDefinitionRetirement {
    match result.expect("the workflow retirement must prepare") {
        WorkflowDefinitionRetirementOutcome::Retired(retired) => retired,
        unexpected => panic!("expected a retired definition, got {unexpected:?}"),
    }
}

fn expect_stale_retirement(
    result: Result<
        WorkflowDefinitionRetirementOutcome,
        WorthQueryWorkflowDefinitionRetirementPreparationDenial,
    >,
) {
    match result.expect("the workflow retirement must reach commit comparison") {
        WorkflowDefinitionRetirementOutcome::Application(
            WorthQueryApplicationCommitOutcome::Stale(stale),
        ) => assert!(stale.stale_fact_count() > 0),
        unexpected => panic!("expected a stale retirement, got {unexpected:?}"),
    }
}

fn expect_started(
    result: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial>,
) -> PublishedWorkflowInstanceRef {
    match result.expect("workflow instance-start preparation must succeed") {
        WorkflowInstanceStartOutcome::Started(started) => started.instance().clone(),
        other => panic!("workflow instance start failed: {other:?}"),
    }
}

fn expect_stale_start(
    result: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial>,
) {
    match result.expect("the workflow start must reach commit comparison") {
        WorkflowInstanceStartOutcome::Application(WorthQueryApplicationCommitOutcome::Stale(
            stale,
        )) => assert!(stale.stale_fact_count() > 0),
        unexpected => panic!("expected stale workflow instance start, got {unexpected:?}"),
    }
}
