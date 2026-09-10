//! Bounded measurement of an immutable diagnostic authority carrier only.
use super::DiagnosticHistory;
use super::DiagnosticsState;
use crate::data::persistent_ord_map::PersistentOrdMap;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial,
};
#[cfg(test)]
use crate::state::SignalBranchId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BranchCarrierChargeDenial {
    NotBranchCarrier,
    Storage(RetainedStoragePreparationDenial),
}
impl From<RetainedStoragePreparationDenial> for BranchCarrierChargeDenial {
    fn from(value: RetainedStoragePreparationDenial) -> Self {
        Self::Storage(value)
    }
}

impl DiagnosticsState {
    /// Construction cost of fork_branch_carrier, independent of source history.
    /// This creates no temporary carrier, catalog, history root or name string.
    pub(crate) fn prepare_branch_carrier_charge(
        &self,
        work: &mut Preparation,
    ) -> Result<Charge, RetainedStoragePreparationDenial> {
        work.visit()?;
        use crate::data::retained_storage::{arc_allocation_charge, btree_structure_charge};
        arc_allocation_charge::<Vec<crate::data::proof::InvalidationTraceRecord>>()?
            .checked_add(fresh_history_charge(&self.recent_history, work)?)?
            .checked_add(fresh_history_charge(&self.replay_events, work)?)?
            .checked_add(fresh_history_charge(&self.lineage_records, work)?)?
            .checked_add(fresh_map_charge(&self.replay_events_by_branch, work)?)?
            .checked_add(fresh_map_charge(&self.replay_events_by_node, work)?)?
            .checked_add(fresh_map_charge(&self.replay_events_by_artifact, work)?)?
            .checked_add(fresh_map_charge(&self.replay_cursor_offsets, work)?)?
            .checked_add(fresh_map_charge(&self.snapshot_replay_cursors, work)?)?
            .checked_add(fresh_map_charge(&self.lineage_records_by_artifact, work)?)?
            .checked_add(fresh_map_charge(&self.lineage_records_by_node, work)?)?
            .checked_add(fresh_map_charge(&self.explanation_facts, work)?)?
            .checked_add(fresh_map_charge(&self.provenance_facts, work)?)?
            .checked_add(btree_structure_charge::<
                crate::state::SignalBranchId,
                crate::state::SignalBranchHandle,
            >(1)?)?
            .checked_add(Charge::capacity::<u8>(
                super::branching::BOOTSTRAP_BRANCH_NAME.len(),
            )?)
    }

    /// This is deliberately not a measurement of arbitrary mutable diagnostics.
    /// History-bearing state must use its owning growth accounting.
    pub(crate) fn retained_branch_carrier_charge(
        &self,
        work: &mut Preparation,
    ) -> Result<Charge, BranchCarrierChargeDenial> {
        work.visit()?;
        let Self {
            latest_flow: None,
            latest_failure: None,
            latest_rollback: None,
            latest_observation: None,
            latest_graph_summary: None,
            pending_graph_summary: None,
            pending_input: None,
            latest_frontier_execution: None,
            latest_invalidation_planning_estimate: None,
            request_mirror: _,
            installed_retention_budget: _,
            installed_tier: _,
            installed_frontier_tracing_policy: _,
            replay_cursor_offset_base: _,
            active_branch: _,
            next_replay_cursor: _,
            next_snapshot_id: _,
            next_branch_id: _,
            next_lineage_artifact_id: _,
            next_lineage_sequence: _,
            observation_activation_mask: _,
            lineage_custody,
            replay_events_by_branch,
            replay_events_by_node,
            replay_events_by_artifact,
            replay_cursor_offsets,
            snapshot_replay_cursors,
            lineage_records_by_artifact,
            lineage_records_by_node,
            explanation_facts,
            provenance_facts,
            recent_history,
            replay_events,
            lineage_records,
            branch_catalog,
            latest_invalidation_trace_records,
        } = self
        else {
            return Err(BranchCarrierChargeDenial::NotBranchCarrier);
        };
        if !latest_invalidation_trace_records.is_empty()
            || latest_invalidation_trace_records.capacity() != 0
        {
            return Err(BranchCarrierChargeDenial::NotBranchCarrier);
        }
        work.visit()?;
        let mut charge = crate::data::retained_storage::arc_allocation_charge::<
            Vec<crate::data::proof::InvalidationTraceRecord>,
        >()?;
        charge = charge.checked_add(empty_history_charge(recent_history, work)?)?;
        charge = charge.checked_add(empty_history_charge(replay_events, work)?)?;
        charge = charge.checked_add(empty_history_charge(lineage_records, work)?)?;
        charge = charge.checked_add(empty_map_charge(replay_events_by_branch, work)?)?;
        charge = charge.checked_add(empty_map_charge(replay_events_by_node, work)?)?;
        charge = charge.checked_add(empty_map_charge(replay_events_by_artifact, work)?)?;
        charge = charge.checked_add(empty_map_charge(replay_cursor_offsets, work)?)?;
        charge = charge.checked_add(empty_map_charge(snapshot_replay_cursors, work)?)?;
        charge = charge.checked_add(empty_map_charge(lineage_records_by_artifact, work)?)?;
        charge = charge.checked_add(empty_map_charge(lineage_records_by_node, work)?)?;
        charge = charge.checked_add(empty_map_charge(explanation_facts, work)?)?;
        charge = charge.checked_add(empty_map_charge(provenance_facts, work)?)?;
        charge = charge.checked_add(branch_catalog.retained_heap_charge(work)?)?;
        charge = charge.checked_add(lineage_custody.retained_heap_charge(work)?)?;
        Ok(charge)
    }
}

