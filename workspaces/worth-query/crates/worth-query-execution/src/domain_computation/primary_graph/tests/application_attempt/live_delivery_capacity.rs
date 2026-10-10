use worth_query_declaration::facade::application_schema::ApplicationRetainedEffectBinding;
use worth_runtime_bridge::facade::TruthBranchHeadSource;

use super::{admitted_program_with_emit, authenticated_principal, idempotency, resolved_account};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, AccountActivityBinding,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedChange, WorthQueryApplicationCommitOutcome,
};

#[test]
fn one_missing_batch_slot_denies_before_world_or_bridge_movement() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    assert_live_reservation_denial(0, u64::MAX, "slot-short", 211, resource_request);
}

#[test]
fn one_missing_payload_byte_denies_before_world_or_bridge_movement() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let payload = "byte-short".to_owned();
    let required = AccountActivityBinding::retained_bytes(&payload);
    assert!(required > 0);
    assert_live_reservation_denial(1, required - 1, &payload, 212, resource_request);
}

fn assert_live_reservation_denial(
    batch_capacity: usize,
    byte_capacity: u64,
    payload: &str,
    identity: u8,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let branch = world.application.current_world();
    let selected = world.application.on_branch(branch).select().unwrap();
    let product_before = selected.product().selected_commit().clone();
    let bridge_before = bridge_head_commit(&world, resource_request);
    let _live = world
        .application
        .primary_provider
        .observe_application_commit_causality(selected.product());
    world
        .application
        .primary_provider
        .set_application_commit_causality_limits(batch_capacity, byte_capacity);
    let program = admitted_program_with_emit(
        &world,
        &principal,
        &account,
        &request,
        "must-not-move",
        Some(payload),
    );

    let outcome = world
        .application
        .on_branch(branch)
        .transaction()
        .apply(WorthQueryAdmittedChange::new(
            program,
            idempotency(identity, identity),
        ))
        .commit()
        .expect("the public product token remains admitted");
    let Err(outcome) = outcome.require_committed() else {
        panic!("fanout reservation failure cannot satisfy require_committed");
    };
    assert!(matches!(
        outcome,
        WorthQueryApplicationCommitOutcome::Aborted
    ));
    let product_after = world.application.on_branch(branch).select().unwrap();
    assert_eq!(product_after.product().selected_commit(), &product_before);
    assert_eq!(bridge_head_commit(&world, resource_request), bridge_before);
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0
    );
    resolved_account(&world, "open", &request);
}

fn bridge_head_commit(
    world: &crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> worth_runtime_bridge::facade::TruthCommitIdentity {
    world
        .application
        .primary_provider
        .graph
        .relational_bridge_source()
        .load_branch_head_patch(
            &crate::domain_computation::primary_graph::primary_truth_branch_identity(),
            resource_request,
        )
        .expect("the bridge truth head remains available")
        .commit_identity()
        .clone()
}
