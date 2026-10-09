#[cfg(any(test, debug_assertions))]
use crate::data::error::SignalError;
use crate::data::handle::NodeId;

use crate::data::graph::signal_graph::SignalGraph;

impl SignalGraph {
    #[cfg(debug_assertions)]
    pub(crate) fn topology_debug_asserts_enabled(&self) -> bool {
        topology_debug_asserts_enabled()
    }

    #[cfg(any(test, debug_assertions))]
    pub(crate) fn assert_bidirectional_consistency(&self) -> Result<(), SignalError> {
        let mut visits = ConsistencyEdgeVisits::default();
        for (index, slot) in self.arena.nodes.iter().enumerate() {
            if slot.is_occupied() {
                self.assert_node_bidirectional_consistency(
                    NodeId::new(index as u32, slot.generation),
                    &mut visits,
                )?;
            }
        }
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    pub(crate) fn assert_bidirectional_consistency_at(
        &self,
        node: NodeId,
        former_sources: &[NodeId],
        desired_sources: &[NodeId],
    ) -> Result<(), SignalError> {
        self.assert_reconciled_node_consistency(
            node,
            former_sources,
            desired_sources,
            &mut ConsistencyEdgeVisits::default(),
        )
    }

    #[inline]
    pub(crate) fn debug_assert_bidirectional_consistency_at(
        &self,
        _node: NodeId,
        _former_sources: &[NodeId],
        _desired_sources: &[NodeId],
    ) {
        #[cfg(debug_assertions)]
        if topology_debug_asserts_enabled() {
            self.assert_bidirectional_consistency_at(_node, _former_sources, _desired_sources)
                .expect("reconciled topology should remain bidirectionally consistent");
        }
    }

    #[cfg(debug_assertions)]
    #[inline]
    pub(crate) fn debug_assert_bidirectional_consistency_for_nodes(&self, _nodes: &[NodeId]) {
        #[cfg(debug_assertions)]
        if topology_debug_asserts_enabled() {
            for &node in _nodes {
                self.assert_node_bidirectional_consistency(
                    node,
                    &mut ConsistencyEdgeVisits::default(),
                )
                .expect("repaired topology should remain bidirectionally consistent");
            }
        }
    }

    #[inline]
    pub(in crate::data::graph::topology) fn debug_assert_rebuilt_subscriber_index(&self) {
        #[cfg(debug_assertions)]
        if topology_debug_asserts_enabled() {
            self.assert_bidirectional_consistency()
                .expect("rebuilt topology should remain bidirectionally consistent");
        }
    }

    #[cfg(any(test, debug_assertions))]
    fn assert_reconciled_node_consistency(
        &self,
        node: NodeId,
        former_sources: &[NodeId],
        desired_sources: &[NodeId],
        visits: &mut ConsistencyEdgeVisits,
    ) -> Result<(), SignalError> {
        self.assert_node_bidirectional_consistency(node, visits)?;
        for &source in former_sources.iter().chain(desired_sources) {
            if self.is_alive(source) && self.subscriber_membership(source, node, visits)? {
                self.assert_subscriber_dependency(source, node, visits)?;
            }
        }
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn assert_node_bidirectional_consistency(
        &self,
        node: NodeId,
        visits: &mut ConsistencyEdgeVisits,
    ) -> Result<(), SignalError> {
        if !self.is_alive(node) {
            return Ok(());
        }
        if self.get_entry(node)?.is_tombstoned() {
            return Ok(());
        }
        for edge in self.raw_dependencies_of(node)? {
            self.assert_dependency_subscriber(edge.source(), node, visits)?;
        }
        for &subscriber in self.raw_subscribers_of(node)? {
            visits.edge();
            self.assert_subscriber_dependency(node, subscriber, visits)?;
        }
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn assert_subscriber_dependency(
        &self,
        source: NodeId,
        subscriber: NodeId,
        visits: &mut ConsistencyEdgeVisits,
    ) -> Result<(), SignalError> {
        if self.is_alive(subscriber)
            && !self.raw_dependencies_of(subscriber)?.iter().any(|edge| {
                visits.edge();
                edge.source() == source
            })
        {
            return Err(missing_dependency_edge(source, subscriber));
        }
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn assert_dependency_subscriber(
        &self,
        source: NodeId,
        node: NodeId,
        visits: &mut ConsistencyEdgeVisits,
    ) -> Result<(), SignalError> {
        visits.edge();
        if self.is_alive(source) && !self.subscriber_membership(source, node, visits)? {
            return Err(SignalError::internal(format!(
                "topology inconsistency: missing subscriber edge {source} -> {node}"
            )));
        }
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn subscriber_membership(
        &self,
        source: NodeId,
        node: NodeId,
        visits: &mut ConsistencyEdgeVisits,
    ) -> Result<bool, SignalError> {
        Ok(self.raw_subscribers_of(source)?.iter().any(|subscriber| {
            visits.edge();
            *subscriber == node
        }))
    }

    #[cfg(test)]
    pub(crate) fn bidirectional_consistency_visits_at_for_test(
        &self,
        node: NodeId,
        former_sources: &[NodeId],
        desired_sources: &[NodeId],
    ) -> Result<usize, SignalError> {
        let mut visits = ConsistencyEdgeVisits::default();
        self.assert_reconciled_node_consistency(
            node,
            former_sources,
            desired_sources,
            &mut visits,
        )?;
        Ok(visits.edges)
    }
}

#[cfg(any(test, debug_assertions))]
#[derive(Default)]
struct ConsistencyEdgeVisits {
    #[cfg(test)]
    edges: usize,
}

#[cfg(any(test, debug_assertions))]
impl ConsistencyEdgeVisits {
    #[inline]
    fn edge(&mut self) {
        #[cfg(test)]
        {
            self.edges += 1;
        }
    }
}

#[cfg(any(test, debug_assertions))]
fn missing_dependency_edge(source: NodeId, subscriber: NodeId) -> SignalError {
    SignalError::internal(format!(
        "topology inconsistency: missing dependency edge {source} -> {subscriber}"
    ))
}

#[cfg(debug_assertions)]
fn topology_debug_asserts_enabled() -> bool {
    #[cfg(test)]
    {
        if TOPOLOGY_CHECK_SKIPS.with(std::cell::Cell::get) > 0 {
            return false;
        }
    }
    std::env::var_os("WORTH_SIGNAL_SKIP_TOPOLOGY_DEBUG_ASSERTS").is_none()
}

#[cfg(test)]
thread_local! {
    static TOPOLOGY_CHECK_SKIPS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
impl SignalGraph {
    /// Skips the calling thread's automatic topology checks while `run` executes.
    /// Tests use this instead of the process environment, which other tests read.
    pub(crate) fn with_topology_debug_asserts_skipped<T>(run: impl FnOnce() -> T) -> T {
        struct Restore;
        impl Drop for Restore {
            fn drop(&mut self) {
                TOPOLOGY_CHECK_SKIPS.with(|skips| skips.set(skips.get() - 1));
            }
        }
        TOPOLOGY_CHECK_SKIPS.with(|skips| skips.set(skips.get() + 1));
        let _restore = Restore;
        run()
    }
}
