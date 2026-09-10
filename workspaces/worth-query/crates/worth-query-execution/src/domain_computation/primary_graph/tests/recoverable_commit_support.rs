//! Real World commit fixture for recovery and dispatch boundary tests.

use super::application_attempt::{authenticated_principal, idempotency, resolved_account};
use super::fixture::{installed_authorization_world, live_scope, AuthorizationWorld};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
};

pub(in crate::domain_computation) fn committed_recoverable_application(
) -> WorthQueryApplicationCommitReceipt {
    recoverable_application_world(241, "recovery-fixture-committed").1
}

pub(in crate::domain_computation) fn recoverable_application_world(
    seed: u8,
    replacement: &str,
) -> (AuthorizationWorld, WorthQueryApplicationCommitReceipt) {
    let world = installed_authorization_world(true);
    let receipt = commit_on_world(&world, seed, "open", replacement);
    (world, receipt)
}

pub(in crate::domain_computation) fn two_recoverable_application_commits(
    first_seed: u8,
    second_seed: u8,
) -> (
    AuthorizationWorld,
    WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationCommitReceipt,
) {
    let world = installed_authorization_world(true);
    let first = commit_on_world(&world, first_seed, "open", "first-recoverable-commit");
    let second = commit_on_world(
        &world,
        second_seed,
        "first-recoverable-commit",
        "second-recoverable-commit",
    );
    (world, first, second)
}

fn commit_on_world(
    world: &AuthorizationWorld,
    seed: u8,
    current_status: &str,
    replacement: &str,
) -> WorthQueryApplicationCommitReceipt {
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let account = resolved_account(world, current_status, &request);
    let program = super::application_attempt::preimage_evidence::retained_status_program(
        world,
        &principal,
        &account,
        &request,
        replacement,
        super::application_attempt::preimage_evidence::RetentionMutationBreadth::Narrow,
    );
    match world
        .application
        .compare_and_commit_application(program, idempotency(seed, seed.wrapping_add(1)))
    {
        WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
        unexpected => panic!("real recoverable fixture must commit through World: {unexpected:?}"),
    }
}
