//! Adoption carries concrete workflow facts, never content: equal-content
//! publications and their instances stay distinct through a carry, and
//! nothing prepared under the source program survives adoption as authority.

use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowDefinitionRetirementOutcome,
    WorkflowProgressOutcome, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitOutcome;

use super::workflow_participant::{expect_started, live_instance_on_first_program};
use crate::bounded_dimension_model::{
    dimension_entry::PART_IDENTITY,
    operator_identity::{authenticate_operator, request_scope},
    workflow::{
        advance_on_second, prepare_second_program_adoption, publish_adoption, publish_definition,
        retire_definition, second_program_workflow_inventory, start_instance, terminal_definition,
        WorkflowAdvanceInput, WorkflowAdvanceIntent,
    },
};

#[test]
fn equal_content_publications_and_their_instances_never_alias() {
    let (application, first, early) = live_instance_on_first_program(85_500);
    let main = application.current_world();
    match retire_definition(&application, first.clone(), 85_521).expect("retirement prepares") {
        WorkflowDefinitionRetirementOutcome::Retired(_) => {}
        other => panic!("the first publication did not retire: {other:?}"),
    }
    let second = republish(&application, 85_522);
    assert_ne!(
        second.entity_id(),
        first.entity_id(),
        "an equal-content publication is its own definition",
    );
    let late = expect_started(start_instance(&application, second, 85_523));
    assert_ne!(late.entity_id(), early.entity_id());

    let inventory = second_program_workflow_inventory(&application, main);
    assert_eq!(inventory.instances().len(), 2);
    assert!(inventory.instance(early.entity_id()).is_some());
    assert!(inventory.instance(late.entity_id()).is_some());
    publish_adoption(prepare_second_program_adoption(
        &application,
        main,
        Some(&|inventory| inventory.carry_compatible().unwrap()),
    ));

    assert!(matches!(
        advance_on_second(&application, main, early.clone(), 85_530),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    match advance_on_second(&application, main, early, 85_531) {
        Ok(WorkflowProgressOutcome::Application(WorthQueryApplicationCommitOutcome::Stale(_))) => {}
        other => panic!("a completed instance never advances again: {other:?}"),
    }
    assert!(
        matches!(
            advance_on_second(&application, main, late, 85_532),
            Ok(WorkflowProgressOutcome::Completed(_))
        ),
        "the equal-content sibling keeps its own progress",
    );
}

#[test]
fn an_advance_prepared_under_the_source_program_is_stale_after_adoption() {
    let (application, _, instance) = live_instance_on_first_program(85_600);
    let main = application.current_world();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let prepared = runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&85_621_u64)
        .prepare_workflow_advance(&application, instance.clone())
        .expect("the advance prepares under P0");
    publish_adoption(prepare_second_program_adoption(
        &application,
        main,
        Some(&|inventory| inventory.carry_compatible().unwrap()),
    ));

    match prepared.execute() {
        WorkflowProgressOutcome::Application(WorthQueryApplicationCommitOutcome::Denied(_)) => {}
        other => panic!("a P0 binding never commits after the carry: {other:?}"),
    }
    assert!(
        matches!(
            advance_on_second(&application, main, instance, 85_622),
            Ok(WorkflowProgressOutcome::Completed(_))
        ),
        "the carried instance progresses only through the P1 binding",
    );
}

fn republish(
    application: &crate::bounded_dimension_model::host::BoundedDimensionWorkflowRuntime,
    key: u64,
) -> PublishedWorkflowDefinitionRef {
    match publish_definition(
        application,
        terminal_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the equal-content publication prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the equal-content publication did not publish: {other:?}"),
    }
}
