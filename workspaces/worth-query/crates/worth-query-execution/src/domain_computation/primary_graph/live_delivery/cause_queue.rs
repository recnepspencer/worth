use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_declaration::facade::application_schema::{
    ApplicationEffectPayload, ApplicationEffectRef,
};
use worth_runtime_bridge::facade::BridgeManagedQueueOccupancy;

use super::{
    WorthQueryLiveCommitBatchCell, WorthQueryLiveDeliverySource, WorthQueryLiveSourcePoll,
    WorthQueryLiveSubscription,
};
use crate::domain_computation::managed_run::WorthQueryManagedLowerExecutionBasis;

pub(in crate::domain_computation::primary_graph) struct WorthQueryLiveCauseQueue<Payload> {
    subscription: WorthQueryLiveSubscription,
    cursor: u64,
    active_batch: Option<WorthQueryActiveLiveBatch>,
    pending: VecDeque<WorthQueryBufferedLiveCause>,
    _payload: PhantomData<fn() -> Payload>,
}

struct WorthQueryActiveLiveBatch {
    batch: Arc<WorthQueryLiveCommitBatchCell>,
    next_emission: usize,
}

struct WorthQueryBufferedLiveCause {
    batch: Arc<WorthQueryLiveCommitBatchCell>,
    emission: usize,
    occupancy: BridgeManagedQueueOccupancy,
}

pub(in crate::domain_computation::primary_graph) enum WorthQueryLiveCauseFillPosture {
    Pending,
    Overflow(u64),
    Closed,
    Unavailable,
}

impl<Payload> WorthQueryLiveCauseQueue<Payload> {
    pub(in crate::domain_computation::primary_graph) fn open(
        source: &WorthQueryLiveDeliverySource,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        capacity: usize,
    ) -> Self {
        let subscription = source.open(observation);
        let cursor = subscription.cursor();
        Self {
            subscription,
            cursor,
            active_batch: None,
            pending: VecDeque::with_capacity(capacity),
            _payload: PhantomData,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn buffered_cause_count(&self) -> usize {
        self.pending.len()
    }

    pub(in crate::domain_computation::primary_graph) fn front<Schema, Effect>(
        &self,
        effect: &ApplicationEffectRef<Schema, Effect, Payload>,
    ) -> Option<(
        &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
        &crate::basis::WorthQueryProductObservationLease,
        &Payload,
    )>
    where
        Payload: ApplicationEffectPayload,
    {
        let cause = self.pending.front()?;
        let batch = cause.batch.batch();
        let payload = batch.emissions.get(cause.emission)?.payload_ref(effect)?;
        Some((&batch.publication, &batch.product, payload))
    }

    pub(in crate::domain_computation::primary_graph) fn acknowledge_front(
        &mut self,
        basis: &mut WorthQueryManagedLowerExecutionBasis,
    ) -> Result<(), ()> {
        let Some(mut cause) = self.pending.pop_front() else {
            return Ok(());
        };
        if let Err(failure) = basis
            .bridge
            .release_managed_queue_occupancy(cause.occupancy)
        {
            cause.occupancy = failure.into_occupancy();
            self.pending.push_front(cause);
            return Err(());
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn release_all(
        &mut self,
        basis: &mut WorthQueryManagedLowerExecutionBasis,
    ) -> Result<(), ()> {
        while let Some(mut cause) = self.pending.pop_front() {
            if let Err(failure) = basis
                .bridge
                .release_managed_queue_occupancy(cause.occupancy)
            {
                cause.occupancy = failure.into_occupancy();
                self.pending.push_front(cause);
                return Err(());
            }
        }
        Ok(())
    }
}

impl<Payload> WorthQueryLiveCauseQueue<Payload>
where
    Payload: ApplicationEffectPayload,
{
    pub(in crate::domain_computation::primary_graph) fn fill<Schema, Effect>(
        &mut self,
        source: &WorthQueryLiveDeliverySource,
        basis: &mut WorthQueryManagedLowerExecutionBasis,
        effect: ApplicationEffectRef<Schema, Effect, Payload>,
        capacity: usize,
        mut admits_payload: impl FnMut(&Payload) -> bool,
    ) -> WorthQueryLiveCauseFillPosture {
        let mut terminal = WorthQueryLiveCauseFillPosture::Pending;
        while self.pending.len() < capacity {
            if self.active_batch.is_none() {
                let batch = match source.poll(&self.subscription, self.cursor) {
                    WorthQueryLiveSourcePoll::Batch(batch) => batch,
                    WorthQueryLiveSourcePoll::Pending => break,
                    WorthQueryLiveSourcePoll::Overflow { missed } => {
                        terminal = WorthQueryLiveCauseFillPosture::Overflow(missed);
                        break;
                    }
                    WorthQueryLiveSourcePoll::Closed => {
                        terminal = WorthQueryLiveCauseFillPosture::Closed;
                        break;
                    }
                };
                self.active_batch = Some(WorthQueryActiveLiveBatch {
                    batch,
                    next_emission: 0,
                });
            }
            let active = self
                .active_batch
                .as_mut()
                .expect("source batch installed above");
            let batch = active.batch.batch();
            let Some(emission) = batch.emissions.get(active.next_emission) else {
                self.cursor = batch
                    .sequence
                    .checked_add(1)
                    .expect("live sequence space is exhausted");
                self.active_batch = None;
                continue;
            };
            let emission_index = active.next_emission;
            active.next_emission += 1;
            let Some(payload) = emission
                .payload_ref(&effect)
                .filter(|value| admits_payload(value))
            else {
                continue;
            };
            let _ = payload;
            let admission = match basis.bridge.enqueue_managed_queue(1) {
                Ok(admission) => admission,
                Err(_) => return WorthQueryLiveCauseFillPosture::Unavailable,
            };
            let (_, occupancy) = admission.into_parts();
            self.pending.push_back(WorthQueryBufferedLiveCause {
                batch: Arc::clone(&active.batch),
                emission: emission_index,
                occupancy,
            });
        }
        terminal
    }
}
