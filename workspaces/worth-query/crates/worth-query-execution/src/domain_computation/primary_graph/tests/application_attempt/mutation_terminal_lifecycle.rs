use std::time::{Duration, Instant};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationCommitOutcome,
};

#[test]
fn committed_terminal() {
    let world = installed_authorization_world(true);
    let baseline = baselines(&world);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = admitted_program(&world, &principal, &account, &request, "terminal-commit");
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, idempotency(61, 61)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_baselines(&world, baseline);
}

#[test]
fn stale_terminal() {
    let world = installed_authorization_world(true);
    let baseline = baselines(&world);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let winner = admitted_program(&world, &principal, &account, &request, "terminal-winner");
    let stale = admitted_program(&world, &principal, &account, &request, "terminal-stale");
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(winner, idempotency(62, 62)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    super::assert_product_basis_stale(
        world
            .application
            .compare_and_commit_application(stale, idempotency(63, 63)),
        "the serial loser bound to the prior product",
    );
    assert_baselines(&world, baseline);
}

#[test]
fn cancelled_terminal() {
    let world = installed_authorization_world(true);
    let baseline = baselines(&world);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = admitted_program(&world, &principal, &account, &request, "terminal-cancel");
    cancellation.cancel();
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, idempotency(64, 64)),
        WorthQueryApplicationCommitOutcome::Cancelled
    ));
    assert_baselines(&world, baseline);
}

#[test]
fn provider_denial_terminal() {
    let world = installed_authorization_world(true);
    let baseline = baselines(&world);
    let program = program(&world, "terminal-denied");
    world.faults.reject_next_session_prepare();
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, idempotency(65, 65)),
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.stage() == WorthQueryApplicationCommitDenialStage::ProviderPlan
    ));
    assert_baselines(&world, baseline);
}

#[test]
fn aborted_terminal() {
    let world = installed_authorization_world(true);
    let baseline = baselines(&world);
    let program = program(&world, "terminal-abort");
    world.faults.reject_next_commit_before_transaction();
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, idempotency(66, 66)),
        WorthQueryApplicationCommitOutcome::Aborted
    ));
    assert_baselines(&world, baseline);
}

#[test]
fn skipped_owner_admission_is_a_denied_terminal() {
    let world = installed_authorization_world(true);
    let baseline = baselines(&world);
    let program = program(&world, "terminal-skipped-owner");
    world.faults.skip_next_invariant_owner_execution();
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, idempotency(67, 67)),
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.stage() == WorthQueryApplicationCommitDenialStage::InvariantExecution
    ));
    assert_baselines(&world, baseline);
}

#[test]
fn response_loss_terminal() {
    let world = installed_authorization_world(true);
    let baseline = baselines(&world);
    let program = program(&world, "terminal-response-loss");
    world.faults.lose_next_commit_response();
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, idempotency(68, 68)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_baselines(&world, baseline);
}

#[test]
fn idempotent_retry_terminal() {
    let world = installed_authorization_world(true);
    let baseline = baselines(&world);
    let first = program(&world, "terminal-idempotent");
    let retry = program(&world, "terminal-idempotent");
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(first, idempotency(69, 69)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(retry, idempotency(69, 69)),
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
    ));
    assert_baselines(&world, baseline);
}

fn program(
    world: &super::super::fixture::AuthorizationWorld,
    replacement: &str,
) -> super::program_fixture::Program {
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let account = resolved_account(world, "open", &request);
    admitted_program(world, &principal, &account, &request, replacement)
}

fn baselines(world: &super::super::fixture::AuthorizationWorld) -> (usize, usize) {
    (
        world.invariant.active_snapshot_count(),
        world.application.provider_session_resource_count(),
    )
}

fn assert_baselines(world: &super::super::fixture::AuthorizationWorld, expected: (usize, usize)) {
    assert_eq!(world.invariant.active_snapshot_count(), expected.0);
    assert_eq!(
        world.application.provider_session_resource_count(),
        expected.1
    );
}
