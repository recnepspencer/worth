use worth_relational::facade::history::BranchId;
use worth_runtime_bridge::facade::BridgeWritebackOutcomeClass;

use super::execution_support::{
    create_entity, exact_branch_snapshot, relational_runtime_with_intent_strategy, test_bridge,
    test_bridge_with_writeback_authority,
};
use super::support::{
    admitted_branch_merge_effect, admitted_mutation_effect_for_entity_with_binding,
    admitted_tenant_writeback_effect, native_name_patch, runtime_workflow_binding_for_branch,
    runtime_workflow_binding_with_snapshot,
};
use crate::aspect_field_authoring::aspect_key;
use crate::effect_lifecycle::{
    scope_admitted_effect_plan, EffectExecutionAuthority, EffectExecutionDenialKind,
    ExecutedEffectAuthorityArtifact,
};

pub(super) use super::execution_support::{branch_snapshot_identity, runtime_snapshot_identity};

#[test]
fn lowered_mutation_execution_runs_through_relational_strategy_authority() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(&mut runtime, "before", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should be created");
    let lowered = scope_admitted_effect_plan(admitted_mutation_effect_for_entity_with_binding(
        runtime_workflow_binding_with_snapshot(runtime_snapshot_identity(&runtime)),
        entity_id,
        native_name_patch("authority-plan"),
    ))
    .lower()
    .expect("mutation should lower");

    let executed = lowered
        .execute_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect("lowered mutation should execute");

    assert!(matches!(
        executed.artifact(),
        ExecutedEffectAuthorityArtifact::Mutation(_)
    ));
    assert_eq!(executed.counters().executed_effect_count(), 1);
    assert_eq!(executed.counters().effect_execution_width(), 1);
    assert_eq!(
        executed.authority_owner(),
        executed.lowered().authority_owner()
    );
    let snapshot = exact_branch_snapshot(&mut runtime, "main");
    let read_view = runtime
        .read_truth()
        .read_snapshot(&snapshot)
        .expect("snapshot should read");
    let updated = read_view
        .entities()
        .iter()
        .find(|record| record.entity_id == entity_id)
        .expect("entity should still exist after execution");
    assert_eq!(entity_name(updated), Some("authority-plan".to_string()));
    assert!(runtime.snapshots().release_snapshot(&snapshot).is_ok());
}

#[test]
fn performed_mutation_returns_settlement_deferred_without_denial_telemetry() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(&mut runtime, "before", BranchId("main".to_string()));
    let baseline_published_snapshots = runtime
        .storage_access()
        .storage_stats()
        .published_snapshot_handle_count;
    let lowered = scope_admitted_effect_plan(admitted_mutation_effect_for_entity_with_binding(
        runtime_workflow_binding_with_snapshot(runtime_snapshot_identity(&runtime)),
        entity_id,
        native_name_patch("performed-before-settlement"),
    ))
    .lower()
    .expect("mutation should lower");
    runtime.fail_next_durable_append_for_test();

    let stop = lowered
        .execute_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect_err("performed mutation must require settlement");
    let deferred = stop
        .settlement_deferred()
        .expect("performed mutation is not an execution denial");
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
        .expect("exact owner repairs the performed mutation");
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
fn dropped_mutation_settlement_recovers_by_commit_id_and_reopens_the_route() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(&mut runtime, "abandon-before", BranchId("main".to_string()));
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
        .execute_with(EffectExecutionAuthority::relational(&mut runtime))
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
        &mut runtime,
        "after-dropped-settlement-repair",
        BranchId("main".to_string()),
    );
}

