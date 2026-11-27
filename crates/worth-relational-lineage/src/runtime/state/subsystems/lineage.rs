use std::collections::{BTreeMap, BTreeSet};

use super::lineage_identity::LineageIdentityAllocator;
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::lineage::data::events::LineageEventRecord;
use crate::contracts::lineage::data::graph::LineageNode;
use crate::contracts::lineage::data::naming::{PersistentNameHolder, PersistentNameIdentity};
use worth_relational_storage::internal_port::storage::substrate::owned_state::RuntimeOwnedState;

pub mod event_page;
pub mod event_residency;
pub mod group_states;
pub mod name_indexes;
pub mod naming_basis;
pub mod naming_basis_binding;
pub mod node_page;
pub mod page_admission;
pub use page_admission::PreparedLineageColdPages;
pub mod published_batch;
pub mod source_close;
pub mod source_retirement;
pub use event_residency::ResidentLineageEventPage;
pub use naming_basis_binding::LineageNamingBasisBinding;
pub mod node_residency;
pub mod resolution_indexes;
pub mod state;
pub use node_residency::ResidentLineageNode;

pub use state::{LineageState, ValidatedLineageEventBatch};

/// The runtime's lineage authority, owned behind its own lock so lineage
/// installation during settlement never demands exclusive access to the runtime.
#[derive(Debug, Default)]
pub struct LineageSubsystem {
    discovery_metadata: std::sync::Mutex<()>,
    state: RuntimeOwnedState<LineageState>,
    event_admissions:
        RuntimeOwnedState<BTreeSet<worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator>>,
    event_residency:
        RuntimeOwnedState<Option<std::sync::Arc<event_residency::RuntimeLineageEventResidency>>>,
    node_residency:
        RuntimeOwnedState<Option<std::sync::Arc<node_residency::RuntimeLineageNodeResidency>>>,
}

impl LineageSubsystem {
    /// Serialize only the short read-side canonical snapshot/validation calls.
    /// No provider callback or lineage traversal belongs inside this scope.
    pub fn with_discovery_metadata<T>(&self, observe: impl FnOnce() -> T) -> T {
        let _snapshot = self
            .discovery_metadata
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        observe()
    }

    /// Replace the whole subsystem, for checkpoint restore.
    pub fn install(&self, state: LineageState) {
        *self.state.write() = state;
        *self.node_residency.write() = None;
        *self.event_residency.write() = None;
        self.event_admissions.write().clear();
    }

