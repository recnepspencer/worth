use worth_relational::facade::history::BranchId;
use worth_runtime_bridge::facade::BridgeWritebackOutcomeClass;

use crate::effect_lifecycle::{
    bridge_observation_execution_receipt_subject_identity,
    bridge_observation_execution_record_subject_identity,
    bridge_observation_outcome_subject_identity, bridge_observation_receipt_subject_identity,
    bridge_observation_request_subject_identity, effect_batch, scope_admitted_effect_plan,
    BridgeExecutionOracle, EffectAuthoringBasis, EffectExecutionAuthority,
    EffectExecutionOracleErrorKind, EffectExecutionOracleVerificationKind,
    RelationalExecutionOracle,
};

use super::execution_support::{
    branch_snapshot_identity, create_entity, relational_runtime_with_intent_strategy,
    runtime_snapshot_identity, test_bridge_with_writeback_authority,
};
use super::support::{
    admitted_branch_merge_effect, admitted_mutation_effect_for_entity_with_binding,
    admitted_tenant_writeback_effect, branch_mutation_basis, native_name_patch,
    raw_mutation_effect_with_binding, runtime_workflow_binding_for_branch,
    runtime_workflow_binding_with_snapshot,
};

#[test]
fn mutation_execution_verifies_against_independent_relational_runtime_state() {
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
        native_name_patch("oracle-plan"),
    ))
    .lower()
    .expect("mutation should lower");

    let executed = lowered
        .execute_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect("mutation should execute");
    let verification = executed
        .verify_against_relational_runtime(&runtime)
        .expect("oracle verification should succeed");

    assert_eq!(
        verification.verification_kind(),
        EffectExecutionOracleVerificationKind::Mutation
    );
    assert_eq!(verification.component_count(), 1);
    assert!(verification.relational_oracle_for_reporting().is_some());
}

