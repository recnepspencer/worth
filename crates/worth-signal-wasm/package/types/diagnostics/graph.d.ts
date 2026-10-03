import type {
  DiagnosticsTier,
  NodeIdSummary,
} from "./policy.js";

export interface TouchedScopeSummary {
  seed_scopes: ReadonlyArray<PartitionSubscriptionSummary>;
  inclusion_scopes: ReadonlyArray<PartitionSubscriptionSummary>;
  transitive_reached_scopes: ReadonlyArray<PartitionSubscriptionSummary>;
  direct_dirty_scopes: ReadonlyArray<PartitionSubscriptionSummary>;
  maybe_stale_scopes: ReadonlyArray<PartitionSubscriptionSummary>;
  touched_nodes: ReadonlyArray<NodeIdSummary>;
  touched_sources: ReadonlyArray<NodeIdSummary>;
}

export interface PartitionSubscriptionSummary {
  partition: string;
  detail?: string | null;
  match_mode: "WholePartition" | "PartitionAndDetail" | string;
}

export interface FrontierExecutionCounters {
  frontier_seed_count: number;
  frontier_group_count: number;
  frontier_direct_wave_count: number;
  frontier_transitive_wave_count: number;
  frontier_partition_scoped_check_count: number;
  frontier_direct_dirty_count: number;
  frontier_maybe_stale_count: number;
  frontier_partition_match_count: number;
  frontier_detail_match_count: number;
  frontier_cycle_check_candidate_count: number;
  frontier_cycle_check_visited_count: number;
  frontier_trace_retained_count: number;
}

export interface FrontierWaveEntrySummary {
  node: NodeIdSummary;
  classification: string;
  inclusion_basis: string;
  narrowed_scopes: ReadonlyArray<PartitionSubscriptionSummary>;
}

export interface FrontierWaveSummary {
  wave_index: number;
  aspect: number;
  entries: ReadonlyArray<FrontierWaveEntrySummary>;
}

export interface InvalidationPlanningEstimate {
  seed_count: number;
  group_count: number;
  direct_wave_count: number;
  transitive_wave_count: number;
  direct_dirty_count: number;
  maybe_stale_count: number;
  partition_scoped_checks: number;
  partition_match_count: number;
  detail_match_count: number;
  cycle_check_candidate_count: number;
}

export interface InvalidationTraceRecord {
  node: NodeIdSummary;
  aspect: number;
  wave_index: number;
  classification: string;
  inclusion_basis: string;
}

export interface GraphSummary {
  profile: DiagnosticsTier;
  active_node_count: number;
  arena_capacity: number;
  tombstone_count: number;
  clean_node_count: number;
  maybe_stale_node_count: number;
  dirty_node_count: number;
  dependency_edge_count: number;
  subscriber_edge_count: number;
  nodes_with_partition_scopes: number;
  nodes_with_trace_summary: number;
  nodes_with_execution_record: number;
  nodes_with_causality: number;
  partition_interner_size: number;
  sample_dirty_nodes: ReadonlyArray<NodeIdSummary>;
  sample_nodes_with_execution_record: ReadonlyArray<NodeIdSummary>;
  metrics: Readonly<Record<string, unknown>>;
}
