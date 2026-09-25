//! Every program-gated commit entry refuses an occurrence whose branch program
//! activation this host cannot attribute to an admitted rostered program.
//!
//! The fixture host carries genuine program support admitted through the real
//! roster path, and its activation cell is deliberately left unpublished, which
//! is the exact evidence a host that never seeded activation presents. Each
//! entry must fail closed on it, name the unresolved-activation cause, and
//! leave the product commit ledger where it was.

use super::super::fixture::{
    installed_authorization_world, installed_program_support, live_scope, rostered_program_revision,
};
use super::program_fixture::admitted_program_required_program;
use super::{authenticated_principal, idempotency, resolved_account};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome,
};

#[test]
fn a_program_action_on_an_unseeded_occurrence_names_the_unresolved_activation() {
    let mut world = installed_authorization_world(true);
    world.application.program_support = Some(installed_program_support(
        &world.application.installed_schema,
    ));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let predecessor = world.selected_product().product().selected_commit().clone();
    let program =
        admitted_program_required_program(&world, &principal, &account, &request, "program-owned");
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
        .compare_and_commit_application_for_program_action(
            &presented,
            program,
            idempotency(71, 71),
        );

    assert_unresolved_activation(outcome, "branch program activation was never published");
    assert_eq!(
        world.selected_product().product().selected_commit(),
        &predecessor
    );
}

#[test]
fn a_required_output_source_on_an_unseeded_occurrence_is_gated_like_its_siblings() {
    let mut world = installed_authorization_world(true);
    world.application.program_support = Some(installed_program_support(
        &world.application.installed_schema,
    ));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let predecessor = world.selected_product().product().selected_commit().clone();
    let program =
        admitted_program_required_program(&world, &principal, &account, &request, "program-owned");
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
        .compare_and_commit_application_for_required_output_source(
            &presented,
            program,
            idempotency(72, 72),
        );

    assert_unresolved_activation(outcome, "branch program activation was never published");
    assert_eq!(
        world.selected_product().product().selected_commit(),
        &predecessor
    );
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
