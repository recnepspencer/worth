//! Full diagnostic root custody for a private mutable evaluation draft.
use super::{DiagnosticsState, PendingFlowInput};
use crate::data::persistent_ord_map::PersistentOrdMap;
use crate::data::retained_storage::SignalConditionalRetentionReservation as Reservation;

impl DiagnosticsState {
    /// Shares accumulated evidence and prepares exclusive indexes for later edits.
    /// This grants neither admission nor a retained-byte reservation.
    pub(crate) fn fork_persistent(&mut self) -> Self {
        self.fork_with_resources(None)
    }

    /// Conversion custody travels with each backing shared by both roots.
    pub(crate) fn fork_reserved(&mut self, resources: &mut Reservation) -> Self {
        self.fork_with_resources(Some(resources))
    }

    fn fork_with_resources(&mut self, mut resources: Option<&mut Reservation>) -> Self {
        let Self {
            request_mirror,
            installed_retention_budget,
            installed_tier,
            installed_frontier_tracing_policy,
            latest_flow,
            latest_failure,
            latest_rollback,
            latest_observation,
            latest_graph_summary,
            pending_graph_summary,
            recent_history,
            replay_events,
            lineage_records,
            replay_events_by_branch,
            replay_events_by_node,
            replay_events_by_artifact,
            replay_cursor_offsets,
            replay_cursor_offset_base,
            snapshot_replay_cursors,
            lineage_records_by_artifact,
            lineage_records_by_node,
            explanation_facts,
            provenance_facts,
            branch_catalog,
            active_branch,
            next_replay_cursor,
            next_snapshot_id,
            next_branch_id,
            next_lineage_artifact_id,
            next_lineage_sequence,
            pending_input,
            latest_frontier_execution,
            latest_invalidation_planning_estimate,
            latest_invalidation_trace_records,
            observation_activation_mask,
            lineage_custody,
        } = self;
        Self {
            request_mirror: *request_mirror,
            installed_retention_budget: *installed_retention_budget,
            installed_tier: *installed_tier,
            installed_frontier_tracing_policy: *installed_frontier_tracing_policy,
            latest_flow: latest_flow.clone(),
            latest_failure: latest_failure.clone(),
            latest_rollback: latest_rollback.clone(),
            latest_observation: latest_observation.clone(),
            latest_graph_summary: latest_graph_summary.clone(),
            pending_graph_summary: pending_graph_summary.clone(),
            recent_history: recent_history.clone(),
            replay_events: replay_events.clone(),
            lineage_records: lineage_records.clone(),
            replay_events_by_branch: fork_index(replay_events_by_branch, resources.as_deref_mut()),
            replay_events_by_node: fork_index(replay_events_by_node, resources.as_deref_mut()),
            replay_events_by_artifact: fork_index(
                replay_events_by_artifact,
                resources.as_deref_mut(),
            ),
            replay_cursor_offsets: fork_index(replay_cursor_offsets, resources.as_deref_mut()),
            replay_cursor_offset_base: *replay_cursor_offset_base,
            snapshot_replay_cursors: fork_index(snapshot_replay_cursors, resources.as_deref_mut()),
            lineage_records_by_artifact: fork_index(
                lineage_records_by_artifact,
                resources.as_deref_mut(),
            ),
            lineage_records_by_node: fork_index(lineage_records_by_node, resources.as_deref_mut()),
            explanation_facts: fork_index(explanation_facts, resources.as_deref_mut()),
            provenance_facts: fork_index(provenance_facts, resources.as_deref_mut()),
            branch_catalog: fork_index(branch_catalog, resources.as_deref_mut()),
            active_branch: *active_branch,
            next_replay_cursor: *next_replay_cursor,
            next_snapshot_id: *next_snapshot_id,
            next_branch_id: *next_branch_id,
            next_lineage_artifact_id: *next_lineage_artifact_id,
            next_lineage_sequence: *next_lineage_sequence,
            pending_input: pending_input
                .as_mut()
                .map(|input| input.fork_with_resources(resources.as_deref_mut())),
            latest_frontier_execution: latest_frontier_execution.clone(),
            latest_invalidation_planning_estimate: latest_invalidation_planning_estimate.clone(),
            latest_invalidation_trace_records: latest_invalidation_trace_records.clone(),
            observation_activation_mask: *observation_activation_mask,
            lineage_custody: lineage_custody.clone(),
        }
    }
}

impl PendingFlowInput {
    fn fork_with_resources(&mut self, mut resources: Option<&mut Reservation>) -> Self {
        let Self {
            changed_nodes,
            changed_aspects,
            changed_region_count,
            causality_kind,
        } = self;
        Self {
            changed_nodes: match resources.as_deref_mut() {
                Some(resources) => changed_nodes.fork_reserved(resources),
                None => changed_nodes.fork_persistent(),
            },
            changed_aspects: match resources.as_deref_mut() {
                Some(resources) => changed_aspects.fork_reserved(resources),
                None => changed_aspects.fork_persistent(),
            },
            changed_region_count: *changed_region_count,
            causality_kind: causality_kind.clone(),
        }
    }
}

#[cfg(test)]
#[path = "fork_tests.rs"]
mod tests;

fn fork_index<K: Clone + Ord, V: Clone>(
    index: &mut PersistentOrdMap<K, V>,
    resources: Option<&mut Reservation>,
) -> PersistentOrdMap<K, V> {
    match resources {
        Some(resources) => index.fork_reserved(resources),
        None => index.fork_persistent(),
    }
}
