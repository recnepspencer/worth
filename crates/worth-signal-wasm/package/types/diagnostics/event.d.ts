export interface EventSubscriberOutcome {
  subscriber_name: string;
  outcome: "Committed" | "Failed" | string;
  requires_data_ids: ReadonlyArray<string>;
  provides_data_ids: ReadonlyArray<string>;
  staged_data_ids: ReadonlyArray<string>;
}

export interface EventEpochSummary {
  ordinal: number;
  barrier: "PerMutation" | "PerOperation" | "PerCommit" | "OnDemandRead" | string;
  emitted_event_count: number;
  subscriber_count: number;
  committed_subscriber_count: number;
  failed_subscriber_position: number | null;
  subscriber_outcomes: ReadonlyArray<EventSubscriberOutcome>;
  outcome: "Committed" | "RolledBack" | "Failed" | string;
  failure_subscriber: string | null;
  message: string | null;
}
