//! Operational cloning reconstructs charge facts for copied diagnostic roots.
use super::DiagnosticsState;

impl Clone for DiagnosticsState {
    fn clone(&self) -> Self {
        // Exclusive persistent maps clone payloads and discard cached charges:
        // owned Vec capacities may change. Repair the selected writer roots at
        // this actual reconstruction boundary, never on persistent forks or
        // ordinary lineage appends.
        let mut cloned = Self {
            request_mirror: self.request_mirror,
            installed_retention_budget: self.installed_retention_budget,
            installed_tier: self.installed_tier,
            installed_frontier_tracing_policy: self.installed_frontier_tracing_policy,
            latest_flow: self.latest_flow.clone(),
            latest_failure: self.latest_failure.clone(),
            latest_rollback: self.latest_rollback.clone(),
            latest_observation: self.latest_observation.clone(),
            latest_graph_summary: self.latest_graph_summary.clone(),
            pending_graph_summary: self.pending_graph_summary.clone(),
            recent_history: self.recent_history.clone(),
            replay_events: self.replay_events.clone(),
            lineage_records: self.lineage_records.clone(),
            replay_events_by_branch: self.replay_events_by_branch.clone(),
            replay_events_by_node: self.replay_events_by_node.clone(),
            replay_events_by_artifact: self.replay_events_by_artifact.clone(),
            replay_cursor_offsets: self.replay_cursor_offsets.clone(),
            replay_cursor_offset_base: self.replay_cursor_offset_base,
            snapshot_replay_cursors: self.snapshot_replay_cursors.clone(),
            lineage_records_by_artifact: self.lineage_records_by_artifact.clone(),
            lineage_records_by_node: self.lineage_records_by_node.clone(),
            explanation_facts: self.explanation_facts.clone(),
            provenance_facts: self.provenance_facts.clone(),
            branch_catalog: self.branch_catalog.clone(),
            active_branch: self.active_branch,
            next_replay_cursor: self.next_replay_cursor,
            next_snapshot_id: self.next_snapshot_id,
            next_branch_id: self.next_branch_id,
            next_lineage_artifact_id: self.next_lineage_artifact_id,
            next_lineage_sequence: self.next_lineage_sequence,
            pending_input: self.pending_input.clone(),
            latest_frontier_execution: self.latest_frontier_execution.clone(),
            latest_invalidation_planning_estimate: self
                .latest_invalidation_planning_estimate
                .clone(),
            latest_invalidation_trace_records: self.latest_invalidation_trace_records.clone(),
            observation_activation_mask: self.observation_activation_mask,
            lineage_custody: self.lineage_custody.clone(),
            fact_custody: self.fact_custody.clone(),
            transaction_flow_scope: self.transaction_flow_scope,
        };
        cloned.reconstitute_selected_epoch_charge();
        cloned
    }
}
