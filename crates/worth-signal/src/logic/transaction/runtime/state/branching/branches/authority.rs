use crate::data::graph::SignalGraph;
use crate::data::node::CheckpointNodeImage;
use crate::data::telemetry::RuntimeTelemetry;
use crate::data::temporal::{
    TemporalWakeOwner, TemporalWakeRetirementBatch, TemporalWakeRetirementReason,
};
use crate::logic::checkpoint::CheckpointRuntime;
use crate::logic::transaction::runtime::config::SignalRuntimeConfig;
use crate::state::{SignalBranchId, SignalSnapshotId};

use super::super::super::merge::{BranchMergeKind, BranchMergeStrategy, BranchMutationLedger};
use super::super::super::reconstructability::{AuthorityState, DerivedState};
use super::super::super::temporal::TemporalRuntimeState;

#[path = "authority/conditional_owned_async.rs"]
mod conditional_owned_async;
#[cfg(test)]
#[path = "authority/replacement_test_observation.rs"]
mod replacement_test_observation;

pub(crate) struct SignalForkedBranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) state: BranchState<D, I, T>,
    pub(crate) work: crate::data::graph::signal_graph::SignalGraphForkWork,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SignalBranchPersistentSharing {
    pub(crate) graph: crate::data::graph::signal_graph::SignalGraphPersistentSharing,
    pub(crate) config_roots_shared: bool,
    pub(crate) derived_roots_shared: bool,
}

#[cfg(test)]
pub(crate) struct SignalBranchPersistentIdentity<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    graph: crate::data::graph::signal_graph::SignalGraphPersistentIdentity,
    config: SignalRuntimeConfig<T>,
    derived: DerivedState<D, I>,
    pub(crate) hot_page_identities: Vec<usize>,
}

