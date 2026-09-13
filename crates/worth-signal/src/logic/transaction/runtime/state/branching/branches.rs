mod authority;
mod canonical_transaction;
mod catalog;
mod lifecycle;
mod owner_partition;
mod owner_snapshot;
mod retention;
mod selection;
mod snapshot_storage;
mod transfer;

pub(crate) use authority::BranchState;
pub(in crate::logic::transaction::runtime) use authority::{
    BranchAncestryState, LatestMergeReference,
};
pub(crate) use canonical_transaction::SignalCanonicalCallerUnwind;
pub(in crate::logic::transaction::runtime) use catalog::BranchManager;
pub(in crate::logic::transaction::runtime::state) use catalog::DEFAULT_MAXIMUM_STORED_SIGNAL_BRANCH_SNAPSHOTS;
pub(in crate::logic::transaction::runtime) use owner_partition::SignalOwnerPartitionDenial;
pub(crate) use owner_partition::{
    SignalOwnerMetadataCloseBatch, SignalOwnerMetadataState, SignalOwnerPartition,
    SignalOwnerRetirementCleanup, SignalOwnerSnapshotReservationDenial,
};
pub(in crate::logic::transaction::runtime) use snapshot_storage::SignalBranchSnapshotStorageDenial;

use crate::data::graph::SignalGraph;
use crate::data::telemetry::RuntimeTelemetry;
use crate::logic::transaction::runtime::config::SignalRuntimeConfig;
use crate::state::{SignalBranchId, SignalSnapshotId};

use super::super::merge::BranchMutationLedger;
use super::super::reconstructability::{AuthorityState, DerivedState};

#[derive(Debug, Clone)]
pub(crate) struct SnapshotBranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    config: SignalRuntimeConfig<T>,
    derived: DerivedState<D, I>,
    ancestry: BranchAncestryState,
    mutation_ledger: BranchMutationLedger,
    installed_definition: Option<
        crate::branch::owner_services::conditional_execution::SignalInstalledDefinitionBinding,
    >,
}

#[derive(Debug, Clone)]
pub(crate) struct SnapshotStatePacket<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    branch_id: SignalBranchId,
    snapshot_id: SignalSnapshotId,
    state: SnapshotBranchState<D, I, T>,
}

impl<D, I, T> SnapshotBranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::logic::transaction::runtime::state) fn resource(
        &self,
    ) -> &super::super::resource::ResourceRuntimeState {
        &self.derived.resource
    }

    pub fn from_branch_state(state: &BranchState<D, I, T>) -> Self {
        Self {
            config: state.config().clone(),
            derived: state.derived.clone(),
            ancestry: state.ancestry().clone(),
            mutation_ledger: state.mutation_ledger().clone(),
            installed_definition: state.installed_definition.clone(),
        }
    }

    pub fn into_branch_state(
        self,
        graph: SignalGraph,
        runtime_telemetry: Option<RuntimeTelemetry>,
    ) -> BranchState<D, I, T> {
        let telemetry = if graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        ) {
            runtime_telemetry.unwrap_or(self.derived.telemetry)
        } else {
            RuntimeTelemetry::default()
        };
        let mut state = BranchState::new(
            AuthorityState {
                graph,
                config: self.config,
            },
            DerivedState {
                checkpoint: self.derived.checkpoint,
                resource: self.derived.resource,
                temporal: self.derived.temporal,
                telemetry,
            },
            self.ancestry,
            self.mutation_ledger,
        );
        // Absence is also retained truth: a pre-seal snapshot cannot acquire
        // the destination's current service binding during restoration.
        state.installed_definition = self.installed_definition;
        state
    }

    pub fn packet(self, snapshot_id: SignalSnapshotId) -> SnapshotStatePacket<D, I, T> {
        SnapshotStatePacket {
            branch_id: self.ancestry.branch_id(),
            snapshot_id,
            state: self,
        }
    }
}

impl<D, I, T> SnapshotStatePacket<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn into_parts(
        self,
    ) -> (
        SignalBranchId,
        SignalSnapshotId,
        SnapshotBranchState<D, I, T>,
    ) {
        (self.branch_id, self.snapshot_id, self.state)
    }
}
