use std::collections::BTreeSet;

const DIRECT_CAUSE_OWNER: &str =
    include_str!("../../../../../logic/invalidation/causality/dependency_admission.rs");
const DIRECT_PUBLICATION_OWNER: &str =
    include_str!("../../../../../logic/invalidation/causality/dependency_admission/publication.rs");
const SEMANTIC_DECISION_OWNER: &str =
    include_str!("../../../../../data/graph/runtime/effect/output_commit/semantic_decision.rs");
const PRODUCED_DELTA_OWNER: &str =
    include_str!("../../../../../data/graph/runtime/effect/output_commit/produced_delta.rs");
const REVALIDATION_OWNER: &str =
    include_str!("../../../../../logic/invalidation/causality/revalidation.rs");
const CAUSE_VALIDATION_OWNER: &str =
    include_str!("../../../../../logic/invalidation/causality/revalidation/cause_validation.rs");
const CAUSE_AGGREGATION_OWNER: &str =
    include_str!("../../../../../logic/invalidation/causality/cause_aggregation.rs");
const OUTPUT_COMMIT_OWNER: &str =
    include_str!("../../../../../data/graph/runtime/effect/output_commit.rs");
const EFFECT_TELEMETRY_OWNER: &str =
    include_str!("../../../../../data/graph/runtime/effect/evidence.rs");
const ARTIFACT_PREPARATION_OWNER: &str =
    include_str!("../../../../../data/graph/runtime/effect/artifact_preparation.rs");
const WAITER_PREPARATION_OWNER: &str = include_str!(
    "../../../../../data/graph/topology/pending_revalidation/resolution_preparation.rs"
);
const ROUTING_COUNTER_OWNER: &str =
    include_str!("../../../../../logic/invalidation/routing/counters.rs");
const CHECKPOINT_OWNER: &str =
    include_str!("../../../../../data/graph/runtime/graph/checkpoint.rs");
const PLANNING_OWNER: &str = include_str!("../../../../../logic/planner/planning/mod.rs");
const EXECUTION_OWNER: &str = include_str!("../../../../../logic/planner/execution/mod.rs");
const EXECUTION_STAGE_OWNER: &str = include_str!("../../../../../logic/planner/execution/stage.rs");
const PRECOMPUTE_STAGE_OWNER: &str =
    include_str!("../../../../../logic/planner/precompute/stage.rs");
const PRECOMPUTE_DISPATCH_OWNER: &str =
    include_str!("../../../../../logic/planner/precompute/dispatch.rs");
const READ_PREPARATION_OWNER: &str =
    include_str!("../../../../../logic/planner/precompute/read_preparation.rs");
const CONCURRENT_APPLY_OWNER: &str =
    include_str!("../../../../../logic/planner/apply/stage/concurrent.rs");
const CONCURRENT_PACKET_OWNER: &str =
    include_str!("../../../../../logic/planner/apply/stage/concurrent_packets.rs");
const GRAPH_INVALIDATION_AUTHORITY_OWNER: &str =
    include_str!("../../../../../data/graph/storage/entries/invalidation_authority.rs");
const GRAPH_DIAGNOSTIC_SCAN_OWNER: &str =
    include_str!("../../../../../data/graph/storage/diagnostic_scan.rs");

#[test]
fn phase_1_inventory_rejects_unlisted_authority_and_execution_functions() {
    assert_owner_functions(
        DIRECT_PUBLICATION_OWNER,
        &[
            "suppressed_downstream_count",
            "admit_node_and_waiter_publication_work",
            "validate_packet",
            "prepare_direct_cause_publication",
            "publish_direct_output_causes",
        ],
    );
    assert_owner_functions(
        SEMANTIC_DECISION_OWNER,
        &[
            "build_semantic_artifact_write",
            "apply_semantic_output_commit_decision",
        ],
    );

    assert_owner_functions(PRODUCED_DELTA_OWNER, &["prepare_produced_delta"]);

    assert_owner_functions(
        WAITER_PREPARATION_OWNER,
        &[
            "from",
            "capture",
            "into_signal_error",
            "prepare_pending_revalidation_resolution",
            "load_node",
            "current_waiters",
            "resolve",
        ],
    );
    assert_owner_functions(
        ARTIFACT_PREPARATION_OWNER,
        &[
            "prepare_effect_artifact_write",
            "record_prepared_artifact_work",
        ],
    );
    assert_owner_functions(
        DIRECT_CAUSE_OWNER,
        &[
            "merge",
            "validate_packet",
            "prepare_direct_output_causes",
            "prepare_stable_output_resolution",
            "prepare_consumer_cause_set",
        ],
    );
    assert_owner_functions(
        REVALIDATION_OWNER,
        &[
            "node_invalidation_input",
            "resolved_dependency_causes",
            "pending_dependency_revalidation",
            "ensure_cause_readmission_complete",
            "readmit_checkpoint_causes",
            "validate_direct_invalidation_storage",
            "inject_pending_causes_unchecked_for_test",
        ],
    );
    assert_owner_functions(
        CAUSE_VALIDATION_OWNER,
        &[
            "validate_prepared_causes_before_evaluation",
            "validate_pending_causes",
            "validate_prepared_pending_causes",
            "validate_pending_cause",
            "commit_authority_matches",
            "validate_cause_identity_axes",
        ],
    );
    assert_owner_functions(
        CHECKPOINT_OWNER,
        &[
            "capture_checkpoint_authority",
            "restore_from_checkpoint_authority",
            "restore_from_checkpoint_image",
            "restore_node_arena",
            "rebuild_checkpoint_topology",
            "checkpoint_authority_arena_capacity",
            "checkpoint_authority_live_node_id_at",
            "capture_checkpoint_dependency_snapshot_batch",
            "derive_dependency_snapshot_restore_batch_from_checkpoint_batch",
        ],
    );
    assert_owner_functions(
        CAUSE_AGGREGATION_OWNER,
        &[
            "changed_scopes_for_edge",
            "reconcile_edge_cause",
            "reconcile_changed_scopes",
        ],
    );
    assert_owner_functions(
        OUTPUT_COMMIT_OWNER,
        &[
            "committed",
            "report",
            "apply_effect",
            "prepare_output_commit_packet",
            "prepare_output_commit_packet_with_probe",
            "prevalidate_output_commit_packet",
            "publish_output_commit_packet",
            "publish_prepared_parallel_apply_commit_packet",
        ],
    );
    assert_owner_functions(
        EFFECT_TELEMETRY_OWNER,
        &["record_effect_telemetry", "retains_runtime_cold_artifacts"],
    );
    assert_owner_functions(ROUTING_COUNTER_OWNER, &["record_diagnostic_projection"]);
}

