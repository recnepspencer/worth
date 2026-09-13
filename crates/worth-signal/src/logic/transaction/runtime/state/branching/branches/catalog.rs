use std::collections::{BTreeMap, BTreeSet};

use crate::data::graph::SignalGraph;
use crate::data::telemetry::RuntimeTelemetry;
use crate::logic::checkpoint::CheckpointRuntime;
use crate::logic::transaction::runtime::config::SignalRuntimeConfig;
use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotId};

use super::super::super::merge::BranchMutationLedger;
use super::super::super::reconstructability::{AuthorityState, DerivedState};
use super::super::super::temporal::TemporalRuntimeState;
use crate::branch::SignalBranchRetirementReceipt;

use super::authority::{BranchAncestryState, BranchState};
use super::SnapshotBranchState;
use crate::branch::{SignalBranchRetentionRegistry, ValidatedSignalBranchName};

pub(in crate::logic::transaction::runtime::state) const DEFAULT_MAXIMUM_STORED_SIGNAL_BRANCH_SNAPSHOTS: usize =
    4_096;

#[derive(Debug, Clone)]
pub(super) struct BranchRuntimeMeta {
    pub(super) ancestry: BranchAncestryState,
    pub(super) mutation_ledger: BranchMutationLedger,
}

pub(in crate::logic::transaction::runtime) struct BranchManager<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) owner_runtime_instance_id: u64,
    pub(super) branches: BTreeMap<SignalBranchId, BranchState<D, I, T>>,
    pub(super) live_branch_catalog: BTreeMap<SignalBranchId, SignalBranchHandle>,
    pub(super) branch_meta: BTreeMap<SignalBranchId, BranchRuntimeMeta>,
    pub(super) snapshots:
        BTreeMap<(SignalBranchId, SignalSnapshotId), SnapshotBranchState<D, I, T>>,
    pub(super) maximum_stored_snapshots: usize,
    pub(super) children_by_parent: BTreeMap<SignalBranchId, BTreeSet<SignalBranchId>>,
    pub(super) retirement_receipts: BTreeMap<SignalBranchId, SignalBranchRetirementReceipt>,
    pub(super) active_merge_participants: BTreeSet<SignalBranchId>,
    pub(super) branch_head_generations: BTreeMap<SignalBranchId, u64>,
    pub(super) branch_restore_snapshot_ids: BTreeMap<SignalBranchId, Option<SignalSnapshotId>>,
    pub(super) retention: Option<SignalBranchRetentionRegistry>,
    pub(super) next_node_index: u32,
    pub(super) next_snapshot_id: u64,
    pub(super) next_branch_id: u64,
    pub(super) next_lineage_artifact_id: u64,
    pub(super) next_lineage_sequence: u64,
}

