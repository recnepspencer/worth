//! Node changes derived by direct-cause admission, separate from cause-store writes.
use crate::data::error::SignalError;
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::data::graph::storage::PreparedInvalidationCache;
use crate::data::graph::PreparedPendingRevalidationResolution;
#[cfg(test)]
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;

#[derive(Debug)]
pub(super) struct PreparedCauseNodeReplacement {
    pub(super) consumer: NodeId,
    pub(super) cause_set: PendingCauseSetId,
    pub(super) cache: PreparedInvalidationCache,
}

#[derive(Debug)]
pub(crate) struct PreparedDirectCauseNodes {
    pub(super) replacements: Vec<PreparedCauseNodeReplacement>,
    pub(super) waiters: PreparedPendingRevalidationResolution,
}

impl PreparedDirectCauseNodes {
    pub(crate) fn selected_nodes(&self) -> impl ExactSizeIterator<Item = NodeId> + '_ {
        self.waiters.nodes.keys().copied()
    }

    #[cfg(test)]
    pub(super) fn publish(self, graph: &mut SignalGraph) -> Result<(), SignalError> {
        let index = self.visit_node_changes(|node, cause, projected| {
            if let Some((cause_set, cache)) = cause {
                graph.publish_node_cause_resolution(node, cause_set, cache, projected)
            } else {
                graph.publish_node_revalidation_resolution(node, projected)
            }
        })?;
        index.publish(graph);
        Ok(())
    }

    pub(crate) fn visit_node_changes(
        self,
        mut publish: impl FnMut(
            NodeId,
            Option<(PendingCauseSetId, PreparedInvalidationCache)>,
            crate::data::graph::PendingRevalidationNodeProjection,
        ) -> Result<(), SignalError>,
    ) -> Result<crate::data::graph::PreparedPendingRevalidationIndex, SignalError> {
        let (nodes, index) = self.waiters.split_node_changes();
        let mut replacements = self.replacements.into_iter().peekable();
        for (node, projected) in nodes {
            if replacements
                .peek()
                .is_some_and(|next| next.consumer == node)
            {
                let replacement = replacements.next().expect("matching prepared consumer");
                publish(
                    node,
                    Some((replacement.cause_set, replacement.cache)),
                    projected,
                )?;
            } else {
                publish(node, None, projected)?;
            }
        }
        assert!(
            replacements.next().is_none(),
            "every direct consumer has a waiter projection"
        );
        Ok(index)
    }
}
