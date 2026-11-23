use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::data::graph::runtime::graph::{map_node_edit_accounting, map_node_edit_retention};
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::retained_storage::{
    RetainedStorageCharge, RetainedStoragePreparationDenial, SignalConditionalRetentionReservation,
};

use super::buckets::IndexedSubscriptionMembership;

pub(in crate::data::graph) struct PreparedReverseSubscriptionReplacement {
    consumer: NodeId,
    memberships: Vec<IndexedSubscriptionMembership>,
    _custody: Option<SignalConditionalRetentionReservation>,
    overlay_custody: Option<SignalConditionalRetentionReservation>,
}

impl PreparedReverseSubscriptionReplacement {
    pub(in crate::data::graph) fn publish(self, graph: &mut SignalGraph) {
        graph.topology.reverse_subscriptions.publish_replacement(
            self.consumer,
            self.memberships,
            self.overlay_custody,
        );
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
            true,
        )?
        .publish(self);
        Ok(())
    }

    pub(in crate::data::graph) fn prepare_reverse_subscription_replacement(
        &self,
        consumer: NodeId,
        edges: &[DependencyEdge],
        work: &mut crate::logic::evaluation::EvaluationWork<'_, '_>,
        first_in_batch: bool,
    ) -> Result<PreparedReverseSubscriptionReplacement, SignalError> {
        self.validate_handle(consumer)?;
        work.reserve(
            edges
                .len()
                .checked_mul(std::mem::size_of::<IndexedSubscriptionMembership>() + 1),
        )?;
        let charge = RetainedStorageCharge::capacity::<IndexedSubscriptionMembership>(edges.len())
            .map_err(map_node_edit_accounting)?;
        let custody = self.reserve_reverse_edit(charge)?;
        let overlay_custody = if first_in_batch {
            self.prepare_reverse_overlay_detach(work)?
        } else {
            None
        };
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
            overlay_custody,
        })
    }

    fn prepare_reverse_overlay_detach(
        &self,
        work: &mut crate::logic::evaluation::EvaluationWork<'_, '_>,
    ) -> Result<Option<SignalConditionalRetentionReservation>, SignalError> {
        let Some(charge) = self
            .topology
            .reverse_subscriptions
            .fork_edit_charge()
            .map_err(map_node_edit_accounting)?
        else {
            return Ok(None);
        };
        work.reserve(Some(usize::try_from(charge.bytes()).map_err(|_| {
            map_node_edit_accounting(RetainedStoragePreparationDenial::ChargeOverflow)
        })?))?;
        self.reserve_reverse_edit(charge)
    }

    fn reserve_reverse_edit(
        &self,
        charge: RetainedStorageCharge,
    ) -> Result<Option<SignalConditionalRetentionReservation>, SignalError> {
        self.arena
            .retained_node_ledger
            .as_ref()
            .map(|ledger| ledger.reserve(0, charge).map_err(map_node_edit_retention))
            .transpose()
    }
}
