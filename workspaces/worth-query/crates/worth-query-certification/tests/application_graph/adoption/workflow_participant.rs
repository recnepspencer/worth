//! Program adoption decides every live workflow fact on the branch explicitly:
//! carry it to the target in place, retire a definition, or cancel an instance
//! that has performed nothing.

use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryWorkflowInstancePreparationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationUncommitted, WorthQueryBranchAdoptionPreparationDenial,
    WorthQueryWorkflowDefinitionDisposition as DefinitionDisposition,
    WorthQueryWorkflowInstanceDisposition as InstanceDisposition,
};

use crate::document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime},
    programs::RetentionProgramP1,
    workflow::{
        advance_instance, prepare_second_program_adoption, publish_adoption, publish_definition,
        second_revision, start_instance, support_workflow_program, terminal_definition,
    },
};

#[test]
fn carry_moves_a_live_instance_and_its_definition_to_the_target() {
    let (application, definition, instance) = live_instance_on_first_program(85_000);
    let main = application.current_world();

    assert!(matches!(
        prepare_second_program_adoption(&application, main, None),
        Err(
            WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
                WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRequired
            )
        )
    ));
    let (published_under, carried) = application
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .workflow_definition_revisions_for_test(&host_phase, &definition)
        })
        .expect("installed host call opens")
        .expect("workflow branch remains admitted")
        .expect("workflow activation facts remain readable");
    assert!(published_under.is_some() && carried.is_none());
    publish_adoption(
        &application,
        prepare_second_program_adoption(
            &application,
            main,
            Some(worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::carry_compatible()),
        ),
    );
    let (still_published_under, carried) = application
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .workflow_definition_revisions_for_test(&host_phase, &definition)
        })
        .expect("installed host call opens")
        .expect("workflow branch remains admitted")
        .expect("workflow activation facts remain readable");
    assert_eq!(
        still_published_under, published_under,
        "carriage never rewrites publication provenance",
    );
    assert!(carried.is_some() && carried != published_under);
    match advance_instance(&application, instance.clone(), 85_010)
        .expect("the carried instance advances under P1")
    {
        WorkflowProgressOutcome::Completed(transition) => assert!(transition.terminal()),
        other => panic!("the carried instance did not complete under P1: {other:?}"),
    }
    let finished = advance_instance(&application, instance, 85_011);
    assert!(
        matches!(
            finished,
            Ok(WorkflowProgressOutcome::Application(
                WorthQueryApplicationUncommitted::Stale(_)
            ))
        ),
        "a completed carried instance does not advance again: {finished:?}",
    );
    let restarted = start_instance(&application, definition, 85_012);
    assert!(
        matches!(restarted, Ok(WorkflowInstanceStartOutcome::Started(_))),
        "the carried definition starts new P1 instances: {restarted:?}",
    );
}

#[test]
fn retire_and_cancel_end_the_source_workflow_on_the_branch() {
    let (application, definition, instance) = live_instance_on_first_program(85_100);
    let main = application.current_world();
    publish_adoption(
        &application,
        prepare_second_program_adoption(
            &application,
            main,
            Some(
                worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::new()
                    .definition(&definition, DefinitionDisposition::Retire)
                    .instance(&instance, InstanceDisposition::Cancel),
            ),
        ),
    );

    let cancelled = advance_instance(&application, instance, 85_110);
    assert!(
        cancelled.is_err(),
        "a cancelled instance never advances: {cancelled:?}"
    );
    let retired = start_instance(&application, definition, 85_111);
    assert!(
        retired.is_err(),
        "a retired definition starts nothing: {retired:?}"
    );
}

#[test]
fn choices_made_against_a_moved_inventory_are_refused() {
    let (application, definition, first) = live_instance_on_first_program(85_200);
    let main = application.current_world();
    let stale = worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::new()
        .definition(&definition, DefinitionDisposition::Carry)
        .instance(&first, InstanceDisposition::Carry);
    expect_started(start_instance(&application, definition, 85_210));

    match prepare_second_program_adoption(&application, main, Some(stale)) {
        Err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRejected,
        )) => {}
        Err(other) => {
            panic!("incomplete choices must refuse without disclosing fresh inventory: {other:?}")
        }
        Ok(_) => panic!("choices made before the second start must not adopt"),
    }
    publish_adoption(
        &application,
        prepare_second_program_adoption(
            &application,
            main,
            Some(worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::carry_compatible()),
        ),
    );
}

pub(super) fn live_instance_on_first_program(
    key: u64,
) -> (
    DocumentWorkflowRuntime,
    PublishedWorkflowDefinitionRef,
    PublishedWorkflowInstanceRef,
) {
    let mut application = publish_workflow_on_first_program();
    support_workflow_program::<RetentionProgramP1>(&mut application);
    let definition = match publish_definition(
        &application,
        terminal_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the P0 definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the P0 definition did not publish: {other:?}"),
    };
    let instance = expect_started(start_instance(&application, definition.clone(), key + 1));
    (application, definition, instance)
}

pub(super) fn expect_started(
    outcome: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial>,
) -> PublishedWorkflowInstanceRef {
    match outcome.expect("the instance prepares") {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the instance did not start: {other:?}"),
    }
}
