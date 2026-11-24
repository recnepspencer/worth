//! Admit both subscriber counts before creating subscription custody.
use super::{
    WorthQueryLiveDeliverySource, WorthQueryLiveProductKey, WorthQueryLiveProductPartition,
    WorthQueryLiveSubscription,
};
use std::sync::{atomic::Ordering, Arc};

#[derive(Debug)]
pub struct WorthQueryLiveSubscriptionCapacityExhausted;

impl WorthQueryLiveDeliverySource {
    pub fn open(
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
        let partition = state
            .partitions
            .entry(key.clone())
            .or_insert_with(|| WorthQueryLiveProductPartition::new(batch_capacity, observation));
        partition.subscriber_count = subscribers;
        let cursor = partition.next_sequence;
        state.subscriber_count = total;
        Ok(WorthQueryLiveSubscription {
            state: Arc::clone(&self.state),
            key,
            cursor,
        })
    }

    /// Metered, commit-aligned live subscription opening for whole-result views.
    /// Rejects stale selected observations and charges partition/lookup work
    /// before admitting the subscriber.
    pub fn open_funded(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<WorthQueryLiveSubscription, super::WorthQueryLiveAdmissionDenial> {
        let key = WorthQueryLiveProductKey::from_observation(observation);
        let mut candidate = self.funded_partition(execution, observation)?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::lookup_work(execution, &state, &key)?;
        if state.closed {
            return Err("live delivery is closed".into());
        }
        let total = state
            .subscriber_count
            .checked_add(1)
            .ok_or("live subscriber count is exhausted")?;
        if let Some(partition) = state.partitions.get(&key) {
            if partition.retired || partition.reservation_live {
                return Err("whole-result live subscription cannot join this partition".into());
            }
            if partition.last_product_commit.as_ref() != Some(observation.selected_commit()) {
                return Err("whole-result live opening has a stale World image".into());
            }
            partition
                .subscriber_count
                .checked_add(1)
                .ok_or("live subscriber count is exhausted")?;
        }
        let _allocation = self.install_funded_partition(&mut state, &key, &mut candidate)?;
        let partition = state
            .partitions
            .get_mut(&key)
            .expect("actual opening partition");
        partition.subscriber_count += 1;
        let cursor = partition.next_sequence;
        state.subscriber_count = total;
        Ok(WorthQueryLiveSubscription {
            state: Arc::clone(&self.state),
            key,
            cursor,
        })
    }
}

pub mod court_doors_for_test;