#[test]
fn lowered_merge_execution_runs_through_relational_merge_authority() {
    let mut runtime = relational_runtime_with_intent_strategy();
    create_entity(&mut runtime, "main", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("candidate".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("candidate branch should be created");
    create_entity(
        &mut runtime,
        "feature-only",
        BranchId("candidate".to_string()),
    );
    let lowered = scope_admitted_effect_plan(admitted_branch_merge_effect())
        .lower()
        .expect("merge should lower");

    let executed = lowered
        .execute_with(
            EffectExecutionAuthority::crate_relational_without_application_schema(&mut runtime),
        )
        .expect("lowered merge should execute");

    assert!(matches!(
        executed.artifact(),
        ExecutedEffectAuthorityArtifact::Merge(_)
    ));
    assert_eq!(executed.counters().executed_effect_count(), 1);
    assert_eq!(executed.counters().effect_execution_width(), 1);
    let merge = executed
        .as_merge()
        .expect("merge artifact should be present");
    assert_eq!(
        merge.commit.outcome().commit.version_id.0,
        runtime
            .history()
            .historical_latest_commit()
            .unwrap()
            .version_id
            .0
    );
}

#[test]
fn lowered_writeback_execution_requires_bridge_authority() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let lowered = scope_admitted_effect_plan(admitted_tenant_writeback_effect())
        .lower()
        .expect("writeback should lower");

    let denial = lowered
        .execute_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect_err("writeback execution should reject non-bridge authority");
    let denial = denial.denial().expect("authority rejection is a denial");

    assert_eq!(
        denial.denial_kind(),
        EffectExecutionDenialKind::AuthorityOverrideRejected
    );
    assert_eq!(denial.counters().execution_denied_count(), 1);
}

#[test]
fn lowered_mutation_execution_rejects_bridge_host_override() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(&mut runtime, "before", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should be created");
    let lowered = scope_admitted_effect_plan(admitted_mutation_effect_for_entity_with_binding(
        runtime_workflow_binding_for_branch(
            branch_snapshot_identity(&runtime, "branch-a"),
            "branch-a",
        ),
        entity_id,
        native_name_patch("authority-plan"),
    ))
    .lower()
    .expect("mutation should lower");

    let denial = lowered
        .execute_with(EffectExecutionAuthority::bridge(&test_bridge()))
        .expect_err("mutation execution should reject bridge host override");
    let denial = denial.denial().expect("authority rejection is a denial");

    assert_eq!(
        denial.denial_kind(),
        EffectExecutionDenialKind::AuthorityOverrideRejected
    );
    assert_eq!(denial.counters().execution_denied_count(), 1);
}

#[test]
fn lowered_merge_execution_rejects_bridge_host_override() {
    let lowered = scope_admitted_effect_plan(admitted_branch_merge_effect())
        .lower()
        .expect("merge should lower");

    let denial = lowered
        .execute_with(EffectExecutionAuthority::bridge(&test_bridge()))
        .expect_err("merge execution should reject bridge host override");
    let denial = denial.denial().expect("authority rejection is a denial");

    assert_eq!(
        denial.denial_kind(),
        EffectExecutionDenialKind::AuthorityOverrideRejected
    );
    assert_eq!(denial.counters().execution_denied_count(), 1);
}

#[test]
fn lowered_writeback_execution_denies_without_bound_writeback_authority() {
    let lowered = scope_admitted_effect_plan(admitted_tenant_writeback_effect())
        .lower()
        .expect("writeback should lower");

    let denial = lowered
        .execute_with(EffectExecutionAuthority::bridge(&test_bridge()))
        .expect_err("writeback execution should fail when no writeback authority is bound");
    let denial = denial.denial().expect("missing authority is a denial");

    assert_eq!(
        denial.denial_kind(),
        EffectExecutionDenialKind::BridgeWritebackExecutionFailed
    );
    assert_eq!(denial.counters().execution_denied_count(), 1);
}

#[test]
fn lowered_writeback_execution_runs_through_bridge_authority() {
    let bridge = test_bridge_with_writeback_authority();
    let lowered = scope_admitted_effect_plan(admitted_tenant_writeback_effect())
        .lower()
        .expect("writeback should lower");

    let executed = lowered
        .execute_with(EffectExecutionAuthority::bridge(&bridge))
        .expect("lowered writeback should execute");

    let (outcome, receipt) = executed
        .as_writeback()
        .expect("writeback artifact should be present");
    let execution = executed
        .writeback_execution()
        .expect("writeback execution proof should be retained");
    assert_eq!(
        executed.authority_owner(),
        executed.lowered().authority_owner()
    );
    assert_eq!(executed.counters().executed_effect_count(), 1);
    assert_eq!(executed.counters().effect_execution_width(), 1);
    assert_eq!(
        receipt.outcome_class(),
        BridgeWritebackOutcomeClass::AuthoritativeCommit
    );
    assert_eq!(receipt.failure_class(), None);
    assert_eq!(
        outcome.authoritative_artifact_digest(),
        receipt.authoritative_artifact_digest()
    );
    assert_eq!(execution.outcome().digest(), outcome.digest());
    assert_eq!(
        execution.execution_receipt().authority_receipt_digest(),
        receipt.digest()
    );
    let record = bridge
        .diagnostics()
        .last_writeback_execution_record()
        .expect("bridge should retain one writeback execution record");
    assert_eq!(record.outcome_digest(), Some(outcome.digest()));
    assert_eq!(record.receipt_digest(), Some(receipt.digest()));
    assert_eq!(record.request_digest(), Some(receipt.request_digest()));
}

fn entity_name(record: &worth_relational::facade::runtime::EntityReadRecord) -> Option<String> {
    let state = record.authoritative_aspect_state.as_ref()?;
    let value = state.get(&aspect_key("name").ok()?)?;
    match value.view() {
        worth_foundational::facade::ContractValidatedAspectValueView::Scalar(
            worth_foundational::facade::AspectValue::String(
                worth_foundational::facade::InternedString::Raw(actual),
            ),
        ) => Some(actual.clone()),
        _ => None,
    }
}
