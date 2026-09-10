use crate::data::error::SignalError;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;

mod index_publication;
mod publication_work;
mod retained_publication;
pub(crate) use index_publication::PreparedPendingRevalidationIndex;
pub(crate) use retained_publication::PreparedRetainedPendingRevalidationIndex;
mod resolution_preparation;
pub(crate) use resolution_preparation::{
    PendingRevalidationNodeProjection, PendingRevalidationPreparationDenial,
    PreparedPendingRevalidationResolution,
};

impl SignalGraph {
    pub(in crate::data::graph) fn replace_pending_revalidation_waiters(
        &mut self,
        consumer: NodeId,
        previous: &[NodeId],
        current: &[NodeId],
    ) {
        for producer in previous {
            if let Some(waiters) = self.topology.pending_revalidation_waiters.get_mut(producer) {
                waiters.remove(&consumer);
                if waiters.is_empty() {
                    self.topology.pending_revalidation_waiters.remove(producer);
                }
            }
        }
        for producer in current {
            self.topology
                .pending_revalidation_waiters
                .entry(*producer)
                .or_default()
                .insert(consumer);
        }
    }

    pub(crate) fn pending_revalidation_waiters(
        &mut self,
        producer: NodeId,
    ) -> Result<Vec<NodeId>, SignalError> {
        let Some(candidates) = self.topology.pending_revalidation_waiters.get(&producer) else {
            return Ok(Vec::new());
        };
        let mut current = Vec::new();
        let mut stale = Vec::new();
        for &consumer in candidates {
            if !self.is_alive(consumer) {
                stale.push(consumer);
                continue;
            }
            if self
                .pending_dependency_revalidation(consumer)?
                .is_some_and(|pending| pending.unresolved_producers().contains(&producer))
            {
                if current.is_empty() {
                    current.reserve_exact(candidates.len());
                }
                current.push(consumer);
            } else {
                stale.push(consumer);
            }
        }
        if !current.is_empty() && current.len().saturating_mul(2) < current.capacity() {
            current = current.into_boxed_slice().into_vec();
        }
        if stale.len() == candidates.len() {
            self.topology.pending_revalidation_waiters.remove(&producer);
        } else if !stale.is_empty() {
            let waiters = self
                .topology
                .pending_revalidation_waiters
                .get_mut(&producer)
                .expect("nonempty current waiter set must remain indexed");
            for consumer in stale {
                waiters.remove(&consumer);
            }
        }
        Ok(current)
    }

    pub(crate) fn rebuild_pending_revalidation_waiters(&mut self) -> Result<(), SignalError> {
        let mut rebuilt =
            crate::data::persistent_ord_map::PersistentOrdMap::<NodeId, im::OrdSet<NodeId>>::new();
        for consumer in self.live_node_ids() {
            let Some(pending) = self.pending_dependency_revalidation(consumer)? else {
                continue;
            };
            for producer in pending.unresolved_producers() {
                rebuilt.entry(*producer).or_default().insert(consumer);
            }
        }
        self.topology.pending_revalidation_waiters = rebuilt;
        Ok(())
    }
}

#[cfg(test)]
mod tests;

impl SignalGraph {
    /// Installs an already discovered outcome. No waiter traversal occurs here.
    /// The enclosing packet must reserve storage before its first publication.
    pub(crate) fn publish_pending_revalidation_resolution(
        &mut self,
        prepared: PreparedPendingRevalidationResolution,
    ) -> Result<(), SignalError> {
        let (nodes, index) = prepared.split_node_changes();
        for (node, projected) in nodes {
            self.publish_node_revalidation_resolution(node, projected)?;
        }
        index.publish(self);
        Ok(())
    }
}

pub(crate) mod preparation_work;
