import type {
  DiagnosticsCounterMap,
  DiagnosticsTier,
  DiagnosticsVariant,
  NodeIdSummary,
} from "./policy.js";
import type {
  CallbackRuntimeNodeSummary,
} from "./callback.js";
import type {
  ObservationBoundarySummary,
} from "./observation.js";
import type {
  ApplySummary,
  ChangeInputSummary,
  InvalidationSummary,
  PlanningSummary,
  PrecomputeSummary,
  RollbackSummary,
} from "./execution.js";
import type {
  EventEpochSummary,
} from "./event.js";

export interface FlowCauseSample {
  node: NodeIdSummary;
  cause_kinds: ReadonlyArray<string>;
  scope_kinds: ReadonlyArray<string>;
  scope_notes: ReadonlyArray<string>;
  suspect_classes: ReadonlyArray<string>;
  rewired: boolean;
  conservative_recompute: boolean;
}

export interface ReuseBasisSummary {
  strategy?: string | null;
  source: DiagnosticsVariant;
  crossing: DiagnosticsVariant;
  dependency_snapshot_basis?: number | string | null;
  topology_regime_basis?: number | null;
  structural_dependency_basis?: number | string | null;
  artifact_family_basis?: string | null;
  partition_region_basis_count: number;
}

export interface ExplanationSummary {
  profile: DiagnosticsTier;
  node: NodeIdSummary;
  materialization_mode: DiagnosticsVariant;
  state: DiagnosticsVariant;
  dirty_aspect_count: number;
  upstream_count: number;
  changed_upstream_count: number;
  skipped_upstream_count: number;
  condition_deferred_count: number;
  clean_upstream_count: number;
  missing_snapshot_count: number;
  dependency_removed_count: number;
  conservative_cause_count: number;
  direct_scope_count: number;
  translated_scope_count: number;
  discarded_scope_count: number;
  insufficient_scope_count: number;
  rewired_dependency_count: number;
  direct_cause_kinds: ReadonlyArray<DiagnosticsVariant>;
  scope_provenance_kinds: ReadonlyArray<string>;
  cause_note_samples: ReadonlyArray<string>;
  triage_classes: ReadonlyArray<string>;
  propagation_suppressed: boolean;
  contract_reads_mask: number | string;
  contract_produces_mask: number | string;
  contract_partition_scope_count: number;
  required_context: DiagnosticsVariant;
  execution_record_id: number | null;
  semantic_segment_id: number | null;
  output_change: string | null;
  memoized_origin: string | null;
  reuse_basis: ReuseBasisSummary | null;
  reuse_origin: string | null;
  reuse_certification_proof_count: number;
  changed_region_count: number;
  causality_kind: string | null;
}

export interface FlowSummary {
  profile: DiagnosticsTier;
  change: ChangeInputSummary;
  invalidation: InvalidationSummary;
  planning: PlanningSummary;
  precompute: PrecomputeSummary;
  apply: ApplySummary;
  cause_samples: ReadonlyArray<FlowCauseSample>;
  event_epochs: ReadonlyArray<EventEpochSummary>;
  observation: ObservationBoundarySummary | null;
  rollback: RollbackSummary | null;
  explanation: ExplanationSummary | null;
}

export interface FlowSurfaceSummary {
  flow: FlowSummary;
  callbackNodes: ReadonlyArray<CallbackRuntimeNodeSummary>;
}

export interface ObservationSurfaceSummary {
  observation: ObservationBoundarySummary;
  callbackNodes: ReadonlyArray<CallbackRuntimeNodeSummary>;
}

export interface ExecutionHistoryNodeSummary {
  node: NodeIdSummary;
  execution_record_id: number | null;
  semantic_segment_id: number | null;
  output_change: string | null;
  memoized_origin: string | null;
  reuse_basis: ReuseBasisSummary | null;
  reuse_origin: string | null;
  persistent_correspondence_kind: string | null;
  composition_region_count: number;
  reuse_certification_proof_count: number;
  changed_partition_count: number;
  causality_kind: string | null;
}

export interface ExecutionHistorySummary {
  profile: DiagnosticsTier;
  traced_node_count: number;
  execution_record_count: number;
  latest_execution_record_id: number | null;
  reuse_origin_counts: DiagnosticsCounterMap;
  nodes: ReadonlyArray<ExecutionHistoryNodeSummary>;
}

export interface ExecutionHistorySurfaceSummary {
  history: ExecutionHistorySummary;
  callbackNodes: ReadonlyArray<CallbackRuntimeNodeSummary>;
}