impl<D, I, T> BranchManager<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn new() -> Self {
        Self {
            owner_runtime_instance_id: 0,
            branches: BTreeMap::new(),
            live_branch_catalog: BTreeMap::new(),
            branch_meta: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            maximum_stored_snapshots: DEFAULT_MAXIMUM_STORED_SIGNAL_BRANCH_SNAPSHOTS,
            children_by_parent: BTreeMap::new(),
            retirement_receipts: BTreeMap::new(),
            active_merge_participants: BTreeSet::new(),
            branch_head_generations: BTreeMap::new(),
            branch_restore_snapshot_ids: BTreeMap::new(),
            retention: Some(SignalBranchRetentionRegistry::new(0)),
            next_node_index: 0,
            next_snapshot_id: 0,
            next_branch_id: 1,
            next_lineage_artifact_id: 0,
            next_lineage_sequence: 0,
        }
    }

    pub fn with_live_catalog(
        live_branch_catalog: BTreeMap<SignalBranchId, SignalBranchHandle>,
        owner_runtime_instance_id: u64,
    ) -> Self {
        let branch_head_generations = live_branch_catalog
            .keys()
            .copied()
            .map(|branch_id| (branch_id, 0))
            .collect();
        let branch_restore_snapshot_ids = live_branch_catalog
            .keys()
            .copied()
            .map(|branch_id| (branch_id, None))
            .collect();
        Self {
            owner_runtime_instance_id,
            live_branch_catalog,
            branch_head_generations,
            branch_restore_snapshot_ids,
            retention: Some(SignalBranchRetentionRegistry::new(
                owner_runtime_instance_id,
            )),
            ..Self::new()
        }
    }

    pub const fn owner_runtime_instance_id(&self) -> u64 {
        self.owner_runtime_instance_id
    }

    pub fn set_maximum_stored_snapshots(&mut self, maximum_stored_snapshots: usize) {
        self.maximum_stored_snapshots = maximum_stored_snapshots;
    }

    pub fn capture_active_state(
        &mut self,
        authority: AuthorityState<T>,
        derived: DerivedState<D, I>,
        ancestry: BranchAncestryState,
        mutation_ledger: BranchMutationLedger,
    ) -> BranchState<D, I, T> {
        self.observe_allocator_state(&authority.graph);
        BranchState::new(authority, derived, ancestry, mutation_ledger)
    }

    pub fn restore_active_state(
        &mut self,
        mut state: BranchState<D, I, T>,
        graph: &mut SignalGraph,
        config: &mut SignalRuntimeConfig<T>,
        checkpoint: &mut CheckpointRuntime<D, I>,
        resource: &mut super::super::super::resource::ResourceRuntimeState,
        temporal: &mut TemporalRuntimeState,
        telemetry: &mut RuntimeTelemetry,
        count_temporal_restore: bool,
    ) {
        self.observe_allocator_state(state.graph());
        state
            .graph_mut()
            .synchronize_node_allocator(self.next_node_index);
        state
            .graph_mut()
            .diagnostics_state_mut()
            .synchronize_branch_snapshot_allocator(self.next_snapshot_id, self.next_branch_id);
        state
            .graph_mut()
            .diagnostics_state_mut()
            .synchronize_lineage_allocator(
                self.next_lineage_artifact_id,
                self.next_lineage_sequence,
            );
        self.record_branch_meta(
            state.branch_id(),
            state.ancestry().clone(),
            state.mutation_ledger().clone(),
        );
        let branch_id = state.branch_id();
        state
            .graph_mut()
            .diagnostics_state_mut()
            .synchronize_branch_catalog(&self.live_branch_catalog, branch_id);
        let (authority, derived, _, _) = state.into_parts();
        let (restored_graph, restored_config) = authority.into_parts();
        *graph = restored_graph;
        *config = restored_config;
        *checkpoint = derived.checkpoint;
        *resource = derived.resource;
        *temporal = derived.temporal;
        let mut restored_telemetry = derived.telemetry;
        if count_temporal_restore {
            resource.bump_restore_epoch(branch_id, &mut restored_telemetry.resource);
            temporal.bump_previous_value_capability_epoch();
        }
        if count_temporal_restore {
            restored_telemetry
                .temporal
                .branch_local_temporal_restore_count += 1;
            restored_telemetry
                .temporal
                .branch_restore_temporal_rebuild_denial_count += 1;
        }
        *telemetry = restored_telemetry;
        self.observe_allocator_state(graph);
    }

    pub fn project_catalog(&self, active_branch: SignalBranchId, active_graph: &mut SignalGraph) {
        active_graph
            .diagnostics_state_mut()
            .synchronize_branch_catalog(&self.live_branch_catalog, active_branch);
    }

    pub fn create_live_branch(
        &mut self,
        name: ValidatedSignalBranchName,
        parent_branch_id: SignalBranchId,
        parent_head_snapshot_id: Option<SignalSnapshotId>,
    ) -> Result<SignalBranchHandle, crate::data::error::SignalError> {
        let branch_id = SignalBranchId(self.next_branch_id.max(1));
        self.next_branch_id = branch_id.0.checked_add(1).ok_or_else(|| {
            crate::data::error::SignalError::internal("Signal branch identity exhausted")
        })?;
        let handle = SignalBranchHandle {
            id: branch_id,
            name: name.into_inner(),
            parent_branch_id: Some(parent_branch_id),
            head_snapshot_id: parent_head_snapshot_id,
        };
        self.live_branch_catalog.insert(branch_id, handle.clone());
        self.branch_restore_snapshot_ids.insert(branch_id, None);
        Ok(handle)
    }

    pub fn set_branch_head_snapshot(
        &mut self,
        branch_id: SignalBranchId,
        snapshot_id: SignalSnapshotId,
    ) {
        if let Some(branch) = self.live_branch_catalog.get_mut(&branch_id) {
            branch.head_snapshot_id = Some(snapshot_id);
        }
        let catalog = self.live_branch_catalog.clone();
        if let Some(state) = self.branches.get_mut(&branch_id) {
            state
                .graph_mut()
                .diagnostics_state_mut()
                .synchronize_branch_catalog(&catalog, branch_id);
        }
    }

    pub(in crate::logic::transaction::runtime::state::branching) fn store_branch_state(
        &mut self,
        state: BranchState<D, I, T>,
    ) {
        self.observe_branch_state(&state);
        self.branches.insert(state.branch_id(), state);
    }

    pub(in crate::logic::transaction::runtime::state::branching) fn observe_active_branch_state(
        &mut self,
        state: &BranchState<D, I, T>,
    ) {
        self.observe_branch_state(state);
    }

    pub fn branch_mutation_ledger(
        &self,
        branch_id: SignalBranchId,
    ) -> Option<&BranchMutationLedger> {
        self.branches
            .get(&branch_id)
            .map(BranchState::mutation_ledger)
            .or_else(|| {
                self.branch_meta
                    .get(&branch_id)
                    .map(|meta| &meta.mutation_ledger)
            })
    }

    pub(in crate::logic::transaction::runtime) fn branch_mutation_ledger_mut(
        &mut self,
        branch_id: SignalBranchId,
        baseline_snapshot_id: Option<crate::state::SignalSnapshotId>,
    ) -> &mut BranchMutationLedger {
        &mut self
            .branch_meta
            .entry(branch_id)
            .or_insert_with(|| BranchRuntimeMeta {
                ancestry: BranchAncestryState::new(branch_id, None, baseline_snapshot_id),
                mutation_ledger: BranchMutationLedger::default()
                    .with_baseline_snapshot(baseline_snapshot_id),
            })
            .mutation_ledger
    }

    pub(in crate::logic::transaction::runtime) fn transaction_branch_state_mut(
        &mut self,
        branch_id: SignalBranchId,
        baseline_snapshot_id: Option<crate::state::SignalSnapshotId>,
    ) -> (
        &mut BranchMutationLedger,
        &mut u64,
        &mut Option<crate::state::SignalSnapshotId>,
    ) {
        let meta = self
            .branch_meta
            .entry(branch_id)
            .or_insert_with(|| BranchRuntimeMeta {
                ancestry: BranchAncestryState::new(branch_id, None, baseline_snapshot_id),
                mutation_ledger: BranchMutationLedger::default()
                    .with_baseline_snapshot(baseline_snapshot_id),
            });
        let generation = self.branch_head_generations.entry(branch_id).or_insert(0);
        let restore_snapshot_id = self
            .branch_restore_snapshot_ids
            .entry(branch_id)
            .or_insert(None);
        (&mut meta.mutation_ledger, generation, restore_snapshot_id)
    }

    fn observe_allocator_state(&mut self, graph: &SignalGraph) {
        self.next_node_index = self.next_node_index.max(graph.node_allocator_state());
        let (next_snapshot_id, next_branch_id) =
            graph.diagnostics_state().branch_snapshot_allocator_state();
        let (next_lineage_artifact_id, next_lineage_sequence) =
            graph.diagnostics_state().lineage_allocator_state();
        self.next_snapshot_id = self.next_snapshot_id.max(next_snapshot_id);
        self.next_branch_id = self.next_branch_id.max(next_branch_id);
        self.next_lineage_artifact_id = self.next_lineage_artifact_id.max(next_lineage_artifact_id);
        self.next_lineage_sequence = self.next_lineage_sequence.max(next_lineage_sequence);
    }

    fn observe_branch_state(&mut self, state: &BranchState<D, I, T>) {
        self.observe_allocator_state(state.graph());
        self.record_branch_meta(
            state.branch_id(),
            state.ancestry().clone(),
            state.mutation_ledger().clone(),
        );
    }

    pub(super) fn record_branch_meta(
        &mut self,
        branch_id: SignalBranchId,
        ancestry: BranchAncestryState,
        mutation_ledger: BranchMutationLedger,
    ) {
        self.branch_meta.insert(
            branch_id,
            BranchRuntimeMeta {
                ancestry,
                mutation_ledger,
            },
        );
    }
}

impl<D, I, T> Default for BranchManager<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}
