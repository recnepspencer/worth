//! Explicit cold preparation of every retained diagnostic root.
use super::{DiagnosticHistory, DiagnosticsState, PendingFlowInput};
use crate::data::persistent_ord_map::PersistentOrdMap;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl DiagnosticsState {
    /// Returns heap charge only; the enclosing slot owns inline storage charges.
    /// Denial may leave child accounting prepared, but changes no semantic state.
    /// This does not carry an aggregate charge or reserve mutation capacity.
    /// Effective index histories receive edit facts without copying map entries.
    /// Ordinary execution must not use this to repair accounting after mutation.
    pub(crate) fn prepare_retained_heap_charge(
        &mut self,
        work: &mut Work,
    ) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            request_mirror: _,
            installed_retention_budget: _,
            installed_tier: _,
            installed_frontier_tracing_policy: _,
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
            replay_cursor_offset_base: _,
            snapshot_replay_cursors,
            lineage_records_by_artifact,
            lineage_records_by_node,
            explanation_facts,
            provenance_facts,
            branch_catalog,
            active_branch: _,
            next_replay_cursor: _,
            next_snapshot_id: _,
            next_branch_id: _,
            next_lineage_artifact_id: _,
            next_lineage_sequence: _,
            pending_input,
            latest_frontier_execution,
            latest_invalidation_planning_estimate,
            latest_invalidation_trace_records,
            observation_activation_mask: _,
            lineage_custody,
        } = self;
        prepare_index_histories(replay_events_by_branch, work)?;
        prepare_index_histories(replay_events_by_node, work)?;
        prepare_index_histories(replay_events_by_artifact, work)?;
        prepare_index_histories(lineage_records_by_artifact, work)?;
        prepare_index_histories(lineage_records_by_node, work)?;
        let mut charge = lineage_custody.retained_heap_charge(work)?;
        charge = charge.checked_add(latest_flow.retained_heap_charge(work)?)?;
        charge = charge.checked_add(latest_failure.retained_heap_charge(work)?)?;
        charge = charge.checked_add(latest_rollback.retained_heap_charge(work)?)?;
        charge = charge.checked_add(latest_observation.retained_heap_charge(work)?)?;
        charge = charge.checked_add(latest_graph_summary.retained_heap_charge(work)?)?;
        charge = charge.checked_add(pending_graph_summary.retained_heap_charge(work)?)?;
        charge = charge.checked_add(recent_history.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(replay_events.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(lineage_records.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(replay_events_by_branch.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(replay_events_by_node.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(replay_events_by_artifact.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(replay_cursor_offsets.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(snapshot_replay_cursors.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(lineage_records_by_artifact.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(lineage_records_by_node.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(explanation_facts.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(provenance_facts.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(branch_catalog.prepare_retained_charge(work)?)?;
        charge = charge.checked_add(match pending_input {
            Some(input) => input.prepare_retained_heap_charge(work)?,
            None => Charge::ZERO,
        })?;
        charge = charge.checked_add(latest_frontier_execution.retained_heap_charge(work)?)?;
        charge = charge
            .checked_add(latest_invalidation_planning_estimate.retained_heap_charge(work)?)?;
        charge =
            charge.checked_add(latest_invalidation_trace_records.retained_heap_charge(work)?)?;
        Ok(charge)
    }
}
fn prepare_index_histories<K: Clone + Ord, T: RetainedStorageMeasurement>(
    index: &PersistentOrdMap<K, DiagnosticHistory<T>>,
    work: &mut Work,
) -> Result<(), Denial> {
    for history in index.values() {
        work.visit()?;
        history.prepare_retained_charge(work)?;
    }
    Ok(())
}
impl PendingFlowInput {
    fn prepare_retained_heap_charge(&mut self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            changed_nodes,
            changed_aspects,
            changed_region_count: _,
            causality_kind,
        } = self;
        changed_nodes
            .prepare_retained_charge(work)?
            .checked_add(changed_aspects.prepare_retained_charge(work)?)?
            .checked_add(causality_kind.retained_heap_charge(work)?)
    }
}
#[cfg(test)]
mod tests;