#[cfg(test)]
impl<D, I, T> SignalBranchPersistentIdentity<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn sharing_with(&self, other: &Self) -> SignalBranchPersistentSharing {
        SignalBranchPersistentSharing {
            graph: self.graph.sharing_with(&other.graph),
            config_roots_shared: self.config.shares_fork_storage_with(&other.config),
            derived_roots_shared: self.derived.shares_fork_storage_with(&other.derived),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::logic::transaction::runtime) struct LatestMergeReference {
    source_branch_id: SignalBranchId,
    source_snapshot_id: Option<SignalSnapshotId>,
    target_snapshot_id_before: Option<SignalSnapshotId>,
    target_snapshot_id_after: Option<SignalSnapshotId>,
    merge_kind: BranchMergeKind,
    merge_strategy: BranchMergeStrategy,
}

impl LatestMergeReference {
    pub fn new(
        source_branch_id: SignalBranchId,
        source_snapshot_id: Option<SignalSnapshotId>,
        target_snapshot_id_before: Option<SignalSnapshotId>,
        target_snapshot_id_after: Option<SignalSnapshotId>,
        merge_kind: BranchMergeKind,
        merge_strategy: BranchMergeStrategy,
    ) -> Self {
        Self {
            source_branch_id,
            source_snapshot_id,
            target_snapshot_id_before,
            target_snapshot_id_after,
            merge_kind,
            merge_strategy,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::logic::transaction::runtime) struct BranchAncestryState {
    branch_id: SignalBranchId,
    parent_branch_id: Option<SignalBranchId>,
    forked_from_snapshot_id: Option<SignalSnapshotId>,
    latest_merge_reference: Option<LatestMergeReference>,
}

impl BranchAncestryState {
    pub fn new(
        branch_id: SignalBranchId,
        parent_branch_id: Option<SignalBranchId>,
        forked_from_snapshot_id: Option<SignalSnapshotId>,
    ) -> Self {
        Self {
            branch_id,
            parent_branch_id,
            forked_from_snapshot_id,
            latest_merge_reference: None,
        }
    }

    pub fn set_latest_merge_reference(
        &mut self,
        latest_merge_reference: Option<LatestMergeReference>,
    ) {
        self.latest_merge_reference = latest_merge_reference;
    }

    pub fn branch_id(&self) -> SignalBranchId {
        self.branch_id
    }

    pub fn parent_branch_id(&self) -> Option<SignalBranchId> {
        self.parent_branch_id
    }

    pub fn forked_from_snapshot_id(&self) -> Option<SignalSnapshotId> {
        self.forked_from_snapshot_id
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) authority: AuthorityState<T>,
    pub(super) derived: DerivedState<D, I>,
    ancestry: BranchAncestryState,
    pub(super) mutation_ledger: BranchMutationLedger,
    pub(super) installed_definition: Option<
        crate::branch::owner_services::conditional_execution::SignalInstalledDefinitionBinding,
    >,
}

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn fork_for_owner_cell(
        &mut self,
        parent: &crate::state::SignalBranchHandle,
        destination: crate::state::SignalBranchHandle,
    ) -> SignalForkedBranchState<D, I, T> {
        let (graph, work) = self.authority.graph.fork_owner_branch();
        let config = self.authority.config.fork_persistent();
        let mut fork = Self::new(
            AuthorityState { graph, config },
            self.derived.fork_persistent(),
            BranchAncestryState::new(
                destination.id,
                Some(parent.id),
                destination.head_snapshot_id,
            ),
            BranchMutationLedger::default().with_baseline_snapshot(destination.head_snapshot_id),
        );
        // Only this admitted owner fork carries installed lineage. Ordinary
        // graph forks deliberately clear their construction lowering claim.
        fork.installed_definition = self.installed_definition.clone();
        let catalog = std::iter::once((destination.id, destination.clone())).collect();
        fork.graph_mut()
            .diagnostics_state_mut()
            .synchronize_branch_catalog(&catalog, destination.id);
        crate::diagnostics::recorder::record_branch_fork_lineage(
            fork.graph_mut(),
            destination.id,
            parent.id,
            destination.name,
            parent.name.clone(),
        );
        SignalForkedBranchState { state: fork, work }
    }

    #[cfg(test)]
    pub(crate) fn persistent_identity(&self) -> SignalBranchPersistentIdentity<D, I, T> {
        SignalBranchPersistentIdentity {
            graph: self.graph().persistent_identity(),
            config: self.authority.config.fork_storage_identity(),
            derived: self.derived.fork_storage_identity(),
            hot_page_identities: self.graph().hot_page_identities(),
        }
    }

    pub(in crate::logic::transaction::runtime) fn new(
        authority: AuthorityState<T>,
        derived: DerivedState<D, I>,
        ancestry: BranchAncestryState,
        mutation_ledger: BranchMutationLedger,
    ) -> Self {
        Self {
            authority,
            derived,
            ancestry,
            mutation_ledger,
            installed_definition: None,
        }
    }

    pub(crate) fn seal_definition_custody(&mut self, definition_basis: u64) {
        if self.installed_definition.is_none() {
            self.installed_definition = Some(
                crate::branch::owner_services::conditional_execution::SignalInstalledDefinitionBinding::capture(
                    self.graph(), definition_basis,
                ),
            );
        }
    }

    pub(crate) fn installed_definition(
        &self,
    ) -> Option<
        &crate::branch::owner_services::conditional_execution::SignalInstalledDefinitionBinding,
    > {
        self.installed_definition.as_ref()
    }

    pub fn graph(&self) -> &SignalGraph {
        &self.authority.graph
    }

    pub fn graph_mut(&mut self) -> &mut SignalGraph {
        &mut self.authority.graph
    }

    pub(in crate::logic::transaction::runtime) fn ancestry(&self) -> &BranchAncestryState {
        &self.ancestry
    }

    pub(crate) fn owner_retirement_forked_from_snapshot_id(&self) -> Option<SignalSnapshotId> {
        self.ancestry.forked_from_snapshot_id()
    }

    pub(in crate::logic::transaction::runtime) fn ancestry_mut(
        &mut self,
    ) -> &mut BranchAncestryState {
        &mut self.ancestry
    }

    pub fn mutation_ledger(&self) -> &BranchMutationLedger {
        &self.mutation_ledger
    }

    pub fn mutation_ledger_mut(&mut self) -> &mut BranchMutationLedger {
        &mut self.mutation_ledger
    }

    pub fn config(&self) -> &SignalRuntimeConfig<T> {
        &self.authority.config
    }

    pub fn checkpoint(&self) -> &CheckpointRuntime<D, I> {
        &self.derived.checkpoint
    }

    pub fn runtime_telemetry(&self) -> &RuntimeTelemetry {
        &self.derived.telemetry
    }

    pub(in crate::logic::transaction::runtime) fn temporal(&self) -> &TemporalRuntimeState {
        &self.derived.temporal
    }

    pub(in crate::logic::transaction::runtime::state) fn resource(
        &self,
    ) -> &super::super::super::resource::ResourceRuntimeState {
        &self.derived.resource
    }

    pub fn branch_id(&self) -> SignalBranchId {
        self.ancestry.branch_id()
    }

    pub(in crate::logic::transaction::runtime) fn into_parts(
        self,
    ) -> (
        AuthorityState<T>,
        DerivedState<D, I>,
        BranchAncestryState,
        BranchMutationLedger,
    ) {
        (
            self.authority,
            self.derived,
            self.ancestry,
            self.mutation_ledger,
        )
    }

    pub fn reset_mutation_ledger(&mut self, baseline_snapshot: Option<SignalSnapshotId>) {
        self.mutation_ledger =
            BranchMutationLedger::default().with_baseline_snapshot(baseline_snapshot);
    }

    #[cfg(test)]
    pub fn clear_merge_boundary_proof(&mut self) {
        self.mutation_ledger = BranchMutationLedger::default();
    }

    pub fn clear_branch_mutation_nodes(&mut self) {
        self.graph_mut().clear_branch_mutation_nodes();
    }

    pub fn replace_node_from_checkpoint_image(
        &mut self,
        node: crate::data::handle::NodeId,
        image: CheckpointNodeImage,
    ) -> Result<TemporalWakeRetirementBatch, crate::data::error::SignalError> {
        if !self.graph().is_alive(node) {
            return Err(crate::data::error::SignalError::invalid_input(format!(
                "cannot replace checkpoint image for non-live node owner {node}"
            )));
        }

        self.graph_mut()
            .replace_entry_from_checkpoint_image(node, image)?;
        self.derived.temporal.retire_wakes_for_owner(
            TemporalWakeOwner::Node(node),
            TemporalWakeRetirementReason::Superseded,
            Some(&mut self.derived.telemetry.temporal),
        )
    }
}
