use std::collections::BTreeMap;

use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::DedupedNodeBatch;
use crate::data::temporal::TemporalExecutionSummary;
use crate::logic::planner::ExecutionReport;

pub(super) fn collect_dirty_targets(graph: &SignalGraph) -> Vec<NodeId> {
    DedupedNodeBatch::canonicalize_unordered(graph.live_node_ids().into_iter().filter(|node| {
        graph
            .get_state(*node)
            .is_ok_and(|state| !matches!(state, crate::data::node::NodeState::Clean))
    }))
    .into_vec()
}

pub(super) fn empty_execution_report() -> ExecutionReport {
    ExecutionReport {
        execution: Vec::new(),
        plan_summary: crate::logic::planner::PlanSummary::default(),
        stage_count: 0,
        task_count: 0,
        maybe_stale_validation_tasks: 0,
        latest_execution_record_id: None,
        temporal_summary: TemporalExecutionSummary::default(),
        reuse_origin_counts: BTreeMap::new(),
        tasks_executed: 0,
        tasks_pruned: 0,
        tasks_validated_clean: 0,
        tasks_deferred_by_condition: 0,
        tasks_reverted_clean_by_condition: 0,
        tasks_satisfied_by_memoization: 0,
        tasks_with_suppressed_propagation: 0,
        execution_snapshots_built: 0,
        prepared_evaluations_produced: 0,
        prepared_evaluations_applied: 0,
        dependency_capture_updates: 0,
        execution_snapshot_nanos: 0,
        stage_precompute_nanos: 0,
        stage_apply_nanos: 0,
        semantic_finalize_nanos: 0,
        semantic_segment_count: 0,
        stages: Vec::new(),
    }
}
