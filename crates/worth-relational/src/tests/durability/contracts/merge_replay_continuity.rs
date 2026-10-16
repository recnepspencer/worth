use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_recovery_preserves_merge_parent_order() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let main = create_entity_outcome(execution, &runtime, "main");
    runtime
        .history_authority()
        .fork_branch_from(
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
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let replay = recovered.replay();
    let recovered_merge = replay
        .canonical_commit_envelope(merge.commit.commit_id)
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

    let runtime = persisted_runtime_with_test_schema();
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

    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = persisted_runtime_with_test_schema();
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

    let runtime = persisted_runtime_with_test_schema();
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

    let segment_path = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap()
        .store
        .unwrap()
        .segments
        .last()
        .expect("persisted segment after merge")
        .path
        .clone();
    let mut file =
        crate::durability::log::native_file_codec::read_segment_file(execution, &segment_path)
            .unwrap();
    let merge_entry = file
        .entries
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|entry| entry.commit.commit_id == merge.commit.commit.commit_id)
        .expect("merge entry in durable segment");
    assert!(merge_entry.merge_execution_authority.is_some());
    let populated_nested_bytes = merge_entry
        .allocation_inventory()
        .authoritative_nested_bytes;
    let mut omitted_merge_authority = merge_entry.clone();
    omitted_merge_authority.merge_execution_authority = None;
    assert!(
        populated_nested_bytes
            > omitted_merge_authority
                .allocation_inventory()
                .authoritative_nested_bytes
    );
    merge_entry.diagnostics_summary.entries.clear();
    crate::durability::log::native_file_codec::write_segment_file(execution, &segment_path, &file)
        .unwrap();

    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let replay = recovered.replay();
    let recovered_merge = replay
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("recovered merge envelope");

    assert!(recovered_merge.diagnostics_summary.entries.is_empty());
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

#[test]
fn durability_contract_reports_parent_order_parity_drift_when_durable_segment_is_tampered() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let main = create_entity_outcome(execution, &runtime, "main");
    create_branch_from_main(&runtime, "feature");
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

    let segment_path = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap()
        .store
        .unwrap()
        .segments
        .last()
        .expect("persisted segment after merge")
        .path
        .clone();
    let mut file =
        crate::durability::log::native_file_codec::read_segment_file(execution, &segment_path)
            .unwrap();
    let merge_entry = file
        .entries
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|entry| entry.commit.commit_id == merge.commit.commit_id)
        .expect("merge entry in durable segment");
    assert_eq!(
        merge_entry.commit.parents,
        vec![main.commit.commit_id, feature.commit.commit_id]
    );
    merge_entry.commit.parents.reverse();
    crate::durability::log::native_file_codec::write_segment_file(execution, &segment_path, &file)
        .unwrap();

    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::ReplayFailure);
    assert_eq!(
        error.history_drift_class,
        Some(crate::facade::history::HistoryDriftClass::DurabilityParityDrift)
    );
    assert!(error.detail.contains("parity drifted"));
}

#[test]
fn durability_contract_does_not_reconstruct_missing_merge_authority_from_diagnostics() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
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

    let segment_path = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap()
        .store
        .unwrap()
        .segments
        .last()
        .expect("persisted segment after merge")
        .path
        .clone();
    let mut file =
        crate::durability::log::native_file_codec::read_segment_file(execution, &segment_path)
            .unwrap();
    let merge_entry = file
        .entries
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|entry| entry.commit.commit_id == merge.commit.commit.commit_id)
        .expect("merge entry in durable segment");
    assert!(merge_entry.merge_execution_authority.is_some());
    assert!(!merge_entry.diagnostics_summary.entries.is_empty());
    merge_entry.merge_execution_authority = None;
    crate::durability::log::native_file_codec::write_segment_file(execution, &segment_path, &file)
        .unwrap();

    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::ReplayFailure);
    assert_eq!(error.history_drift_class, None);
    assert!(error
        .detail
        .contains("failed to reconstruct merge execution summary"));
}
