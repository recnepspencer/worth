use worth_relational::facade::history::BranchId;

use crate::effect_lifecycle::{effect_batch, EffectAuthoringBasis, EffectExecutionAuthority};

use super::super::execution_support::{
    create_entity, relational_runtime_with_intent_strategy, runtime_snapshot_identity,
};
use super::super::support::{
    branch_mutation_basis, native_name_patch, raw_mutation_effect_with_binding,
    runtime_workflow_binding_with_snapshot,
};

#[test]
fn performed_batch_returns_settlement_deferred_without_denial_telemetry() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(
        request_execution,
        &mut runtime,
        "batch-before",
        BranchId("main".to_string()),
    );
    let baseline_published_snapshots = runtime
        .storage_access()
        .storage_stats()
        .published_snapshot_handle_count;
    let binding = runtime_workflow_binding_with_snapshot(runtime_snapshot_identity(&runtime));
    let lowered = effect_batch()
        .using_basis(EffectAuthoringBasis::from(branch_mutation_basis()))
        .push(raw_mutation_effect_with_binding(
            binding,
            entity_id,
            native_name_patch("batch-performed-before-settlement"),
        ))
        .admit()
        .expect("batch should admit")
        .lower()
        .expect("batch should lower");
    runtime.fail_next_durable_append_for_test();

    let stop = lowered
        .execute_with(
            request_execution,
            EffectExecutionAuthority::relational(&mut runtime),
        )
        .expect_err("performed batch must require settlement");
    let deferred = stop
        .settlement_deferred()
        .expect("performed batch is not an execution denial");
    assert!(stop.denial().is_none());
    assert_eq!(deferred.counters().executed_effect_count(), 1);
    assert_eq!(deferred.counters().execution_denied_count(), 0);
    assert_eq!(
        deferred.counters().publication_settlement_deferred_count(),
        1
    );
    let settlement = deferred.settlement().clone();
    assert_eq!(
        runtime.history().historical_latest_commit(),
        Some(settlement.commit().clone())
    );
    deferred
        .repair_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect("exact owner repairs the performed batch");
    deferred
        .repair_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect("the sole-owner pending record is gone, so a second repair removes no handle");
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats()
            .published_snapshot_handle_count,
        baseline_published_snapshots
    );
}

#[test]
fn dropped_batch_settlement_recovers_by_commit_id_and_reopens_the_route() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(
        request_execution,
        &mut runtime,
        "batch-abandon-before",
        BranchId("main".to_string()),
    );
    let baseline = runtime
        .storage_access()
        .storage_stats()
        .published_snapshot_handle_count;
    let binding = runtime_workflow_binding_with_snapshot(runtime_snapshot_identity(&runtime));
    let lowered = effect_batch()
        .using_basis(EffectAuthoringBasis::from(branch_mutation_basis()))
        .push(raw_mutation_effect_with_binding(
            binding,
            entity_id,
            native_name_patch("batch-abandoned-settlement"),
        ))
        .admit()
        .unwrap()
        .lower()
        .unwrap();
    runtime.fail_next_durable_append_for_test();

    let stop = lowered
        .execute_with(
            request_execution,
            EffectExecutionAuthority::relational(&mut runtime),
        )
        .expect_err("performed batch exposes deferred settlement");
    let commit_id = stop
        .settlement_deferred()
        .expect("performed batch retains recovery identity")
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
        .expect("batch authority recovers after the external token is dropped");
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
        "after-dropped-batch-settlement-repair",
        BranchId("main".to_string()),
    );
}
