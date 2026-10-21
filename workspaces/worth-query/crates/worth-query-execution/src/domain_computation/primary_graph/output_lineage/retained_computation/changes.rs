//! The exact prior record alone pairs retained state with its native frontier.
use super::super::invalidation::ComputationChanges;
use super::super::invalidation::SourceInvalidationOwner;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::RetainedComputation;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;
impl PriorComputationRecord {
    pub(in crate::domain_computation::primary_graph) fn changes(
        &self,
        state: &Arc<RetainedComputation>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<Option<ComputationChanges>, CompanionPreflightStop> {
        let Some(record) = self.cell.get() else {
            return Ok(None);
        };
        let row = record
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(&row.computation, RecordedComputation::Retained(held) if Arc::ptr_eq(held, state))
        {
            return Ok(None);
        }
        drop(row);
        let Some(facts) = record.observed_source_facts() else {
            return Ok(None);
        };
        owner
            .computation_changes(runtime, snapshot, &record.settlement_identity, &facts)
            .map(|changes| changes.map(|changes| changes.bind_original(state.original_facts())))
    }
    pub(in crate::domain_computation::primary_graph) fn matches_changes(
        &self,
        changes: &ComputationChanges,
    ) -> bool {
        self.cell
            .get()
            .is_some_and(|record| Arc::ptr_eq(&record.settlement_identity, &changes.identity))
    }
}
