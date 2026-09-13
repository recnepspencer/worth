use super::super::scratch::{ScratchLeaseKind, TraversalScratch};
use super::super::signal_graph::SignalGraph;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;

impl SignalGraph {
    pub fn unregister_node(&mut self, id: NodeId) -> Result<(), SignalError> {
        self.validate_handle(id)?;
        self.with_scratch(ScratchLeaseKind::Churn, |graph, scratch| {
            let scratch = scratch.traversal_mut();
            graph.collect_retired_node_adjacency(id, scratch)?;
            graph.sever_retired_upstream_links(id, &scratch.node_buffer_a)?;
            graph.repair_retired_downstream_topology(id, &scratch.node_buffer_b)?;
            graph.replace_reverse_subscriptions_for_consumer(id, &[])?;
            graph.release_pending_causes(id)?;
            graph.retire_node_slot(id)?;
            Ok(())
        })
    }

    fn collect_retired_node_adjacency(
        &mut self,
        id: NodeId,
        scratch: &mut TraversalScratch,
    ) -> Result<(), SignalError> {
        scratch.node_buffer_a.clear();
        scratch.node_buffer_b.clear();

        scratch.node_buffer_a.extend(
            self.runtime_dependencies_of(id)?
                .iter()
                .map(|edge| edge.source()),
        );
        scratch
            .node_buffer_b
            .extend(self.runtime_subscribers_of(id)?.iter().copied());
        Ok(())
    }

    fn sever_retired_upstream_links(
        &mut self,
        retired: NodeId,
        sources: &[NodeId],
    ) -> Result<(), SignalError> {
        self.reconcile_subscriber_sets(retired, sources, &[])
    }

    fn repair_retired_downstream_topology(
        &mut self,
        retired: NodeId,
        subscribers: &[NodeId],
    ) -> Result<(), SignalError> {
        for &subscriber in subscribers {
            if self.is_alive(subscriber) {
                let current = self.current_runtime_dependencies_of(subscriber)?.to_vec();
                let retained = current
                    .into_iter()
                    .filter(|edge| edge.source() != retired)
                    .collect::<Vec<_>>();
                self.set_dependency_edges_sorted(subscriber, &retained)?;
            }
        }
        Ok(())
    }

    fn retire_node_slot(&mut self, id: NodeId) -> Result<(), SignalError> {
        if self.get_entry(id)?.pending_cause_set_id()
            != crate::data::graph::storage::invalidation_causes::PendingCauseSetId::EMPTY
        {
            return Err(SignalError::invalid_input(
                "node retirement requires pending cause authority to be released",
            ));
        }
        debug_assert!(
            !self.arena.free_slots.contains(id.index() as usize),
            "free list already contained slot {} before unregister",
            id.index()
        );
        self.arena.nodes[id.index() as usize].vacate();
        self.conditional_dependency_versions.remove(&id);
        self.arena.active_nodes = self.arena.active_nodes.saturating_sub(1);
        self.arena.record_retired_node();
        if !self.arena.nodes[id.index() as usize].is_retired() {
            self.arena.free_list.push_back(id.index());
            self.arena.free_slots.mark(id.index() as usize);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn inject_retired_dependency_for_test(
        &mut self,
        node: NodeId,
        source: NodeId,
        aspect: crate::data::aspect::Aspect,
    ) -> Result<(), SignalError> {
        let current_sources = self.dependency_sources_of(node)?;
        self.reconcile_subscriber_sets(node, &current_sources, &[])?;
        let edge = self.build_dependency_edge(source, aspect, None);
        self.set_dependency_edges_sorted(node, &[edge])
    }

    #[cfg(test)]
    pub(crate) fn inject_retired_subscriber_for_test(
        &mut self,
        node: NodeId,
        subscriber: NodeId,
    ) -> Result<(), SignalError> {
        self.set_subscribers_sorted(node, &[subscriber])
    }
}
