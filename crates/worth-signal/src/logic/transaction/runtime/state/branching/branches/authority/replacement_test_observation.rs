use crate::data::graph::signal_graph::{
    SignalGraphCloneLocalObservation, SignalGraphRetainedObservation,
};
use crate::data::handle::NodeId;
use crate::data::telemetry::RuntimeTelemetry;
use crate::logic::checkpoint::CheckpointReplacementObservation;
use crate::logic::transaction::runtime::config::SignalRuntimeConfigReplacementObservation;

use crate::logic::transaction::runtime::state::resource::ResourceRuntimeState;
use crate::logic::transaction::runtime::state::temporal::TemporalRuntimeState;

use super::{BranchAncestryState, BranchMutationLedger, BranchState};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SignalBranchReplacementObservation<D, I, T>
where
    D: Copy + Ord,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) graph_retained: SignalGraphRetainedObservation,
    pub(crate) graph_clone_local: SignalGraphCloneLocalObservation,
    pub(crate) config: SignalRuntimeConfigReplacementObservation<T>,
    pub(crate) checkpoint: CheckpointReplacementObservation<D, I>,
    resource: ResourceRuntimeState,
    temporal: TemporalRuntimeState,
    pub(crate) telemetry: RuntimeTelemetry,
    ancestry: BranchAncestryState,
    pub(crate) mutation_ledger: BranchMutationLedger,
}

impl<D, I, T> SignalBranchReplacementObservation<D, I, T>
where
    D: Copy + Ord,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn resource_matches(&self, other: &Self) -> bool {
        self.resource == other.resource
    }

    pub(crate) fn temporal_matches(&self, other: &Self) -> bool {
        self.temporal == other.temporal
    }

    pub(crate) fn ancestry_matches(&self, other: &Self) -> bool {
        self.ancestry == other.ancestry
    }
}

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn replacement_observation(
        &self,
        nodes: &[NodeId],
        domains: &[D],
    ) -> SignalBranchReplacementObservation<D, I, T> {
        SignalBranchReplacementObservation {
            graph_retained: self.graph().replacement_retained_observation(),
            graph_clone_local: self.graph().replacement_clone_local_observation(),
            config: self.authority.config.replacement_observation(nodes),
            checkpoint: self.derived.checkpoint.replacement_observation(domains),
            resource: self.derived.resource.clone(),
            temporal: self.derived.temporal.clone(),
            telemetry: self.derived.telemetry,
            ancestry: self.ancestry.clone(),
            mutation_ledger: self.mutation_ledger.clone(),
        }
    }
}
