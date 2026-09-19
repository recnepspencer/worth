//! Prepared waiter index writes, independent of node payload installation.
use std::collections::BTreeMap;

use super::{PendingRevalidationNodeProjection, PreparedPendingRevalidationResolution};
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;

#[derive(Debug)]
pub(crate) struct PreparedPendingRevalidationIndex {
    pub(super) buckets: BTreeMap<NodeId, im::OrdSet<NodeId>>,
}

impl PreparedPendingRevalidationResolution {
    pub(crate) fn split_node_changes(
        self,
    ) -> (
        BTreeMap<NodeId, PendingRevalidationNodeProjection>,
        super::PreparedPendingRevalidationIndex,
    ) {
        (
            self.nodes,
            PreparedPendingRevalidationIndex {
                buckets: self.buckets,
            },
        )
    }
}

impl PreparedPendingRevalidationIndex {
    pub(in crate::data::graph) fn for_replacement(
        graph: &SignalGraph,
        consumer: NodeId,
        previous: &[NodeId],
        current: &[NodeId],
    ) -> Self {
        let mut buckets = BTreeMap::new();
        for &producer in previous.iter().chain(current) {
            if buckets.contains_key(&producer) {
                continue;
            }
            let mut waiters = graph
                .topology
                .pending_revalidation_waiters
                .get(&producer)
                .cloned()
                .unwrap_or_default();
            if current.contains(&producer) {
                waiters.insert(consumer);
            } else {
                waiters.remove(&consumer);
            }
            buckets.insert(producer, waiters);
        }
        Self { buckets }
    }

    /// Storage work is admitted by the enclosing output packet before any write.
    pub(crate) fn publish(self, graph: &mut SignalGraph) {
        for (producer, waiters) in self.buckets {
            if waiters.is_empty() {
                graph
                    .topology
                    .pending_revalidation_waiters
                    .remove(&producer);
            } else {
                graph
                    .topology
                    .pending_revalidation_waiters
                    .insert(producer, waiters);
            }
        }
    }
}
