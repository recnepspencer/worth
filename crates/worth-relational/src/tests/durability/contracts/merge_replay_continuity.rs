use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_recovery_preserves_merge_parent_order() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let main = create_entity_outcome(execution, &runtime, "main");
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let feature = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
        BranchId("feature".to_string()),
    );
    let merge = merge_commit_from_branches(
        execution,
        &runtime,
        BranchId("main".to_string()),
        vec![BranchId("feature".to_string())],
    );
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let replay = recovered.replay();
    let recovered_merge = replay
        .canonical_commit_envelope(execution, merge.commit.commit_id)
        .unwrap()
        .unwrap();

    assert_eq!(
        recovered_merge.commit.parents,
        vec![main.commit.commit_id, feature.commit.commit_id]
    );
    assert_eq!(
        recovered_merge.merge_parent_branches,
        vec![BranchId("feature".to_string())]
    );
    assert_eq!(
        recovered_merge.merge_base_commits,
        vec![main.commit.commit_id]
    );
}

#[test]
fn durability_contract_replays_empty_intent_merge_currentness_once() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
        BranchId("feature".to_string()),
    );
    let merge = merge_commit_from_branches(
        execution,
        &runtime,
        BranchId("main".to_string()),
        vec![BranchId("feature".to_string())],
    );
    let expected = runtime
        .branch_reference_state(&BranchId("main".to_string()))
        .expect("main state after empty-intent merge");
    let expected_target = expected
        .observation()
        .target()
        .as_basis()
        .expect("empty-intent merge still publishes a truth target");

    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let actual = recovered
        .branch_reference_state(&BranchId("main".to_string()))
        .expect("main state after recovery");
    let actual_target = actual
        .observation()
        .target()
        .as_basis()
        .expect("recovered empty-intent merge still has a truth target");

    assert_eq!(
        expected.observation().generation(),
        actual.observation().generation(),
        "recovery must apply the merge currentness transition exactly once"
    );
    assert_eq!(expected.truth_version(), actual.truth_version());
    assert_eq!(
        expected_target.selected_commit_id(),
        actual_target.selected_commit_id()
    );
    assert_eq!(expected_target.version_id(), actual_target.version_id());
    assert_eq!(
        merge.commit.commit_id.0,
        expected_target.selected_commit_id()
    );
}

#[test]
fn durability_contract_replays_merge_from_typed_authority_when_diagnostics_are_absent() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
        BranchId("feature".to_string()),
    );
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge execution");
    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed prepared merge");

    crate::tests::support::attached_test_medium(&runtime).rewrite_commit_envelope(
        execution,
        merge.commit.commit.commit_id,
        |merge_entry| {
            assert!(merge_entry.merge_execution_authority.is_some());
            let populated_nested_bytes = merge_entry
                .allocation_inventory()
                .authoritative_nested_bytes;
            let authority = merge_entry.merge_execution_authority.take().unwrap();
            let omitted_nested_bytes = merge_entry
                .allocation_inventory()
                .authoritative_nested_bytes;
            merge_entry.merge_execution_authority = Some(authority);
            assert!(populated_nested_bytes > omitted_nested_bytes);
            assert!(
                merge_entry.diagnostics_summary.entries.is_empty(),
                "diagnostics never reach the durable artifact"
            );
        },
    );

    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let replay = recovered.replay();
    let recovered_merge = replay
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("recovered merge envelope");

    let source_artifact = attached_test_medium(&runtime)
        .offered()
        .into_iter()
        .find(|artifact| artifact.commit_id() == merge.commit.commit.commit_id)
        .unwrap();
    let source =
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::readmit(
            source_artifact.as_bytes(),
            execution,
        )
        .unwrap();
    assert!(
        source.envelope().diagnostics_summary.entries.is_empty(),
        "the actual recovery source supplies no diagnostics"
    );
    crate::tests::support::assert_durable_commit_meaning_eq(
        execution,
        source.envelope(),
        &recovered_merge,
    );
    assert!(recovered_merge.merge_execution_authority.is_some());
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_string()))
            .expect("main head after recovery")
            .commit_id,
        merge.commit.commit.commit_id
    );
}