    pub fn demand_node(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: LineageId,
    ) -> Result<
        Option<ResidentLineageNode>,
        worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure,
    >{
        crate::contracts::execution::request_custody::check_request(execution)
            .map_err(worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure::Request)?;
        let residency = self.node_residency.read().clone();
        if let Some(residency) = residency
            && residency.covers(id)
        {
            let Some(node) = residency.demand(execution, id)? else {
                return Ok(None);
            };
            let mut state = self.state.write();
            for admitted in node.page_nodes() {
                // A lineage binds one entity forever; an entity may hold
                // several lineages across retire/restore epochs.
                if state
                    .nodes
                    .get(&admitted.lineage_id())
                    .is_some_and(|prior| prior != admitted)
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

    pub fn published_rows(&self) -> Vec<(std::sync::Arc<LineageEventRecord>, CommitId)> {
        self.state.read().published_rows()
    }

    /// A working copy of the lineage state for a reconstruction lane.
    pub fn snapshot_for_reconstruction(
        &self,
        lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    ) -> LineageState {
        self.state.read().duplicate_for_reconstruction(lane)
    }

    pub fn identity_allocator(&self) -> LineageIdentityAllocator {
        self.state.read().identity_allocator.clone()
    }

    pub fn identity_frontiers(&self) -> (u64, u64) {
        self.state.read().identity_allocator.frontiers()
    }

    pub fn set_identity_frontiers(&self, next_lineage_id: u64, next_event_id: u64) {
        self.state
            .write()
            .identity_allocator
            .set_frontiers(next_lineage_id, next_event_id);
    }

    pub fn advance_identity_to(&self, next_lineage_id: Option<u64>, next_event_id: Option<u64>) {
        self.state
            .write()
            .identity_allocator
            .advance_to(next_lineage_id, next_event_id);
    }

    /// The highest lineage identity currently published, for recovery floors.
    pub fn maximum_node_id(&self) -> Option<u64> {
        self.state
            .read()
            .nodes
            .last_key_value()
            .map(|(lineage_id, _)| lineage_id.0)
    }

    /// The highest lineage event identity currently published, for recovery floors.
    pub fn maximum_event_id(&self) -> Option<u64> {
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

    pub fn install_recovered_event_batch(
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

    pub fn record_relation_lineage(&self, relation_id: RelationId, lineage_id: LineageId) {
        self.state
            .write()
            .record_relation_lineage(relation_id, lineage_id);
    }

    pub fn record_fork_ancestry_metadata(
        &self,
        target_branch: BranchId,
        source_branch: BranchId,
        fork_commit_id: CommitId,
    ) {
        self.state.write().record_fork_ancestry_metadata(
            target_branch,
            source_branch,
            fork_commit_id,
        );
    }

    pub fn relation_lineage(&self, relation_id: RelationId) -> Option<LineageId> {
        self.state.read().relation_lineage(relation_id)
    }

    pub fn relation_lineages_snapshot(&self) -> Vec<(RelationId, LineageId)> {
        self.state.read().relation_lineages_snapshot()
    }

    pub fn holder_and_name_for_lineage(
        &self,
        branch_id: &BranchId,
        lineage_id: LineageId,
    ) -> Option<(PersistentNameHolder, PersistentNameIdentity)> {
        self.state
            .read()
            .holder_and_name_for_lineage(branch_id, lineage_id)
    }

    pub fn naming_basis_binding(&self) -> LineageNamingBasisBinding {
        LineageNamingBasisBinding::new(self.state.share())
    }

    pub fn branch_live_persistent_names(
        &self,
        branch_id: &BranchId,
    ) -> BTreeMap<PersistentNameIdentity, PersistentNameHolder> {
        self.state.read().branch_live_persistent_names(branch_id)
    }

    pub fn node(&self, lineage_id: LineageId) -> Option<LineageNode> {
        self.state.read().nodes.get(&lineage_id).cloned()
    }

    pub fn nodes_snapshot(&self) -> Vec<LineageNode> {
        self.state.read().nodes.values().cloned().collect()
    }

    pub fn node_count(&self) -> usize {
        self.state.read().nodes.len()
    }

    pub fn branch_event_positions_for_sources(
        &self,
        branch_id: &BranchId,
        lineage_ids: &BTreeSet<LineageId>,
    ) -> BTreeSet<usize> {
        self.state
            .read()
            .branch_event_positions_for_sources(branch_id, lineage_ids)
    }

    pub fn branch_event_positions_for_lineages(
        &self,
        branch_ids: &BTreeSet<BranchId>,
        lineage_ids: &BTreeSet<LineageId>,
        sources_only: bool,
    ) -> (BTreeSet<usize>, usize) {
        self.state
            .read()
            .branch_event_positions_for_lineages(branch_ids, lineage_ids, sources_only)
    }

    pub fn indexed_lineages_for_entity(&self, entity_id: EntityId) -> (Vec<LineageId>, usize) {
        self.state.read().indexed_lineages_for_entity(entity_id)
    }

    pub fn indexed_lineages_are_exclusive_to_branch(
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

    pub fn event_publication_commit(&self, position: usize) -> Option<CommitId> {
        self.state.read().event_publication_commit(position)
    }

    pub fn event(&self, position: usize) -> Option<std::sync::Arc<LineageEventRecord>> {
        self.state.read().event(position)
    }

    /// Every published event, shared rather than copied, for scans that must not
    /// hold the subsystem lock while they filter.
    pub fn shared_events(&self) -> Vec<std::sync::Arc<LineageEventRecord>> {
        self.state.read().shared_events()
    }

    pub fn event_count(&self) -> usize {
        self.state.read().events().count()
    }

    pub fn branch_events_snapshot_for_test(&self, branch_id: &BranchId) -> Vec<LineageEventRecord> {
        self.state
            .read()
            .branch_events(branch_id)
            .cloned()
            .collect()
    }

    pub fn finalize_cold_naming_basis(
        &self,
        lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    ) {
        self.state.write().finalize_cold_naming_basis(lane);
    }

    /// Replaces the published group lineage with recovered canonical records
    /// after fork ancestry is recorded; refuses the first branch whose
    /// records contradict its group posture.
    pub fn replace_group_lineage(
        &self,
        records: Vec<(
            BranchId,
            CommitId,
            Vec<crate::lineage::groups::CommittedGroupTransition>,
        )>,
    ) -> Result<(), (BranchId, crate::lineage::groups::GroupStateDenial)> {
        self.state.write().replace_group_lineage(records)
    }

    /// The group transitions `branch` published at exactly `commit`.
    pub fn published_group_lineage_at(
        &self,
        branch: &BranchId,
        commit: CommitId,
    ) -> Vec<crate::lineage::groups::CommittedGroupTransition> {
        let state = self.state.read();
        let record = state.group_lineage_at(branch, commit);
        record
            .map(|record| record.transitions.to_vec())
            .unwrap_or_default()
    }

    /// The first group lineage `b` published after `base` (merge refusal).
    pub fn group_lineage_after(&self, b: &BranchId, c: CommitId) -> Option<(CommitId, String)> {
        (self.state.read().group_lineage_after(b, c)).map(|(at, id)| (at, id.to_owned()))
    }

    /// The first commit `b` published named lineage at after `c` (merge refusal).
    pub fn named_lineage_after(&self, b: &BranchId, c: CommitId) -> Option<CommitId> {
        self.state.read().named_lineage_after(b, c)
    }

    pub fn replace_events(
        &self,
        lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
        events: Vec<(LineageEventRecord, CommitId)>,
    ) {
        let mut admissions = self.event_admissions.write();
        self.state.write().replace_events(lane, events);
        admissions.clear();
    }

    pub fn drain_events(&self) -> Vec<(LineageEventRecord, CommitId)> {
        self.state.write().drain_events().collect()
    }
}

impl LineageSubsystem {
    pub fn new() -> Self {
        Self {
            discovery_metadata: Default::default(),
            state: RuntimeOwnedState::new(LineageState::empty()),
            node_residency: Default::default(),
            event_residency: Default::default(),
            event_admissions: Default::default(),
        }
    }

    pub fn fork(&self) -> Self {
        // Admission membership and the copied resolution index form one basis.
        let admissions = self.event_admissions.read();
        Self {
            discovery_metadata: Default::default(),
            state: RuntimeOwnedState::new(self.state.read().duplicate_for_runtime_fork()),
            node_residency: self.node_residency.detached(),
            event_residency: self.event_residency.detached(),
            event_admissions: RuntimeOwnedState::new(admissions.clone()),
        }
    }
}
