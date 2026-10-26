use std::collections::BTreeSet;

use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::{LineageEventRecord, LineageNode};
use crate::runtime::state::subsystems::{RuntimeOwnedState, RuntimeSubsystem};

mod node_page;
mod node_residency;
mod resolution_indexes;
mod state;
pub(crate) use node_residency::ResidentLineageNode;

pub(crate) use state::{LineageState, ValidatedLineageEventBatch};

/// The runtime's lineage authority, owned behind its own lock so lineage
/// installation during settlement never demands exclusive access to the runtime.
#[derive(Debug, Default)]
pub(crate) struct LineageSubsystem {
    state: RuntimeOwnedState<LineageState>,
    node_residency:
        RuntimeOwnedState<Option<std::sync::Arc<node_residency::RuntimeLineageNodeResidency>>>,
}

impl LineageSubsystem {
    /// Replace the whole subsystem, for checkpoint restore.
    pub(crate) fn install(&self, state: LineageState) {
        *self.state.write() = state;
        *self.node_residency.write() = None;
    }

    pub(crate) fn install_cold_node_pages(
        &self,
        directory: &crate::durability::data::RelationalCheckpointDirectory,
    ) {
        *self.node_residency.write() = Some(std::sync::Arc::new(
            node_residency::RuntimeLineageNodeResidency::new(directory),
        ));
    }

