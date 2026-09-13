use crate::data::dependency::DependencyEdge;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::{NodeColdData, NodeHotData, NodeState, NodeWarmData};

/// One borrowed live-node view assembled directly from canonical graph storage.
pub(crate) struct GraphDiagnosticNode<'a> {
    node: NodeId,
    hot: &'a NodeHotData,
    warm: &'a NodeWarmData,
    cold: Option<&'a NodeColdData>,
    dependencies: &'a [DependencyEdge],
    subscribers: &'a [NodeId],
}

impl GraphDiagnosticNode<'_> {
    pub(crate) fn node(&self) -> NodeId {
        self.node
    }

    pub(crate) fn state(&self) -> NodeState {
        self.hot.state
    }

    pub(crate) fn dependencies(&self) -> &[DependencyEdge] {
        self.dependencies
    }

    pub(crate) fn subscribers(&self) -> &[NodeId] {
        self.subscribers
    }

    pub(crate) fn node_runtime_artifact_state_present(&self) -> bool {
        self.warm.runtime_artifact_state.is_some()
    }

    pub(crate) fn execution_record_present(&self) -> bool {
        self.cold
            .and_then(|cold| cold.execution_trace)
            .and_then(|stamp| stamp.execution_record_id)
            .is_some()
    }

    pub(crate) fn causality_present(&self) -> bool {
        self.cold.and_then(|cold| cold.causality.as_ref()).is_some()
    }
}

impl SignalGraph {
    /// Borrow every live node once without reconstructing entries or repeating
    /// handle validation. Persistent lanes and topology stores remain the
    /// authority for fork overlays and appended storage.
    pub(crate) fn diagnostic_nodes(
        &self,
    ) -> impl Iterator<Item = super::GraphDiagnosticNode<'_>> + '_ {
        debug_assert_eq!(self.arena.nodes.len(), self.arena.hot.len());
        debug_assert_eq!(self.arena.nodes.len(), self.arena.warm.len());
        debug_assert_eq!(self.arena.nodes.len(), self.arena.cold.len());

        self.arena
            .nodes
            .iter()
            .zip(self.arena.hot.iter())
            .zip(self.arena.warm.iter())
            .zip(self.arena.cold.iter())
            .enumerate()
            .filter_map(move |(index, (((slot, hot), warm), cold))| {
                if !slot.is_occupied() {
                    return None;
                }
                let hot = hot.as_ref()?;
                Some(GraphDiagnosticNode {
                    node: NodeId::new(index as u32, slot.generation),
                    hot,
                    warm,
                    cold: cold.as_deref(),
                    dependencies: self.topology.dependency_edges.get(hot.dependencies_id),
                    subscribers: self.topology.subscriber_edges.get(hot.subscribers_id),
                })
            })
    }
}
