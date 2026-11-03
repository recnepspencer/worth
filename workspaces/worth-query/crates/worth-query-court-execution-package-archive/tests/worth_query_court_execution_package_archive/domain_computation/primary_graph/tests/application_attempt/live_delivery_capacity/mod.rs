use worth_runtime_bridge::facade::TruthBranchHeadSource;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program_with_emit;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_execution::facade::product::WorthQueryAdmittedChange;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

#[test]
fn one_missing_batch_slot_denies_before_world_or_bridge_movement() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    assert_live_reservation_denial(0, "slot-short", 211, resource_request);
}

pub fn assert_live_reservation_denial(
    batch_capacity: usize,
    payload: &str,
    identity: u8,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_application_advancement(&request, |phase| {
            authenticated_principal(&phase.test_host_request(), &world, &request)
        })
        .unwrap();
    let account = world
        .application
        .with_application_advancement(&request, |phase| {
            resolved_account(&phase.test_host_request(), &world, "open", &request)
        })
        .unwrap();
    let branch = world.application.current_world();
    let selected = world
        .application
        .with_application_advancement(&request, |phase| {
            world.application.on_branch(branch).select(&phase)
        })
        .unwrap()
        .unwrap();
    let product_before = selected.product().selected_commit().clone();
    let bridge_before = bridge_head_commit(&world, resource_request);
    let _live = world
        .application
        .primary_provider
        .observe_application_commit_causality(selected.product());
    world
        .application
        .primary_provider
        .set_application_commit_causality_limits(batch_capacity);
    let program = world
        .application
        .with_application_advancement(&request, |phase| {
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "must-not-move",
                Some(payload),
            )
        })
        .unwrap();

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
    let product_after = world
        .application
        .with_application_advancement(&request, |phase| {
            world.application.on_branch(branch).select(&phase)
        })
        .unwrap()
        .unwrap();
    assert_eq!(product_after.product().selected_commit(), &product_before);
    assert_eq!(bridge_head_commit(&world, resource_request), bridge_before);
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0
    );
    world
        .application
        .with_application_advancement(&request, |phase| {
            resolved_account(&phase.test_host_request(), &world, "open", &request)
        })
        .unwrap();
}

pub fn bridge_head_commit(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> worth_runtime_bridge::facade::TruthCommitIdentity {
    world
        .application
        .primary_provider
        .graph
        .relational_bridge_source()
        .load_branch_head_patch(
            &worth_query_execution::domain_computation::primary_graph::application_branch::primary_truth_branch_identity(),
            resource_request,
        )
        .expect("the bridge truth head remains available")
        .commit_identity()
        .clone()
}
