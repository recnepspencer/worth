//! One pending whole-Parent notification uses the existing Live occupancy/ACK owner.
use super::{
    WorthQueryLiveCommitBatchCell, WorthQueryLiveDeliverySource,
    WorthQueryLivePredicateSubscription, WorthQueryLiveSourcePoll,
};
use crate::domain_computation::{
    managed_run::WorthQueryManagedLowerExecutionBasis,
    primary_graph::{
        application_query::{derived_view::PredicateNoticeDisposition, WatchSlot},
        source_facts::SourceFactsAllocation,
        WorthQueryCommittedProductPublication,
    },
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_runtime_bridge::facade::BridgeManagedQueueOccupancy;

pub(in crate::domain_computation::primary_graph) struct WorthQueryLivePredicateQueue {
    subscription: WorthQueryLivePredicateSubscription,
    cursor: u64,
    pending: Option<Pending>,
}
struct Pending {
    batch: Arc<WorthQueryLiveCommitBatchCell>,
    disposition: PredicateNoticeDisposition,
    occupancy: BridgeManagedQueueOccupancy,
}
pub(in crate::domain_computation::primary_graph) enum WorthQueryLivePredicatePoll {
    Ready,
    Pending,
    Overflow(u64),
    Closed,
    Unavailable,
}
impl WorthQueryLivePredicateQueue {
    pub(in crate::domain_computation::primary_graph) fn new(
        subscription: WorthQueryLivePredicateSubscription,
    ) -> Self {
        let cursor = subscription.subscription().cursor();
        Self {
            subscription,
            cursor,
            pending: None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn subscription(
        &self,
    ) -> &WorthQueryLivePredicateSubscription {
        &self.subscription
    }
    pub(in crate::domain_computation::primary_graph) fn front(
        &self,
    ) -> Option<(
        &WorthQueryCommittedProductPublication,
        PredicateNoticeDisposition,
    )> {
        self.pending
            .as_ref()
            .map(|pending| (&pending.batch.batch().publication, pending.disposition))
    }
    pub(in crate::domain_computation::primary_graph) fn fill(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        source: &WorthQueryLiveDeliverySource,
        basis: &mut WorthQueryManagedLowerExecutionBasis,
        slot: WatchSlot,
    ) -> Result<
        WorthQueryLivePredicatePoll,
        worth_relational::facade::transactions::RelationalExecutionDenialCause,
    > {
        if self.pending.is_some() {
            return Ok(WorthQueryLivePredicatePoll::Ready);
        }
        loop {
            SourceFactsAllocation::work(execution, 1)?;
            let batch = match source.poll(self.subscription.subscription(), self.cursor) {
                WorthQueryLiveSourcePoll::Batch(batch) => batch,
                WorthQueryLiveSourcePoll::Pending => {
                    return Ok(WorthQueryLivePredicatePoll::Pending)
                }
                WorthQueryLiveSourcePoll::Overflow { missed } => {
                    return Ok(WorthQueryLivePredicatePoll::Overflow(missed))
                }
                WorthQueryLiveSourcePoll::Closed => return Ok(WorthQueryLivePredicatePoll::Closed),
            };
            let disposition = if let Some(notices) = &batch.batch().predicate {
                notices.disposition(execution, slot)?
            } else {
                None
            };
            if let Some(disposition) = disposition {
                let admission = match basis.bridge.enqueue_managed_queue(1) {
                    Ok(admission) => admission,
                    Err(_) => return Ok(WorthQueryLivePredicatePoll::Unavailable),
                };
                let (_, occupancy) = admission.into_parts();
                self.pending = Some(Pending {
                    batch,
                    disposition,
                    occupancy,
                });
                return Ok(WorthQueryLivePredicatePoll::Ready);
            }
            self.cursor = batch
                .batch()
                .sequence
                .checked_add(1)
                .expect("publication reserved sequence capacity");
        }
    }
    pub(in crate::domain_computation::primary_graph) fn acknowledge(
        &mut self,
        basis: &mut WorthQueryManagedLowerExecutionBasis,
    ) -> Result<(), ()> {
        let Some(mut pending) = self.pending.take() else {
            return Ok(());
        };
        match basis
            .bridge
            .release_managed_queue_occupancy(pending.occupancy)
        {
            Ok(_) => {
                self.cursor = pending
                    .batch
                    .batch()
                    .sequence
                    .checked_add(1)
                    .expect("publication reserved sequence capacity");
                Ok(())
            }
            Err(failure) => {
                pending.occupancy = failure.into_occupancy();
                self.pending = Some(pending);
                Err(())
            }
        }
    }
}
