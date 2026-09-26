//! Program adoption decides every live workflow fact on the branch explicitly:
//! carry it to the target in place, retire a definition, or cancel an instance
//! that has performed nothing.

use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryWorkflowInstanceStartPreparationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryBranchAdoptionPreparationDenial,
    WorthQueryWorkflowDefinitionDisposition as DefinitionDisposition,
    WorthQueryWorkflowInstanceCustody,
    WorthQueryWorkflowInstanceDisposition as InstanceDisposition,
};

use crate::bounded_dimension_model::{
    host::{publish_workflow_on_first_program, BoundedDimensionWorkflowRuntime},
    programs::DimensionProgramP1,
    workflow::{
        advance_instance, advance_on_second, prepare_second_program_adoption, publish_adoption,
        publish_definition, second_program_workflow_inventory, second_revision, start_instance,
        start_on_second, support_workflow_program, terminal_definition,
    },
};

#[test]
fn carry_moves_a_live_instance_and_its_definition_to_the_target() {
    let (application, definition, instance) = live_instance_on_first_program(85_000);
    let main = application.current_world();

    let inventory = match prepare_second_program_adoption(&application, main, None) {
        Err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRequired { inventory },
        )) => inventory,
        Err(other) => panic!("live workflow facts must be decided: {other:?}"),
        Ok(_) => panic!("live workflow facts must be decided before adoption"),
    };
    let current = inventory
        .definition(definition.entity_id())
        .expect("the current definition is inventoried");
    assert!(current.compatibility().is_compatible());
    assert_eq!(
        current.legal_dispositions(),
        &[DefinitionDisposition::Carry, DefinitionDisposition::Retire],
    );
    let live = inventory
        .instance(instance.entity_id())
        .expect("the live instance is inventoried");
    assert_eq!(
        live.custody(),
        &WorthQueryWorkflowInstanceCustody::Unperformed
    );
    assert_eq!(
        live.legal_dispositions(),
        &[InstanceDisposition::Carry, InstanceDisposition::Cancel],
    );

    let (published_under, carried) = application
        .runtime()
        .workflow_definition_revisions_for_test(&definition);
    assert!(published_under.is_some() && carried.is_none());
    publish_adoption(prepare_second_program_adoption(
        &application,
        main,
        Some(&|inventory| inventory.carry_compatible().unwrap()),
    ));
    let (still_published_under, carried) = application
        .runtime()
        .workflow_definition_revisions_for_test(&definition);
    assert_eq!(
        still_published_under, published_under,
        "carriage never rewrites publication provenance",
    );
    assert!(carried.is_some() && carried != published_under);
    match advance_on_second(&application, main, instance.clone(), 85_010)
        .expect("the carried instance advances under P1")
    {
        WorkflowProgressOutcome::Completed(transition) => assert!(transition.terminal()),
        other => panic!("the carried instance did not complete under P1: {other:?}"),
    }
    assert!(
        advance_instance(&application, instance, 85_011).is_err(),
        "the source vocabulary no longer serves the carried instance",
    );
    let restarted = start_on_second(&application, main, definition, 85_012);
    assert!(
        matches!(restarted, Ok(WorkflowInstanceStartOutcome::Started(_))),
        "the carried definition starts new P1 instances: {restarted:?}",
    );
    let carried = second_program_workflow_inventory(&application, main);
    assert_eq!(carried.source(), &second_revision(&application));
}

#[test]
fn retire_and_cancel_end_the_source_workflow_on_the_branch() {
    let (application, definition, instance) = live_instance_on_first_program(85_100);
    let main = application.current_world();
    publish_adoption(prepare_second_program_adoption(
        &application,
        main,
        Some(&|inventory| {
            let current = &inventory.definitions()[0];
            let live = &inventory.instances()[0];
            inventory
                .dispositions()
                .definition(current, DefinitionDisposition::Retire)
                .and_then(|choices| choices.instance(live, InstanceDisposition::Cancel))
                .expect("retire and cancel are legal for a compatible, unperformed instance")
        }),
    ));

    let cancelled = advance_on_second(&application, main, instance, 85_110);
    assert!(
        cancelled.is_err(),
        "a cancelled instance never advances: {cancelled:?}"
    );
    let retired = start_on_second(&application, main, definition, 85_111);
    assert!(
        retired.is_err(),
        "a retired definition starts nothing: {retired:?}"
    );
    let inventory = second_program_workflow_inventory(&application, main);
    assert!(
        inventory.is_empty(),
        "nothing remains live on the branch: {inventory:?}"
    );
}

#[test]
fn choices_made_against_a_moved_inventory_are_refused() {
    let (application, definition, _) = live_instance_on_first_program(85_200);
    let main = application.current_world();
    let stale = second_program_workflow_inventory(&application, main)
        .carry_compatible()
        .expect("everything is compatible");
    expect_started(start_instance(&application, definition, 85_210));

    match prepare_second_program_adoption(&application, main, Some(&move |_| stale.clone())) {
        Err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            WorthQueryBranchAdoptionPreparationDenial::WorkflowInventoryChanged { inventory },
        )) => assert_eq!(inventory.instances().len(), 2),
        Err(other) => panic!("stale choices must name the fresh inventory: {other:?}"),
        Ok(_) => panic!("choices made before the second start must not adopt"),
    }
    publish_adoption(prepare_second_program_adoption(
        &application,
        main,
        Some(&|inventory| inventory.carry_compatible().unwrap()),
    ));
}

fn live_instance_on_first_program(
    key: u64,
) -> (
    BoundedDimensionWorkflowRuntime,
    PublishedWorkflowDefinitionRef,
    PublishedWorkflowInstanceRef,
) {
    let mut application = publish_workflow_on_first_program();
    support_workflow_program::<DimensionProgramP1>(&mut application);
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

fn expect_started(
    outcome: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial>,
) -> PublishedWorkflowInstanceRef {
    match outcome.expect("the instance prepares") {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the instance did not start: {other:?}"),
    }
}