    pub(crate) fn demand_node(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: LineageId,
    ) -> Result<
        Option<ResidentLineageNode>,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        crate::execution::check_request(execution)
            .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request)?;
        let residency = self.node_residency.read().clone();
        if let Some(residency) = residency
            && residency.covers(id)
        {
            let Some(node) = residency.demand(execution, id)? else {
                return Ok(None);
            };
            let mut state = self.state.write();
            for admitted in node.page_nodes() {
                if state
                    .nodes
                    .get(&admitted.lineage_id())
                    .is_some_and(|prior| prior != admitted)
                    || state
                        .indexed_lineage_for_entity(admitted.entity_id())
                        .0
                        .is_some_and(|prior| prior != admitted.lineage_id())
                {
                    return Err(node.reject_binding());
                }
            }
            for admitted in node.page_nodes() {
                state.record_node(admitted.clone());
            }
            return Ok(Some(node));
        }
        Ok(self
            .state
            .read()
            .nodes
            .get(&id)
            .cloned()
            .map(ResidentLineageNode::published))
    }

    pub(crate) fn snapshot(&self) -> LineageState {
        self.state.read().clone()
    }

    pub(crate) fn identity_allocator(&self) -> super::LineageIdentityAllocator {
        self.state.read().identity_allocator.clone()
    }

    pub(crate) fn identity_frontiers(&self) -> (u64, u64) {
        self.state.read().identity_allocator.frontiers()
    }

    pub(crate) fn set_identity_frontiers(&self, next_lineage_id: u64, next_event_id: u64) {
        self.state
            .write()
            .identity_allocator
            .set_frontiers(next_lineage_id, next_event_id);
    }

    pub(crate) fn advance_identity_to(
        &self,
        next_lineage_id: Option<u64>,
        next_event_id: Option<u64>,
    ) {
        self.state
            .write()
            .identity_allocator
            .advance_to(next_lineage_id, next_event_id);
    }

    /// The highest lineage identity currently published, for recovery floors.
    pub(crate) fn maximum_node_id(&self) -> Option<u64> {
        self.state
            .read()
            .nodes
            .last_key_value()
            .map(|(lineage_id, _)| lineage_id.0)
    }

    /// The highest lineage event identity currently published, for recovery floors.
    pub(crate) fn maximum_event_id(&self) -> Option<u64> {
        self.state
            .read()
            .events()
            .map(|event| event.event_id())
            .max()
    }

    pub(crate) fn install_validated_event_batch(
        &self,
        batch: ValidatedLineageEventBatch,
        publication_commit_id: CommitId,
    ) {
        self.state
            .write()
            .install_validated_event_batch(batch, publication_commit_id);
    }

    pub(crate) fn install_recovered_event_batch(
        &self,
        events: &[LineageEventRecord],
        publication_commit_id: CommitId,
    ) -> Result<(), String> {
        self.state
            .write()
            .install_recovered_event_batch(events, publication_commit_id)
    }

    pub(crate) fn record_node(&self, node: LineageNode) {
        self.state.write().record_node(node);
    }

    pub(crate) fn node(&self, lineage_id: LineageId) -> Option<LineageNode> {
        self.state.read().nodes.get(&lineage_id).cloned()
    }

    pub(crate) fn nodes_snapshot(&self) -> Vec<LineageNode> {
        self.state.read().nodes.values().cloned().collect()
    }

    #[cfg(test)]
    pub(crate) fn node_count(&self) -> usize {
        self.state.read().nodes.len()
    }

    pub(crate) fn branch_event_positions_for_sources(
        &self,
        branch_id: &BranchId,
        lineage_ids: &BTreeSet<LineageId>,
    ) -> BTreeSet<usize> {
        self.state
            .read()
            .branch_event_positions_for_sources(branch_id, lineage_ids)
    }

    pub(crate) fn branch_event_positions_for_lineages(
        &self,
        branch_ids: &BTreeSet<BranchId>,
        lineage_ids: &BTreeSet<LineageId>,
        sources_only: bool,
    ) -> (BTreeSet<usize>, usize) {
        self.state
            .read()
            .branch_event_positions_for_lineages(branch_ids, lineage_ids, sources_only)
    }

    pub(crate) fn indexed_lineage_for_entity(
        &self,
        entity_id: EntityId,
    ) -> (Option<LineageId>, usize) {
        self.state.read().indexed_lineage_for_entity(entity_id)
    }

    pub(crate) fn indexed_lineages_are_exclusive_to_branch(
        &self,
        lineage_ids: &BTreeSet<LineageId>,
        branch_id: &BranchId,
        sources_only: bool,
    ) -> (bool, usize) {
        self.state.read().indexed_lineages_are_exclusive_to_branch(
            lineage_ids,
            branch_id,
            sources_only,
        )
    }

    pub(crate) fn event_publication_commit(&self, position: usize) -> Option<CommitId> {
        self.state.read().event_publication_commit(position)
    }

    pub(crate) fn event(&self, position: usize) -> Option<std::sync::Arc<LineageEventRecord>> {
        self.state.read().event(position)
    }

    /// Every published event, shared rather than copied, for scans that must not
    /// hold the subsystem lock while they filter.
    pub(crate) fn shared_events(&self) -> Vec<std::sync::Arc<LineageEventRecord>> {
        self.state.read().shared_events()
    }

    #[cfg(test)]
    pub(crate) fn event_count(&self) -> usize {
        self.state.read().events().count()
    }

    #[cfg(test)]
    pub(crate) fn branch_events_snapshot(&self, branch_id: &BranchId) -> Vec<LineageEventRecord> {
        self.state
            .read()
            .branch_events(branch_id)
            .cloned()
            .collect()
    }

    pub(crate) fn replace_events(&self, events: Vec<(LineageEventRecord, CommitId)>) {
        self.state.write().replace_events(events);
    }

    pub(crate) fn drain_events(&self) -> Vec<(LineageEventRecord, CommitId)> {
        self.state.write().drain_events().collect()
    }
}

impl RuntimeSubsystem for LineageSubsystem {
    type Config = ();

    fn new(_: &Self::Config) -> Self {
        Self {
            state: RuntimeOwnedState::new(LineageState::empty()),
            node_residency: Default::default(),
        }
    }

    fn fork(&self) -> Self {
        Self {
            state: RuntimeOwnedState::new(self.snapshot()),
            node_residency: self.node_residency.detached(),
        }
    }
}
