//! Adoption decides exactly the live workflow facts the selected branch holds
//! at publication: a start after preparation stales the adoption, and a fork
//! decides only its own live facts, never the history it copied.

use worth_query_host::facade::application_entry::{
    WorkflowProgressOutcome, WorkflowTransitionPreparationDenial,
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryBranchAdoptionPublicationOutcome, WorthQueryWorkflowAdvancePreparationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryBranchAdoptionPreparationDenial,
};
use worth_query_host::facade::runtime::NoEffectCause;

use super::workflow_participant::{expect_started, live_instance_on_first_program};
use crate::bounded_dimension_model::workflow::{
    advance_instance, advance_on_second, prepare_second_program_adoption, publish_adoption,
    second_program_workflow_inventory, start_instance,
};

#[test]
fn a_start_between_preparation_and_publication_stales_the_adoption() {
    let (application, definition, first) = live_instance_on_first_program(85_300);
    let main = application.current_world();
    let choices = second_program_workflow_inventory(&application, main)
        .carry_compatible()
        .expect("everything is compatible");
    let decided = choices.clone();
    let prepared =
        prepare_second_program_adoption(&application, main, Some(&move |_| decided.clone()))
            .expect("the decided inventory prepares");
    let second = expect_started(start_instance(&application, definition, 85_310));

    match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => assert_eq!(
            no_effect.cause(),
            NoEffectCause::StaleExpectedProductHead,
            "the head fence refuses an adoption prepared before the start",
        ),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_) => {
            panic!("an adoption prepared before a start must not strand the new instance")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
            panic!("a stale adoption refuses before owner effects: {unpublished:?}")
        }
    }
    match prepare_second_program_adoption(&application, main, Some(&move |_| choices.clone())) {
        Err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            WorthQueryBranchAdoptionPreparationDenial::WorkflowInventoryChanged { inventory },
        )) => {
            assert_eq!(inventory.instances().len(), 2);
            assert!(inventory.instance(second.entity_id()).is_some());
        }
        Err(other) => panic!("re-preparation must inventory the new start: {other:?}"),
        Ok(_) => panic!("the old choices never decide the new start"),
    }

    publish_adoption(prepare_second_program_adoption(
        &application,
        main,
        Some(&|inventory| inventory.carry_compatible().unwrap()),
    ));
    for (instance, key) in [(first, 85_320), (second, 85_321)] {
        assert!(
            matches!(
                advance_on_second(&application, main, instance, key),
                Ok(WorkflowProgressOutcome::Completed(_))
            ),
            "both instances were carried and complete under P1",
        );
    }
}

#[test]
fn a_fork_adopts_without_deciding_the_instance_it_copied() {
    let (application, definition, instance) = live_instance_on_first_program(85_400);
    let main = application.current_world();
    let fork = application
        .runtime()
        .branches()
        .fork(main)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the fork publishes");

    let copied = second_program_workflow_inventory(&application, fork);
    assert!(
        copied.instances().is_empty(),
        "the copied instance is history on the fork and asks for no disposition",
    );
    assert!(copied.definition(definition.entity_id()).is_some());
    publish_adoption(prepare_second_program_adoption(
        &application,
        fork,
        Some(&|inventory| inventory.carry_compatible().unwrap()),
    ));
    match advance_on_second(&application, fork, instance.clone(), 85_410) {
        // Adoption carries only the fork's live facts, so the copied history
        // stays another branch's incarnation: the fork refuses it by
        // affinity before its program is consulted, under P1 as under P0.
        Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(attempt),
        )) if attempt.kind()
            == WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch => {}
        other => panic!("the adopted fork still refuses the copied instance: {other:?}"),
    }

    match prepare_second_program_adoption(&application, main, None) {
        Err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRequired { inventory },
        )) => assert!(
            inventory.instance(instance.entity_id()).is_some(),
            "the instance still waits on its own branch",
        ),
        Err(other) => panic!("the source branch still owes a disposition: {other:?}"),
        Ok(_) => panic!("the fork's adoption never decides the source branch's instance"),
    }
    assert!(
        matches!(
            advance_instance(&application, instance, 85_411),
            Ok(WorkflowProgressOutcome::Completed(_))
        ),
        "the source branch's instance still progresses under P0",
    );
}
