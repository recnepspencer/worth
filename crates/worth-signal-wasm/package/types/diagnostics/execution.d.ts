import type {
  DiagnosticsCounterMap,
  DiagnosticsTier,
  DiagnosticsVariant,
  NodeIdSummary,
} from "./policy.js";
import type {
  EventEpochSummary,
} from "./event.js";

export interface ChangeInputSummary {
  changed_nodes: ReadonlyArray<NodeIdSummary>;
  changed_aspects: ReadonlyArray<number>;
  changed_region_count: number;
  causality_kind: string | null;
}

export interface InvalidationSummary {
  invalidated_direct_subscribers: number;
  maybe_stale_direct_subscribers: number;
  partition_scoped_checks: number;
  narrowed_frontier_width: number;
  transitive_frontier_width: number;
  frontier_seed_count: number;
  frontier_group_count: number;
  frontier_direct_wave_count: number;
  frontier_transitive_wave_count: number;
  frontier_partition_match_count: number;
  frontier_detail_match_count: number;
  frontier_cycle_check_candidate_count: number;
  frontier_cycle_check_visited_count: number;
  frontier_trace_retained_count: number;
}

export interface EvaluationPlanSummary {
  profile: DiagnosticsTier;
  requested_target_count: number;
  stage_count: number;
  task_count: number;
  max_stage_width: number;
  contract_pruned_count: number;
  stage_widths: ReadonlyArray<number>;
  direct_request_count: number;
  transitive_task_count: number;
  task_reason_counts: DiagnosticsCounterMap;
}

export interface PlanningSummary {
  plan: EvaluationPlanSummary;
}

export interface TemporalExecutionSummary {
  ready_count: number;
  deferred_count: number;
  runtime_clock_authority_count: number;
  resolver_fallback_count: number;
  runtime_scheduled_wake_count: number;
}

export interface ExecutionReportSummary {
  profile: DiagnosticsTier;
  stage_count: number;
  task_count: number;
  tasks_executed: number;
  tasks_pruned: number;
  tasks_validated_clean: number;
  tasks_deferred_by_condition: number;
  tasks_reverted_clean_by_condition: number;
  tasks_satisfied_by_memoization: number;
  tasks_with_suppressed_propagation: number;
  prepared_evaluations_produced: number;
  prepared_evaluations_applied: number;
  dependency_capture_updates: number;
  semantic_segment_count: number;
  temporal_summary: TemporalExecutionSummary;
  task_outcome_counts: DiagnosticsCounterMap;
  stage_outcome_counts: DiagnosticsCounterMap;
}

export interface PrecomputeSummary {
  executor: DiagnosticsVariant | null;
  stage_count: number;
  task_count: number;
  prepared_evaluations_produced: number;
  tasks_deferred_by_condition: number;
  tasks_satisfied_by_memoization: number;
}

export interface ApplySummary {
  report: ExecutionReportSummary;
  prepared_evaluations_applied: number;
  dependency_capture_updates: number;
  tasks_validated_clean: number;
  tasks_pruned: number;
  tasks_with_suppressed_propagation: number;
}

export interface RollbackSummary {
  rolled_back: boolean;
  staged_node_patch_count: number;
  max_touched_nodes_in_txn: number;
  reason: string | null;
}

export interface FailureSummary {
  profile: DiagnosticsTier;
  phase: string;
  stage_index: number | null;
  node: NodeIdSummary | null;
  executor: DiagnosticsVariant | null;
  execution_record_id: number | null;
  has_plan_summary: boolean;
  rolled_back: boolean;
  staged_node_patch_count: number | null;
  max_touched_nodes_in_txn: number | null;
  event_epochs: ReadonlyArray<EventEpochSummary>;
  message: string;
}

export interface RollbackDiagnostic {
  rolled_back: boolean;
  staged_node_patch_count: number;
  max_touched_nodes_in_txn: number;
  reason: string | null;
  event_epochs: ReadonlyArray<EventEpochSummary>;
}
