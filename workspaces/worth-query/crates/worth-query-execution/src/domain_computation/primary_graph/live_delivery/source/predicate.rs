//! Managed openings join the existing publication partition, never a second fence.
mod custody;
use super::*;
use worth_runtime_world::facade::{CompositeCommitIdentity, ProductBranchObservation};

pub(in crate::domain_computation::primary_graph) struct WorthQueryLivePredicatePreparation {
    state: Arc<Mutex<WorthQueryLiveDeliverySourceState>>,
    key: WorthQueryLiveProductKey,
    _allocation: Arc<SourceFactsAllocation>,
}
pub(in crate::domain_computation::primary_graph) struct WorthQueryLivePredicateOpening {
    state: Arc<Mutex<WorthQueryLiveDeliverySourceState>>,
    key: WorthQueryLiveProductKey,
    _allocation: Arc<SourceFactsAllocation>,
}
pub(in crate::domain_computation::primary_graph) struct WorthQueryLivePredicateSubscription {
    subscription: WorthQueryLiveSubscription,
    stamp: Stamp,
    allocation: Arc<SourceFactsAllocation>,
}
#[derive(PartialEq, Eq)]
struct Stamp {
    completed_commit: Option<CompositeCommitIdentity>,
    sequence: u64,
    resync_epoch: u64,
}
impl Stamp {
    fn of(partition: &WorthQueryLiveProductPartition) -> Self {
        Self {
            completed_commit: partition.last_product_commit.clone(),
            sequence: partition.next_sequence,
            resync_epoch: partition.resync_epoch,
        }
    }
}
impl WorthQueryLiveDeliverySource {
    pub(in crate::domain_computation::primary_graph) fn begin_predicate_preparation(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        observation: &ProductBranchObservation,
    ) -> Result<WorthQueryLivePredicatePreparation, WorthQueryLiveAdmissionDenial> {
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
        if let Some(partition) = state.partitions.get(&key) {
            if partition.retired || partition.opening_count != 0 || partition.reservation_live {
                return Err("managed predicate preparation cannot join this partition".into());
            }
            if partition.last_product_commit.as_ref() != Some(observation.selected_commit()) {
                return Err("managed predicate preparation has a stale World image".into());
            }
            partition
                .preparation_count
                .checked_add(1)
                .ok_or("managed preparation count is exhausted")?;
        }
        let allocation = self.install_funded_partition(&mut state, &key, &mut candidate)?;
        let partition = state
            .partitions
            .get_mut(&key)
            .expect("actual prepared partition");
        partition.preparation_count += 1;
        Ok(WorthQueryLivePredicatePreparation {
            state: self.state.clone(),
            key,
            _allocation: allocation,
        })
    }
    /// Call before the Parent read, then re-observe actual World currentness.
    pub(in crate::domain_computation::primary_graph) fn open_predicate(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        observation: &ProductBranchObservation,
    ) -> Result<WorthQueryLivePredicateSubscription, WorthQueryLiveAdmissionDenial> {
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
            if partition.retired || partition.preparation_count != 0 || partition.reservation_live {
                return Err("managed opening cannot read an image being prepared".into());
            }
            if partition.last_product_commit.as_ref() != Some(observation.selected_commit()) {
                return Err("managed opening has a stale World image".into());
            }
            partition
                .subscriber_count
                .checked_add(1)
                .ok_or("live subscriber count is exhausted")?;
        }
        let allocation = self.install_funded_partition(&mut state, &key, &mut candidate)?;
        let partition = state
            .partitions
            .get_mut(&key)
            .expect("actual opening partition");
        partition.subscriber_count += 1;
        let stamp = Stamp::of(partition);
        let subscription = WorthQueryLiveSubscription {
            state: self.state.clone(),
            key,
            cursor: stamp.sequence,
        };
        state.subscriber_count = total;
        Ok(WorthQueryLivePredicateSubscription {
            subscription,
            stamp,
            allocation,
        })
    }
    /// Only this atomic join permits installing the read's predicate record.
    pub(in crate::domain_computation::primary_graph) fn join_predicate_opening(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        subscription: &WorthQueryLivePredicateSubscription,
        observation: &ProductBranchObservation,
    ) -> Result<WorthQueryLivePredicateOpening, WorthQueryLiveAdmissionDenial> {
        if !Arc::ptr_eq(&self.state, &subscription.subscription.state)
            || subscription.subscription.key
                != WorthQueryLiveProductKey::from_observation(observation)
        {
            return Err("managed opening belongs to another Live occurrence".into());
        }
        let key = &subscription.subscription.key;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::lookup_work(execution, &state, key)?;
        if state.closed {
            return Err("live delivery is closed".into());
        }
        let partition = state
            .partitions
            .get_mut(key)
            .ok_or("managed opening partition is unavailable")?;
        if partition.retired
            || partition.preparation_count != 0
            || partition.reservation_live
            || Stamp::of(partition) != subscription.stamp
            || partition.last_product_commit.as_ref() != Some(observation.selected_commit())
        {
            return Err("managed opening lost its selected-image join".into());
        }
        partition.opening_count = partition
            .opening_count
            .checked_add(1)
            .ok_or("managed opening count is exhausted")?;
        Ok(WorthQueryLivePredicateOpening {
            state: self.state.clone(),
            key: key.clone(),
            _allocation: subscription.allocation.clone(),
        })
    }
    pub(in crate::domain_computation::primary_graph) fn admits_predicate_occurrence(
        &self,
        subscription: &WorthQueryLivePredicateSubscription,
        current: &ProductBranchObservation,
    ) -> bool {
        Arc::ptr_eq(&self.state, &subscription.subscription.state)
            && subscription.subscription.key == WorthQueryLiveProductKey::from_observation(current)
    }
    pub(in crate::domain_computation::primary_graph) fn validate_predicate_progress(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        subscription: &WorthQueryLivePredicateSubscription,
        current: &ProductBranchObservation,
    ) -> Result<(), WorthQueryLiveAdmissionDenial> {
        if !self.admits_predicate_occurrence(subscription, current) {
            return Err("managed watch belongs to another Live occurrence".into());
        }
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::lookup_work(execution, &state, &subscription.subscription.key)?;
        if state.closed {
            return Err("live delivery is closed".into());
        }
        let partition = state
            .partitions
            .get(&subscription.subscription.key)
            .ok_or("managed watch partition is unavailable")?;
        if partition.retired
            || partition.resync_epoch != subscription.stamp.resync_epoch
            || partition.last_product_commit.as_ref() != Some(current.selected_commit())
        {
            return Err("managed watch requires resynchronization".into());
        }
        Ok(())
    }
    /// Owner court injects an untracked movement; production publishes exact predicate notices.
    pub fn invalidate_predicates(&self, observation: &ProductBranchObservation) {
        let key = WorthQueryLiveProductKey::from_observation(observation);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(partition) = state.partitions.get_mut(&key) {
            if let Some(next) = partition.resync_epoch.checked_add(1) {
                partition.resync_epoch = next;
            } else {
                partition.retired = true;
            }
        }
    }
}
impl WorthQueryLivePredicateSubscription {
    pub(in crate::domain_computation::primary_graph) fn subscription(
        &self,
    ) -> &WorthQueryLiveSubscription {
        &self.subscription
    }
}

mod court_doors_for_test;
