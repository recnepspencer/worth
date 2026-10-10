//! Every program-gated commit entry refuses an occurrence whose branch program
//! activation this host cannot attribute to an admitted rostered program.
//!
//! The fixture host carries genuine program support admitted through the real
//! roster path, and its activation cell is deliberately left unpublished, which
//! is the exact evidence a host that never seeded activation presents. Each
//! entry must fail closed on it, name the unresolved-activation cause, and
//! leave the product commit ledger where it was.

use super::super::fixture::{
    installed_authorization_world, installed_program_support, live_scope,
    rostered_program_revision, seed_program_activation,
};
use super::program_fixture::admitted_program_required_program;
use super::{authenticated_principal, idempotency, resolved_account};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome,
};

#[test]
fn program_activation_changed_after_preparation_cannot_readmit_the_old_program() {
    use super::super::fixture::{publish_relational_mutation, unadmitted_program_revision};
    use crate::domain_computation::primary_graph::program_occurrence::program_revision_rendering;
    use std::collections::BTreeMap;
    use worth_relational::facade::transactions::{
        AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    };
    let mut world = installed_authorization_world(true);
    world.application.program_support = Some(installed_program_support(
        &world.application.installed_schema,
    ));
    let layout = world
        .application
        .primary_provider
        .graph
        .layout
        .program_activation()
        .clone();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            seed_program_activation(
                &host_custody.borrowed_request_for_test(),
                &world,
                &rostered_program_revision(),
            )
        })
        .expect("the fixture read owns its installed host call");
    let request = live_scope();
    let support = world.application.program_support.as_ref().unwrap();
    let activation = support.activation().published().unwrap();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program_required_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "old-program",
            )
        })
        .expect("the fixture read owns its installed host call");
    let presented = support.present(&rostered_program_revision()).unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            publish_relational_mutation(
                &host_custody.borrowed_request_for_test(),
                &world,
                WorkerIntentBatch::new("hostile-activation-change").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: activation,
                        fields: AspectFieldPatch::from(BTreeMap::from([(
                            layout.program_revision_locator,
                            program_revision_rendering(&unadmitted_program_revision()),
                        )])),
                    }),
                )),
            )
        })
        .expect("the fixture read owns its installed host call");
    let before = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .product()
                .selected_commit()
                .clone()
        })
        .expect("the fixture read owns its installed host call");
    let outcome = world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            world
                .application
                .compare_and_commit_application_for_program_action(&phase.borrowed_request_for_test(),
                    phase,
                    &presented,
                    program,
                    idempotency(73, 73),
                    crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
                 crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
        })
        .expect("fixture owner admits its advancement");
    assert_unresolved_activation(
        outcome,
        "branch program activation names no rostered program",
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .product()
                    .selected_commit(),
                &before
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn a_program_action_on_an_unseeded_occurrence_names_the_unresolved_activation() {
    let mut world = installed_authorization_world(true);
    world.application.program_support = Some(installed_program_support(
        &world.application.installed_schema,
    ));
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let predecessor = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .product()
                .selected_commit()
                .clone()
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program_required_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "program-owned",
            )
        })
        .expect("the fixture read owns its installed host call");
    let support = world
        .application
        .program_support
        .as_ref()
        .expect("the fixture host retains its admitted program support");
    let presented = support
        .present(&rostered_program_revision())
        .expect("the fixture roster admits its own program revision");

    let outcome = world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            world
                .application
                .compare_and_commit_application_for_program_action(&phase.borrowed_request_for_test(),
                    phase,
                    &presented,
                    program,
                    idempotency(71, 71),
                    crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
                 crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
        })
        .expect("fixture owner admits its advancement");

    assert_unresolved_activation(outcome, "branch program activation was never published");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .product()
                    .selected_commit(),
                &predecessor
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn a_required_output_source_on_an_unseeded_occurrence_is_gated_like_its_siblings() {
    let mut world = installed_authorization_world(true);
    world.application.program_support = Some(installed_program_support(
        &world.application.installed_schema,
    ));
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let predecessor = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .product()
                .selected_commit()
                .clone()
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program_required_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "program-owned",
            )
        })
        .expect("the fixture read owns its installed host call");
    let support = world
        .application
        .program_support
        .as_ref()
        .expect("the fixture host retains its admitted program support");
    let presented = support
        .present(&rostered_program_revision())
        .expect("the fixture roster admits its own program revision");

    let outcome = world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            world
                .application
                .compare_and_commit_application_for_required_output_source(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &presented,
                    program,
                    idempotency(72, 72),
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
        })
        .expect("fixture owner admits its advancement");

    assert_unresolved_activation(outcome, "branch program activation was never published");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .product()
                    .selected_commit(),
                &predecessor
            )
        })
        .expect("the fixture read owns its installed host call");
}

/// Reads one settled attempt as the fail-closed integrity refusal it must be.
///
/// The kind is asserted rather than the detail alone, because the refusal this
/// path reaches must be told apart from the presented-versus-active mismatch it
/// used to share a kind with, and the two share a stage.
fn assert_unresolved_activation(outcome: WorthQueryApplicationCommitOutcome, detail: &str) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("an unattributable occurrence must deny before any effect: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProgramActivationUnresolved
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProposalBinding
    );
    assert_eq!(denial.detail(), Some(detail));
}
