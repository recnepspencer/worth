//! dropped mutation settlement recovers by commit id and reopens the route.
use super::*;

#[test]
fn dropped_mutation_settlement_recovers_by_commit_id_and_reopens_the_route() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(
        request_execution,
        &mut runtime,
        "abandon-before",
        BranchId("main".to_string()),
    );
    let baseline = runtime
        .storage_access()
        .storage_stats()
        .published_snapshot_handle_count;
    let lowered = scope_admitted_effect_plan(admitted_mutation_effect_for_entity_with_binding(
        runtime_workflow_binding_with_snapshot(runtime_snapshot_identity(&runtime)),
        entity_id,
        native_name_patch("abandoned-settlement"),
    ))
    .lower()
    .unwrap();
    runtime.fail_next_durable_append_for_test();

    let stop = lowered
        .execute_with(
            request_execution,
            EffectExecutionAuthority::relational(&mut runtime),
        )
        .expect_err("performed mutation exposes deferred settlement");
    let commit_id = stop
        .settlement_deferred()
        .expect("performed mutation retains recovery identity")
        .commit_id();
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats()
            .published_snapshot_handle_count,
        baseline + 1
    );
    drop(stop);
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats()
            .published_snapshot_handle_count,
        baseline + 1
    );
    EffectExecutionAuthority::relational(&mut runtime)
        .repair_pending_settlement(commit_id)
        .expect("effect authority recovers after the external token is dropped");
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats()
            .published_snapshot_handle_count,
        baseline
    );
    create_entity(
        request_execution,
        &mut runtime,
        "after-dropped-settlement-repair",
        BranchId("main".to_string()),
    );
}