fn fresh_history_charge<T>(
    _: &DiagnosticHistory<T>,
    work: &mut Preparation,
) -> Result<Charge, RetainedStoragePreparationDenial> {
    work.visit()?;
    crate::data::retained_storage::ordered_index_charge::<u64, std::sync::Arc<T>>(0)
}

fn fresh_map_charge<K: Clone + Ord, V: Clone>(
    _: &PersistentOrdMap<K, V>,
    work: &mut Preparation,
) -> Result<Charge, RetainedStoragePreparationDenial> {
    work.visit()?;
    crate::data::retained_storage::btree_structure_charge::<K, V>(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carrier_charge_bounds_catalog_capacity_and_rejects_retained_history_storage() {
        let source = DiagnosticsState::default();
        let mut carrier = source.fork_branch_carrier();
        let before = carrier
            .retained_branch_carrier_charge(&mut Preparation::new(1_000))
            .unwrap();
        let branch = carrier.branch_catalog.get_mut(&SignalBranchId(0)).unwrap();
        let original_capacity = branch.name.capacity();
        branch.name.reserve_exact(8_192);
        let additional_capacity = branch.name.capacity() - original_capacity;
        let mut work = Preparation::new(1_000);
        let after = carrier.retained_branch_carrier_charge(&mut work).unwrap();
        assert_eq!(after.bytes() - before.bytes(), additional_capacity as u64);
        assert_eq!(
            carrier
                .retained_branch_carrier_charge(&mut Preparation::new(work.visits()))
                .unwrap(),
            after
        );
        assert!(matches!(
            carrier.retained_branch_carrier_charge(&mut Preparation::new(work.visits() - 1)),
            Err(BranchCarrierChargeDenial::Storage(
                RetainedStoragePreparationDenial::WorkExhausted { .. }
            ))
        ));
        carrier
            .recent_history
            .push_back(crate::diagnostics::summary::ExecutionHistorySummary {
                profile: crate::diagnostics::profile::DiagnosticsTier::Development,
                traced_node_count: 0,
                execution_record_count: 0,
                latest_execution_record_id: None,
                reuse_origin_counts: Default::default(),
                nodes: Vec::new(),
            })
            .unwrap();
        assert!(!carrier.recent_history.is_empty());
        assert_eq!(
            carrier.retained_branch_carrier_charge(&mut Preparation::new(1_000)),
            Err(BranchCarrierChargeDenial::NotBranchCarrier)
        );
        let mut carrier = source.fork_branch_carrier();
        carrier.pending_input = Some(super::super::PendingFlowInput {
            changed_nodes: Default::default(),
            changed_aspects: Default::default(),
            changed_region_count: 0,
            causality_kind: None,
        });
        assert_eq!(
            carrier.retained_branch_carrier_charge(&mut Preparation::new(1_000)),
            Err(BranchCarrierChargeDenial::NotBranchCarrier)
        );
    }
}

fn empty_history_charge<T>(
    values: &DiagnosticHistory<T>,
    work: &mut Preparation,
) -> Result<Charge, BranchCarrierChargeDenial> {
    work.visit()?;
    if values.is_empty() {
        Ok(crate::data::retained_storage::ordered_index_charge::<
            u64,
            std::sync::Arc<T>,
        >(0)?)
    } else {
        Err(BranchCarrierChargeDenial::NotBranchCarrier)
    }
}

fn empty_map_charge<K: Clone + Ord, V: Clone>(
    values: &PersistentOrdMap<K, V>,
    work: &mut Preparation,
) -> Result<Charge, BranchCarrierChargeDenial> {
    work.visit()?;
    if !values.is_empty() {
        return Err(BranchCarrierChargeDenial::NotBranchCarrier);
    }
    // Empty logical contents may still retain a shared base. Only carried
    // accounting establishes its representation cost without reconstruction.
    values
        .prepared_retained_charge()
        .map_err(|_| BranchCarrierChargeDenial::NotBranchCarrier)
}
