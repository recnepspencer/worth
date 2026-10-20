//! Admit both subscriber counts before creating subscription custody.
use super::{
    WorthQueryLiveDeliverySource, WorthQueryLiveProductKey, WorthQueryLiveProductPartition,
    WorthQueryLiveSubscription,
};
use std::collections::VecDeque;
use std::sync::{atomic::Ordering, Arc};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryLiveSubscriptionCapacityExhausted;

impl WorthQueryLiveDeliverySource {
    pub(in crate::domain_computation::primary_graph) fn open(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<WorthQueryLiveSubscription, WorthQueryLiveSubscriptionCapacityExhausted> {
        let key = WorthQueryLiveProductKey::from_observation(observation);
        let batch_capacity = self.batch_capacity.load(Ordering::Acquire);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let total = state
            .subscriber_count
            .checked_add(1)
            .ok_or(WorthQueryLiveSubscriptionCapacityExhausted)?;
        let subscribers = match state.partitions.get(&key) {
            Some(partition) => partition
                .subscriber_count
                .checked_add(1)
                .ok_or(WorthQueryLiveSubscriptionCapacityExhausted)?,
            None => 1,
        };
        let partition =
            state
                .partitions
                .entry(key.clone())
                .or_insert_with(|| WorthQueryLiveProductPartition {
                    batches: VecDeque::with_capacity(batch_capacity),
                    allocated_batch_capacity: batch_capacity,
                    next_sequence: 0,
                    last_product_commit: None,
                    subscriber_count: 0,
                    reservation_live: false,
                    retired: false,
                });
        partition.subscriber_count = subscribers;
        let cursor = partition.next_sequence;
        state.subscriber_count = total;
        Ok(WorthQueryLiveSubscription {
            state: Arc::clone(&self.state),
            key,
            cursor,
        })
    }
}
