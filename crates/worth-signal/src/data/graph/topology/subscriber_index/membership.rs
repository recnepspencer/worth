use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;

use super::buckets::IndexedSubscriptionMembership;

pub(in crate::data::graph) struct PreparedReverseSubscriptionReplacement {
    consumer: NodeId,
    memberships: Vec<IndexedSubscriptionMembership>,
    _custody: Option<crate::data::retained_storage::SignalConditionalRetentionReservation>,
}

impl PreparedReverseSubscriptionReplacement {
    pub(in crate::data::graph) fn publish(self, graph: &mut SignalGraph) {
        graph
            .topology
            .reverse_subscriptions
            .replace_consumer(self.consumer, self.memberships);
    }
}

impl SignalGraph {
    pub(in crate::data::graph) fn replace_reverse_subscriptions_for_consumer(
        &mut self,
        consumer: NodeId,
        edges: &[DependencyEdge],
    ) -> Result<(), SignalError> {
        self.prepare_reverse_subscription_replacement(
            consumer,
            edges,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )?
        .publish(self);
        Ok(())
    }

    pub(in crate::data::graph) fn prepare_reverse_subscription_replacement(
        &self,
        consumer: NodeId,
        edges: &[DependencyEdge],
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<PreparedReverseSubscriptionReplacement, SignalError> {
        self.validate_handle(consumer)?;
        work.reserve(
            edges
                .len()
                .checked_mul(std::mem::size_of::<IndexedSubscriptionMembership>() + 1),
        )?;
        let charge = crate::data::retained_storage::RetainedStorageCharge::capacity::<
            IndexedSubscriptionMembership,
        >(edges.len())
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        let custody = self
            .arena
            .retained_node_ledger
            .as_ref()
            .map(|ledger| {
                ledger
                    .reserve(0, charge)
                    .map_err(crate::data::graph::runtime::graph::map_node_edit_retention)
            })
            .transpose()?;
        let mut memberships = Vec::with_capacity(edges.len());
        for edge in edges {
            let membership = IndexedSubscriptionMembership::from_edge(
                edge.source(),
                edge.aspect(),
                edge.interned_scope(),
            )
            .ok_or_else(|| {
                SignalError::internal(
                    "scoped dependency reached reverse indexing without interned scope",
                )
            })?;
            memberships.push(membership);
        }
        Ok(PreparedReverseSubscriptionReplacement {
            consumer,
            memberships,
            _custody: custody,
        })
    }
}