#[test]
fn phase_1_inventory_rejects_unlisted_planner_entry_functions() {
    assert_owner_functions(
        PLANNING_OWNER,
        &[
            "build_evaluation_plan",
            "build_evaluation_plan_with_policy_resolver",
            "build_evaluation_cursor_with_policy_resolver",
            "build_evaluation_session_with_policy_resolver",
        ],
    );
    assert_owner_functions(
        EXECUTION_OWNER,
        &[
            "prepare_with_context",
            "execute_prepared_plan",
            "execute_prepared_plan_with_precompute",
            "execute_prepared_plan_with_policy",
            "execute_prepared_plan_with_policy_and_temporal_lowering",
            "execute_evaluation_session_with_policy",
            "execute_plan_stage_slices_with_policy",
        ],
    );
    assert_owner_functions(
        EXECUTION_STAGE_OWNER,
        &[
            "execute_stage",
            "run_stage_precompute_pass",
            "run_stage_apply_pass",
            "complete_stage_reporting_pass",
        ],
    );
}

#[test]
fn phase_1_inventory_rejects_unlisted_precompute_and_parallel_functions() {
    assert_owner_functions(
        PRECOMPUTE_STAGE_OWNER,
        &[
            "perform_stage_precompute",
            "run_snapshot_pass",
            "run_precompute_dispatch_pass",
        ],
    );
    assert_owner_functions(
        PRECOMPUTE_DISPATCH_OWNER,
        &[
            "dispatch_stage_precompute",
            "dispatch_stage_precompute_serial",
            "dispatch_stage_precompute_parallel",
        ],
    );
    assert_owner_functions(
        READ_PREPARATION_OWNER,
        &[
            "precompute_stage_serial",
            "precompute_stage_parallel",
            "build_parallel_stage_patches",
            "parallel_duplicate_task_index",
            "into_chunks",
            "split_prevalidated_work",
            "compute_work_item",
        ],
    );
    assert_owner_functions(
        CONCURRENT_APPLY_OWNER,
        &["run_grouped_concurrent_apply_pass"],
    );
    assert_owner_functions(
        CONCURRENT_PACKET_OWNER,
        &[
            "build_group_packet",
            "reduce_grouped_concurrent_packets",
            "publish_group_local_task_commit",
            "grouped_apply_failure_from_build_error",
            "record_grouped_apply_failure",
            "build_concurrent_apply_group_inputs",
            "take_slot",
            "can_lower_true_grouped_concurrent",
            "into_concurrent_worker_input",
        ],
    );
}

#[test]
fn phase_7_inventory_freezes_canonical_graph_owner_functions() {
    assert_owner_functions(
        GRAPH_INVALIDATION_AUTHORITY_OWNER,
        &[
            "node_dependency_revision",
            "node_pending_cause_set_id",
            "admit_pending_cause_handle_reads",
            "node_direct_invalidation_basis",
            "node_direct_invalidation_generation",
            "node_dirty_partition_scope_payload",
            "node_pending_revalidation",
            "set_node_pending_cause_set_id",
            "advance_node_dependency_revision",
            "replace_node_invalidation_cache",
            "install_node_dependency_revalidation",
            "publish_node_revalidation_resolution",
        ],
    );
    assert_owner_functions(
        GRAPH_DIAGNOSTIC_SCAN_OWNER,
        &[
            "node",
            "state",
            "dependencies",
            "subscribers",
            "node_runtime_artifact_state_present",
            "execution_record_present",
            "causality_present",
            "diagnostic_nodes",
        ],
    );
}

fn assert_owner_functions(source: &str, expected: &[&str]) {
    assert_eq!(
        declared_function_names(source),
        expected.iter().copied().collect::<BTreeSet<_>>()
    );
}

fn declared_function_names(source: &str) -> BTreeSet<&str> {
    source
        .lines()
        .filter_map(|line| {
            let declaration = line.trim_start();
            let function = declaration.find("fn ")?;
            let tail = &declaration[function + 3..];
            let end = tail.find(['<', '('])?;
            Some(&tail[..end])
        })
        .collect()
}
