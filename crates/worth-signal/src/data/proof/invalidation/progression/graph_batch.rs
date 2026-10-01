use worth_proof::{Binding, DisjointKeySetFamily};

use super::super::binding::DependencyRevision;
use super::{InvalidationReadinessEpoch, InvalidationWorkBindingAxes};
use crate::data::aspect::{Aspect, AspectMask, MAX_ASPECTS};
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::node::BoundedSignalInputs;

/// Exclusive private proposal surfaces. These keys do not assert disjointness
/// of final graph writes: shared subscriptions, consumers and observations are
/// composed by the epoch publication owner and published canonically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GraphProposalKey {
    State(NodeId),
    Dependencies(NodeId),
    Aspect(NodeId, Aspect),
    Subscriptions(NodeId),
    Snapshot(NodeId),
    Lineage(NodeId),
    Observation(NodeId),
    Diagnostic(NodeId),
}

impl worth_execution::ChargedBytes for GraphProposalKey {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GraphMemberBinding {
    pub(crate) target: NodeId,
    pub(crate) task_order: usize,
    pub(crate) declaration: BoundedSignalInputs,
    pub(crate) producer_versions: Vec<u64>,
    pub(crate) dependency_revision: DependencyRevision,
    pub(crate) produced_aspects: AspectMask,
    pub(crate) invalidation: Option<InvalidationWorkBindingAxes>,
}

worth_proof::binding_axes! {
    pub(crate) struct GraphBatchBindingAxes {
        pub(crate) graph_instance: u64 => GraphInstance,
        pub(crate) epoch: InvalidationReadinessEpoch => Epoch,
        pub(crate) observation_generation: u64 => ObservationGeneration,
        pub(crate) members: Vec<GraphMemberBinding> => Members,
    }
    drift pub(crate) enum GraphBatchBindingDrift;
}

/// Only the invalidation progression owner can create this batch. Planner
/// summaries, executor preference and serialized diagnostics cannot authorize it.
pub(crate) struct DisjointGraphBatch {
    binding: Binding<GraphBatchBindingAxes>,
    family: DisjointKeySetFamily<NodeId, GraphProposalKey>,
}

impl DisjointGraphBatch {
    pub(super) fn admit(
        expected: GraphBatchBindingAxes,
        current: GraphBatchBindingAxes,
    ) -> Result<Self, SignalError> {
        let binding = Binding::new(expected);
        binding
            .ensure_matches(&Binding::new(current))
            .map_err(|_| {
                SignalError::invalid_input("graph batch binding drifted before admission")
            })?;
        // Derive the complete local footprint here. Callers cannot authorize
        // dispatch by supplying a covered target with an incomplete key list.
        let effects = binding
            .axes()
            .members
            .iter()
            .map(|member| {
                let node = member.target;
                let mut keys = Vec::with_capacity(MAX_ASPECTS + 7);
                keys.extend([
                    GraphProposalKey::State(node),
                    GraphProposalKey::Dependencies(node),
                    GraphProposalKey::Subscriptions(node),
                    GraphProposalKey::Snapshot(node),
                    GraphProposalKey::Lineage(node),
                    GraphProposalKey::Observation(node),
                    GraphProposalKey::Diagnostic(node),
                ]);
                for index in 0..MAX_ASPECTS {
                    let aspect = Aspect::new(index as u8);
                    if member
                        .produced_aspects
                        .intersects(AspectMask::from_aspect(aspect))
                    {
                        keys.push(GraphProposalKey::Aspect(node, aspect));
                    }
                }
                keys.sort_unstable();
                (node, keys)
            })
            .collect();
        let family = DisjointKeySetFamily::try_from_sorted_sets(effects)
            .map_err(|_| SignalError::invalid_input("graph batch effect footprints overlap"))?;
        if binding
            .axes()
            .members
            .iter()
            .map(|member| member.target)
            .ne(family.members().iter().map(|(target, _)| *target))
        {
            return Err(SignalError::invalid_input(
                "graph batch footprint coverage mismatch",
            ));
        }
        Ok(Self { binding, family })
    }

    pub(crate) fn binding(&self) -> &GraphBatchBindingAxes {
        self.binding.axes()
    }
    pub(crate) fn effects(&self) -> &[(NodeId, Vec<GraphProposalKey>)] {
        self.family.members()
    }
}
