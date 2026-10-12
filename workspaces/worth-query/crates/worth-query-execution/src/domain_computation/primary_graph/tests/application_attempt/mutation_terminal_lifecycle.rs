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
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let baseline = baselines(request_execution, &world);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "terminal-commit",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            program,
            idempotency(61, 61),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_baselines(request_execution, &world, baseline);
}

#[test]
fn stale_terminal() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let baseline = baselines(request_execution, &world);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let winner = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "terminal-winner",
            )
        })
        .expect("the fixture read owns its installed host call");
    let stale = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "terminal-stale",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            winner,
            idempotency(62, 62),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    super::assert_changed_decision(
        world.application.compare_and_commit_application(
            stale,
            idempotency(63, 63),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        ),
        "the serial loser bound to the prior product",
    );
    assert_baselines(request_execution, &world, baseline);
}

#[test]
fn cancelled_terminal() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let baseline = baselines(request_execution, &world);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "terminal-cancel",
            )
        })
        .expect("the fixture read owns its installed host call");
    cancellation.cancel();
    assert!(matches!(
        world.application.compare_and_commit_application(
            program,
            idempotency(64, 64),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Cancelled
    ));
    assert_baselines(request_execution, &world, baseline);
}

#[test]
fn provider_denial_terminal() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let baseline = baselines(request_execution, &world);
    let program = world
        .application
        .with_host_advancement(|phase| program(&phase, &world, "terminal-denied"))
        .expect("the fixture read owns its installed host call");
    world.faults.reject_next_session_prepare();
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, idempotency(65, 65), crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation),
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.stage() == WorthQueryApplicationCommitDenialStage::ProviderPlan
    ));
    assert_baselines(request_execution, &world, baseline);
}

#[test]
fn aborted_terminal() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let baseline = baselines(request_execution, &world);
    let program = world
        .application
        .with_host_advancement(|phase| program(&phase, &world, "terminal-abort"))
        .expect("the fixture read owns its installed host call");
    world.faults.reject_next_commit_before_transaction();
    assert!(matches!(
        world.application.compare_and_commit_application(
            program,
            idempotency(66, 66),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Aborted
    ));
    assert_baselines(request_execution, &world, baseline);
}

#[test]
fn skipped_owner_admission_is_a_denied_terminal() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let baseline = baselines(request_execution, &world);
    let program = world
        .application
        .with_host_advancement(|phase| program(&phase, &world, "terminal-skipped-owner"))
        .expect("the fixture read owns its installed host call");
    world.faults.skip_next_invariant_owner_execution();
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, idempotency(67, 67), crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation),
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.stage() == WorthQueryApplicationCommitDenialStage::InvariantExecution
    ));
    assert_baselines(request_execution, &world, baseline);
}

#[test]
fn response_loss_terminal() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let baseline = baselines(request_execution, &world);
    let program = world
        .application
        .with_host_advancement(|phase| program(&phase, &world, "terminal-response-loss"))
        .expect("the fixture read owns its installed host call");
    world.faults.lose_next_commit_response();
    assert!(matches!(
        world.application.compare_and_commit_application(
            program,
            idempotency(68, 68),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_baselines(request_execution, &world, baseline);
}

#[test]
fn idempotent_retry_terminal() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let baseline = baselines(request_execution, &world);
    let first = world
        .application
        .with_host_advancement(|phase| program(&phase, &world, "terminal-idempotent"))
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| program(&phase, &world, "terminal-idempotent"))
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            first,
            idempotency(69, 69),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert!(matches!(
        world.application.compare_and_commit_application(
            retry,
            idempotency(69, 69),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
    ));
    assert_baselines(request_execution, &world, baseline);
}

fn program(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &super::super::fixture::AuthorizationWorld,
    replacement: &str,
) -> super::program_fixture::Program {
    let request = live_scope();
    let principal = authenticated_principal(&(phase_custody).test_host_request(), world, &request);
    let account = resolved_account(
        &(phase_custody).test_host_request(),
        world,
        "open",
        &request,
    );
    admitted_program(
        phase_custody,
        world,
        &principal,
        &account,
        &request,
        replacement,
    )
}

fn baselines(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::super::fixture::AuthorizationWorld,
) -> (usize, usize) {
    (
        world.invariant.active_snapshot_count(request_execution),
        world.application.provider_session_resource_count_for_test(),
    )
}

fn assert_baselines(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::super::fixture::AuthorizationWorld,
    expected: (usize, usize),
) {
    assert_eq!(
        world.invariant.active_snapshot_count(request_execution),
        expected.0
    );
    assert_eq!(
        world.application.provider_session_resource_count_for_test(),
        expected.1
    );
}
