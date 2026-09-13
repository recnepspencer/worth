mod chain_bootstrap;
mod dependency_reconciliation_rotating;
mod dependency_reconciliation_stable_shape;
mod dependency_reconciliation_staged;
mod fintech_fanout;
mod measured_packet;
mod measured_pressure;
mod named_scale_slopes;
mod observability_profile;
mod scheduled_node_bound;
mod suppression_fanout;
mod throughput_courtroom;
pub(crate) mod throughput_definition;
mod throughput_parity;
mod throughput_slopes;
mod throughput_world;
mod topology_rewiring;

use serde_json::json;

use crate::facade::{GraphMetrics, RuntimeMetrics, SignalRuntimePolicy};
use crate::tests::performance_support::{PerfCaseContract, PerfTimingPolicy};

pub(super) fn eval_metrics_delta(
    before: RuntimeMetrics,
    after: RuntimeMetrics,
) -> serde_json::Value {
    json!({
        "evaluation_calls": after.evaluation.evaluation_calls - before.evaluation.evaluation_calls,
        "evaluation_nanos": after.evaluation.evaluation_nanos - before.evaluation.evaluation_nanos,
        "nodes_evaluated": after.evaluation.nodes_evaluated - before.evaluation.nodes_evaluated,
        "nodes_recomputed": after.evaluation.nodes_recomputed - before.evaluation.nodes_recomputed,
        "skipped_by_comparator": after.evaluation.skipped_by_comparator - before.evaluation.skipped_by_comparator,
        "suppressed_downstream_propagations": after.evaluation.suppressed_downstream_propagations - before.evaluation.suppressed_downstream_propagations,
        "plans_built": after.planner.plans_built - before.planner.plans_built,
        "tasks_scheduled": after.planner.tasks_scheduled - before.planner.tasks_scheduled,
        "tasks_pruned_before_execution": after.planner.tasks_pruned_before_execution - before.planner.tasks_pruned_before_execution,
        "stage_execution_count": after.execution.stage_execution_count - before.execution.stage_execution_count,
        "stage_execution_nanos": after.execution.stage_execution_nanos - before.execution.stage_execution_nanos,
    })
}

pub(super) fn graph_metrics_delta(before: GraphMetrics, after: GraphMetrics) -> serde_json::Value {
    json!({
        "nodes_evaluated": after.evaluation.nodes_evaluated - before.evaluation.nodes_evaluated,
        "nodes_recomputed": after.evaluation.nodes_recomputed - before.evaluation.nodes_recomputed,
        "skipped_by_comparator": after.evaluation.skipped_by_comparator - before.evaluation.skipped_by_comparator,
        "suppressed_downstream_propagations": after.evaluation.suppressed_downstream_propagations - before.evaluation.suppressed_downstream_propagations,
        "rewiring_apply_count": after.execution.rewiring_apply_count - before.execution.rewiring_apply_count,
        "dependency_capture_updates": after.execution.dependency_capture_updates - before.execution.dependency_capture_updates,
        "dependency_reconcile_nanos": after.execution.dependency_reconcile_nanos - before.execution.dependency_reconcile_nanos,
        "dependency_input_build_nanos": after.execution.dependency_input_build_nanos - before.execution.dependency_input_build_nanos,
        "dependency_input_shape_handle_lookup_nanos": after.execution.dependency_input_shape_handle_lookup_nanos - before.execution.dependency_input_shape_handle_lookup_nanos,
        "dependency_input_previous_snapshot_fetch_nanos": after.execution.dependency_input_previous_snapshot_fetch_nanos - before.execution.dependency_input_previous_snapshot_fetch_nanos,
        "dependency_input_version_scan_nanos": after.execution.dependency_input_version_scan_nanos - before.execution.dependency_input_version_scan_nanos,
        "dependency_input_stable_proof_nanos": after.execution.dependency_input_stable_proof_nanos - before.execution.dependency_input_stable_proof_nanos,
        "dependency_input_version_delta_nanos": after.execution.dependency_input_version_delta_nanos - before.execution.dependency_input_version_delta_nanos,
        "dependency_input_replacement_build_nanos": after.execution.dependency_input_replacement_build_nanos - before.execution.dependency_input_replacement_build_nanos,
        "dependency_input_stable_shape_count": after.execution.dependency_input_stable_shape_count - before.execution.dependency_input_stable_shape_count,
        "dependency_input_replacement_count": after.execution.dependency_input_replacement_count - before.execution.dependency_input_replacement_count,
        "deferred_snapshot_packet_nanos": after.execution.deferred_snapshot_packet_nanos - before.execution.deferred_snapshot_packet_nanos,
        "graph_storage_compaction_count": after.storage.graph_storage_compaction_count - before.storage.graph_storage_compaction_count,
        "dependency_segments_rewritten": after.storage.graph_storage_dependency_segments_rewritten - before.storage.graph_storage_dependency_segments_rewritten,
        "subscriber_segments_rewritten": after.storage.graph_storage_subscriber_segments_rewritten - before.storage.graph_storage_subscriber_segments_rewritten,
        "snapshot_batch_commit_nanos": after.storage.snapshot_batch_commit_nanos - before.storage.snapshot_batch_commit_nanos,
    })
}

pub(super) fn perf_contract<'a>(
    suite: &'a str,
    profile: &'a str,
    timing_policy: PerfTimingPolicy,
    phase_metrics: &'a [&'a str],
) -> PerfCaseContract<'a> {
    PerfCaseContract::new(
        suite,
        profile,
        "serial",
        timing_policy,
        phase_metrics,
        &[],
        &[],
    )
}

pub(super) fn hot_family_contract<'a>(
    suite: &'a str,
    profile: &'a str,
    timing_policy: PerfTimingPolicy,
    phase_metrics: &'a [&'a str],
    access_counter_maxima: &'a [(&'a str, u128)],
) -> PerfCaseContract<'a> {
    PerfCaseContract::new(
        suite,
        profile,
        "serial",
        timing_policy,
        phase_metrics,
        &[],
        access_counter_maxima,
    )
}

pub(super) fn hot_family_contract_with_scoped_allocations<'a>(
    suite: &'a str,
    profile: &'a str,
    timing_policy: PerfTimingPolicy,
    phase_metrics: &'a [&'a str],
    scoped_allocation_metrics: &'a [&'a str],
    access_counter_maxima: &'a [(&'a str, u128)],
) -> PerfCaseContract<'a> {
    PerfCaseContract::new(
        suite,
        profile,
        "serial",
        timing_policy,
        phase_metrics,
        scoped_allocation_metrics,
        access_counter_maxima,
    )
}

pub(super) const ZERO_BROAD_ENTRY_ACCESS: &[(&str, u128)] = &[
    ("materialized_entry_reads", 0),
    ("materialized_entry_writes", 0),
];

pub(super) const ZERO_BROAD_AND_ARTIFACT_ACCESS: &[(&str, u128)] = &[
    ("materialized_entry_reads", 0),
    ("materialized_entry_writes", 0),
    ("runtime_artifact_state_reads", 0),
    ("runtime_artifact_warm_reads", 0),
];

pub(super) fn policy_for(profile_name: &str) -> SignalRuntimePolicy {
    match profile_name {
        "operational" => SignalRuntimePolicy::operational().with_history_limit(4),
        "development" => SignalRuntimePolicy::development().with_history_limit(6),
        "forensic" => SignalRuntimePolicy::forensic().with_history_limit(8),
        other => panic!("unexpected profile for perf test: {other}"),
    }
}
