//! One change projection for both publication and its later mirror handoff.
use super::{required_symbols::CollectedSymbols, *};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::maintenance::{ChangeWork, MaintenanceRefusal};
use std::sync::Arc;
#[derive(Debug)]
pub(crate) struct RegionSymbolChanges {
    pub(super) head: Arc<CollectedSymbols>,
    pub(super) owned: Arc<CollectedSymbols>,
    pub(super) prefixes_changed: bool,
}
impl RegionSymbolChanges {
    pub(super) fn between(previous: &PartitionState, current: &PartitionState) -> Arc<Self> {
        let mut prefixes_changed = false;
        current
            .entity_arena
            .history_prefixes
            .visit_changed_values(&previous.entity_arena.history_prefixes, &mut |_, _, _| {
                prefixes_changed = true
            });
        current
            .relation_arena
            .history_prefixes
            .visit_changed_values(&previous.relation_arena.history_prefixes, &mut |_, _, _| {
                prefixes_changed = true
            });
        Arc::new(Self {
            head: Arc::new(required_symbols::collect_delta(previous, current, false)),
            owned: Arc::new(required_symbols::collect_delta(previous, current, true)),
            prefixes_changed,
        })
    }
    pub(super) fn derive(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        previous: &PartitionState,
        current: &PartitionState,
    ) -> Result<Arc<Self>, RegionContentFailure> {
        ChangeWork::run_phase(execution, "symbol_admission", |work| {
            let changes = Self::between(previous, current);
            work.prepare(changes.head.visits.saturating_add(changes.owned.visits), 0)?;
            Ok::<_, MaintenanceRefusal>(changes)
        }).map_err(|cause| match cause {
            MaintenanceRefusal::Execution(cause) => RegionContentFailure::Execution(cause),
            _ => unreachable!(/* Proof: This scope only computes a column delta and calls ChangeWork::prepare; no semantic maintainer refusal is constructed. */ "symbol derivation only refuses Execution"),
        })
    }
    pub(super) fn unchanged(&self) -> bool {
        self.head.visits == 0 && self.owned.visits == 0 && !self.prefixes_changed
    }
}
