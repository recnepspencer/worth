import type {
  NodeIdSummary,
} from "./policy.js";

export interface ObservationPolicySummary {
  schema_version: "worth.signal.observation-policy.v2";
  trigger: "Visited" | "Recomputed" | "MeaningfulChange" | string;
  delivery_mode: "PerCommittedTransaction" | string;
}

export interface ObservedNodeSetSummary {
  nodes: ReadonlyArray<NodeIdSummary>;
}

export interface CommittedObservationEventSummary {
  observer_id: number;
  handle_id: number;
  policy: ObservationPolicySummary;
  observed_nodes: ObservedNodeSetSummary;
  matched_nodes: ObservedNodeSetSummary;
  visited: boolean;
  recomputed: boolean;
  meaningful_change: boolean;
  trigger_matched: boolean;
  outcome: "Delivered" | "RollbackSuppressed" | string;
}

export interface ObservationBoundarySummary {
  classified_event_count: number;
  trigger_matched_event_count: number;
  delivered_event_count: number;
  rollback_suppressed_event_count: number;
  boundary_events: ReadonlyArray<CommittedObservationEventSummary>;
}
