use crate::facade::history::BranchId;
use crate::facade::replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode};
use crate::tests::support::*;

#[test]
fn merge_ready_history_shape_reports_counter_breadth_explicitly() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let _root = create_entity_outcome(execution, &runtime, "root");
    let _linear = create_entity_outcome(execution, &runtime, "linear");
    create_branch_from_main(&runtime, "feature");
    let _feature = create_entity_outcome_on_branch(
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

    runtime.performance_access().reset_counters();

    let _ = runtime
        .history()
        .ancestor_closure_by_commit_id_order(merge.commit.commit_id);
    let replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: merge.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    assert!(replay.failure.is_none(), "{:?}", replay);

    let runtime_counters = runtime.performance_access().counters();
    assert!(runtime_counters.merge_history_ancestry_traversals >= 1);
    assert!(runtime_counters.merge_history_ancestry_nodes_visited >= 4);
    assert!(runtime_counters.merge_history_parent_comparisons >= 2);
    assert!(runtime_counters.merge_history_replay_planning_nodes_visited >= 4);
    assert!(runtime_counters.merge_history_replay_parent_checks >= 4);

    let recovery_plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, recovery_plan)
        .unwrap();
    let recovered_counters = recovered.performance_access().counters();
    assert!(recovered_counters.merge_history_durability_validation_nodes_visited >= 4);
    assert!(recovered_counters.merge_history_durability_parent_checks >= 4);
    assert!(recovered_counters.merge_history_parent_comparisons >= 2);
}
