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
use crate::document_retention_model::workflow::{
    advance_instance, advance_instance_on, prepare_second_program_adoption, publish_adoption,
    start_instance,
};

#[test]
fn a_start_between_preparation_and_publication_stales_the_adoption() {
    let (application, definition, first) = live_instance_on_first_program(85_300);
    let main = application.current_world();
    let choices = worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::new()
        .definition(
            &definition,
            worth_query_host::facade::primary_graph::WorthQueryWorkflowDefinitionDisposition::Carry,
        )
        .instance(
            &first,
            worth_query_host::facade::primary_graph::WorthQueryWorkflowInstanceDisposition::Carry,
        );
    let decided = choices.clone();
    let prepared = prepare_second_program_adoption(&application, main, Some(decided))
        .expect("the decided inventory prepares");
    let second = expect_started(start_instance(&application, definition, 85_310));

    let scope = crate::document_retention_model::operator_identity::request_scope();
    let principal = crate::document_retention_model::operator_identity::authenticate_operator(
        application.runtime().installed_schema(),
        &scope,
    );
    match prepared.publish(application.runtime(), &principal, &scope) {
        WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, .. } => {
            panic!("authorized adoption refused: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::AdmissionDenied { cause, .. } => {
            panic!("execution admission refused: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied(cause) => {
            panic!("the declared host policy admits adoption: {cause:?}")
        }
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
    match prepare_second_program_adoption(&application, main, Some(choices)) {
        Err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRejected,
        )) => {}
        Err(other) => panic!("re-preparation must inventory the new start: {other:?}"),
        Ok(_) => panic!("the old choices never decide the new start"),
    }

    publish_adoption(
        &application,
        prepare_second_program_adoption(
            &application,
            main,
            Some(worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::carry_compatible()),
        ),
    );
    for (instance, key) in [(first, 85_320), (second, 85_321)] {
        assert!(
            matches!(
                advance_instance(&application, instance, key),
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
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .branches()
                .fork(main)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&host_phase)
                .expect("the fork publishes")
        })
        .expect("installed host call opens");

    publish_adoption(
        &application,
        prepare_second_program_adoption(
            &application,
            fork,
            Some(worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::carry_compatible()),
        ),
    );
    match advance_instance_on(&application, fork, instance.clone(), 85_410) {
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
            WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRequired,
        )) => {}
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

#[test]
fn opaque_choices_cannot_cross_branches_or_decide_one_fact_twice() {
    use worth_query_host::facade::primary_graph::{
        WorthQueryWorkflowDefinitionDisposition as Definition,
        WorthQueryWorkflowDispositions as Choices,
        WorthQueryWorkflowInstanceDisposition as Instance,
    };
    let (application, definition, instance) = live_instance_on_first_program(85_450);
    let main = application.current_world();
    let fork = application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .branches()
                .fork(main)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    // These genuine source references do not become the fork's choices simply
    // because its relational component copied the entities.
    let foreign = Choices::new().definition(&definition, Definition::Carry);
    let duplicate = Choices::new()
        .definition(&definition, Definition::Carry)
        .definition(&definition, Definition::Retire)
        .instance(&instance, Instance::Carry);
    for (branch, choices) in [(fork, foreign), (main, duplicate), (main, Choices::new())] {
        assert!(
            matches!(
                prepare_second_program_adoption(&application, branch, Some(choices)),
                Err(
                    WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
                        WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRejected
                    )
                )
            ),
            "all refused choices expose the same payload-free native result"
        );
    }
    // Explicitly naming the fork's genuine held definition is lawful: copied
    // instance history is not a live obligation in that branch's inventory.
    let fork_choices = Choices::new().definition(&definition.held_on(fork), Definition::Carry);
    publish_adoption(
        &application,
        prepare_second_program_adoption(&application, fork, Some(fork_choices)),
    );
    assert!(
        matches!(
            advance_instance(&application, instance, 85_451),
            Ok(WorkflowProgressOutcome::Completed(_))
        ),
        "the source instance still progresses under its own program"
    );
}
