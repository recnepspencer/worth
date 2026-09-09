use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;

use super::super::runtime::graph::{EdgeTopology, SignalGraph};
mod upstream;

impl SignalGraph {
    pub(crate) fn refresh_runtime_dependencies_of(
        &mut self,
        node: NodeId,
    ) -> Result<(), SignalError> {
        self.refresh_runtime_dependencies_with_work(
            node,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
    }

    pub(crate) fn refresh_runtime_dependencies_with_work(
        &mut self,
        node: NodeId,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        work.reserve(Some(1))?;
        EdgeTopology::prune_dead_dependency_edges(self, node, work)
    }

    pub(crate) fn runtime_dependencies_of(
        &mut self,
        node: NodeId,
    ) -> Result<&[DependencyEdge], SignalError> {
        self.refresh_runtime_dependencies_of(node)?;
        self.current_runtime_dependencies_of(node)
    }

    pub(crate) fn current_runtime_dependencies_of(
        &self,
        node: NodeId,
    ) -> Result<&[DependencyEdge], SignalError> {
        self.raw_dependencies_of(node)
    }

    pub(crate) fn refresh_runtime_subscribers_of(
        &mut self,
        node: NodeId,
    ) -> Result<(), SignalError> {
        EdgeTopology::prune_dead_subscriber_edges(self, node)
    }

    pub(crate) fn runtime_subscribers_of(
        &mut self,
        node: NodeId,
    ) -> Result<&[NodeId], SignalError> {
        self.refresh_runtime_subscribers_of(node)?;
        self.current_runtime_subscribers_of(node)
    }

    pub(crate) fn current_runtime_subscribers_of(
        &self,
        node: NodeId,
    ) -> Result<&[NodeId], SignalError> {
        self.raw_subscribers_of(node)
    }
}