#[test]
fn merge_execution_verifies_against_independent_relational_runtime_state() {
    let mut runtime = relational_runtime_with_intent_strategy();
    create_entity(&mut runtime, "main", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("candidate".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("candidate branch should exist");
    create_entity(
        &mut runtime,
        "candidate-only",
        BranchId("candidate".to_string()),
    );
    let lowered = scope_admitted_effect_plan(admitted_branch_merge_effect())
        .lower()
        .expect("merge should lower");

    let executed = lowered
        .execute_with(
            EffectExecutionAuthority::crate_relational_without_application_schema(&mut runtime),
        )
        .expect("merge should execute");
    let verification = executed
        .verify_against_relational_runtime(&runtime)
        .expect("merge oracle verification should succeed");

    assert_eq!(
        verification.verification_kind(),
        EffectExecutionOracleVerificationKind::Merge
    );
    assert_eq!(verification.component_count(), 1);
    assert!(verification.relational_oracle_for_reporting().is_some());
}

#[test]
fn writeback_execution_verifies_against_independent_bridge_authority_record() {
    let bridge = test_bridge_with_writeback_authority();
    let lowered = scope_admitted_effect_plan(admitted_tenant_writeback_effect())
        .lower()
        .expect("writeback should lower");

    let executed = lowered
        .execute_with(EffectExecutionAuthority::bridge(&bridge))
        .expect("writeback should execute");
    let verification = executed
        .verify_against_bridge_runtime(&bridge)
        .expect("bridge oracle verification should succeed");
    let retained_record = bridge
        .diagnostics()
        .last_writeback_execution_record()
        .expect("bridge should retain the matching writeback execution record");
    assert_eq!(
        retained_record.execution_receipt_digest(),
        executed
            .writeback_execution()
            .map(|execution| execution.execution_receipt().digest())
    );

    let (outcome, receipt) = executed
        .as_writeback()
        .expect("writeback artifact should be present");
    assert_eq!(
        receipt.outcome_class(),
        BridgeWritebackOutcomeClass::AuthoritativeCommit
    );
    assert_eq!(receipt.failure_class(), None);
    assert_eq!(
        verification.verification_kind(),
        EffectExecutionOracleVerificationKind::Writeback
    );
    assert_eq!(verification.component_count(), 1);
    assert_eq!(
        outcome.authoritative_artifact_digest(),
        receipt.authoritative_artifact_digest()
    );
    assert!(verification.bridge_oracle_for_reporting().is_some());
}

#[test]
fn mutation_batch_verifies_against_independent_relational_runtime_state() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let left = create_entity(&mut runtime, "left", BranchId("main".to_string()));
    let right = create_entity(&mut runtime, "right", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should exist");
    let binding = runtime_workflow_binding_with_snapshot(runtime_snapshot_identity(&runtime));

    let executed = effect_batch()
        .using_basis(EffectAuthoringBasis::from(branch_mutation_basis()))
        .push(raw_mutation_effect_with_binding(
            binding.clone(),
            left,
            native_name_patch("left-oracle"),
        ))
        .push(raw_mutation_effect_with_binding(
            binding,
            right,
            native_name_patch("right-oracle"),
        ))
        .admit()
        .expect("batch should admit")
        .lower()
        .expect("batch should lower")
        .execute_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect("batch should execute");
    let verification = executed
        .verify_against_relational_runtime(&runtime)
        .expect("batch oracle verification should succeed");

    assert_eq!(
        verification.verification_kind(),
        EffectExecutionOracleVerificationKind::MutationBatch
    );
    assert_eq!(verification.component_count(), 2);
    assert!(verification.relational_oracle_for_reporting().is_some());
}

#[test]
fn relational_oracle_rejects_independent_commit_mismatch() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(&mut runtime, "before", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should exist");
    let lowered = scope_admitted_effect_plan(admitted_mutation_effect_for_entity_with_binding(
        runtime_workflow_binding_for_branch(
            branch_snapshot_identity(&runtime, "branch-a"),
            "branch-a",
        ),
        entity_id,
        native_name_patch("oracle-plan"),
    ))
    .lower()
    .expect("mutation should lower");
    let executed = lowered
        .execute_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect("mutation should execute");

    let denial = executed
        .verify_against_relational_oracle(&RelationalExecutionOracle::new(
            "branch-a",
            999_999,
            999_999,
            vec![111_111],
        ))
        .expect_err("mismatched relational oracle should deny");

    assert_eq!(
        denial.kind(),
        EffectExecutionOracleErrorKind::RelationalOracleCommitMismatch
    );
}

#[test]
fn bridge_oracle_rejects_independent_receipt_mismatch() {
    let bridge = test_bridge_with_writeback_authority();
    let lowered = scope_admitted_effect_plan(admitted_tenant_writeback_effect())
        .lower()
        .expect("writeback should lower");
    let executed = lowered
        .execute_with(EffectExecutionAuthority::bridge(&bridge))
        .expect("writeback should execute");
    let (outcome, receipt) = executed
        .as_writeback()
        .expect("writeback artifact should be present");

    let denial = executed
        .verify_against_bridge_oracle(&BridgeExecutionOracle::new(
            bridge_observation_execution_record_subject_identity("bridge-record:qa"),
            bridge_observation_outcome_subject_identity(outcome.digest()),
            outcome.outcome_class(),
            bridge_observation_request_subject_identity(receipt.request_digest()),
            bridge_observation_receipt_subject_identity("bridge-receipt-mismatch"),
        ))
        .expect_err("mismatched bridge oracle should deny");

    assert_eq!(
        denial.kind(),
        EffectExecutionOracleErrorKind::BridgeOracleReceiptMismatch
    );

    let execution_receipt_denial = executed
        .verify_against_bridge_oracle(
            &BridgeExecutionOracle::new(
                bridge_observation_execution_record_subject_identity("bridge-record:qa"),
                bridge_observation_outcome_subject_identity(outcome.digest()),
                outcome.outcome_class(),
                bridge_observation_request_subject_identity(receipt.request_digest()),
                bridge_observation_receipt_subject_identity(receipt.digest()),
            )
            .with_execution_receipt_subject_identity(
                bridge_observation_execution_receipt_subject_identity(
                    "bridge-execution-receipt-mismatch",
                ),
            ),
        )
        .expect_err("mismatched bridge execution receipt should deny");

    assert_eq!(
        execution_receipt_denial.kind(),
        EffectExecutionOracleErrorKind::BridgeOracleReceiptMismatch
    );
}

#[test]
fn bridge_runtime_verification_matches_the_executed_record_not_just_the_latest_record() {
    let bridge = test_bridge_with_writeback_authority();
    let lowered = scope_admitted_effect_plan(admitted_tenant_writeback_effect())
        .lower()
        .expect("writeback should lower");

    let first = lowered
        .clone()
        .execute_with(EffectExecutionAuthority::bridge(&bridge))
        .expect("first writeback should execute");
    let second = lowered
        .execute_with(EffectExecutionAuthority::bridge(&bridge))
        .expect("second writeback should execute");

    let first_verification = first
        .verify_against_bridge_runtime(&bridge)
        .expect("first writeback should still find its matching bridge record");
    let second_verification = second
        .verify_against_bridge_runtime(&bridge)
        .expect("second writeback should find its matching bridge record");

    assert_eq!(
        first_verification.verification_kind(),
        EffectExecutionOracleVerificationKind::Writeback
    );
    assert_eq!(
        second_verification.verification_kind(),
        EffectExecutionOracleVerificationKind::Writeback
    );
}
