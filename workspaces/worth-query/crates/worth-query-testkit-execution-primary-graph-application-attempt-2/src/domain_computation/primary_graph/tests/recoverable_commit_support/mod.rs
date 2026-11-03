//! Real World commit fixture for recovery and dispatch boundary tests.

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;

pub fn committed_recoverable_application() -> WorthQueryApplicationCommitReceipt {
    recoverable_application_world(241, "recovery-fixture-committed").1
}

pub fn committed_recoverable_application_with_output_demand_observation(
) -> WorthQueryApplicationCommitReceipt {
    let world = installed_authorization_world(true);
    commit_on_world_with_output_demand_observation(
        &world,
        241,
        "open",
        "performed-output-fixture-committed",
    )
}

pub fn recoverable_application_world(
    seed: u8,
    replacement: &str,
) -> (AuthorizationWorld, WorthQueryApplicationCommitReceipt) {
    let world = installed_authorization_world(true);
    let receipt = commit_on_world(&world, seed, "open", replacement);
    (world, receipt)
}

pub fn two_recoverable_application_commits(
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

pub fn commit_on_world(
    world: &AuthorizationWorld,
    seed: u8,
    current_status: &str,
    replacement: &str,
) -> WorthQueryApplicationCommitReceipt {
    commit_with_observation(world, seed, current_status, replacement, false)
}

pub fn commit_on_world_with_output_demand_observation(
    world: &AuthorizationWorld,
    seed: u8,
    current_status: &str,
    replacement: &str,
) -> WorthQueryApplicationCommitReceipt {
    commit_with_observation(world, seed, current_status, replacement, true)
}

pub fn commit_with_observation(
    world: &AuthorizationWorld,
    seed: u8,
    current_status: &str,
    replacement: &str,
    retain_output_demand_observation: bool,
) -> WorthQueryApplicationCommitReceipt {
    let request = live_scope();
    let program = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase;
            let principal =
                authenticated_principal(&(phase_custody).test_host_request_for_test(), world, &request);
            let account = resolved_account(
                &(phase_custody).test_host_request_for_test(),
                world,
                current_status,
                &request,
            );

            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program(
                &(phase_custody).test_host_request_for_test(),
                world,
                &principal,
                &account,
                &request,
                replacement,
                worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth::Narrow,
            )
        })
        .expect("the recoverable fixture opens its preparation call");
    let program = if retain_output_demand_observation {
        program.with_output_demand_observation()
    } else {
        program
    };
    match world.application.compare_and_commit_application(
        program,
        idempotency(seed, seed.wrapping_add(1)),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    ) {
        WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
        unexpected => panic!("real recoverable fixture must commit through World: {unexpected:?}"),
    }
}
