use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::sync::{Arc, Mutex};

use super::{
    WorthQueryLiveCommitBatchCell, WorthQueryLiveDeliverySourceState, WorthQueryLiveProductKey,
};

pub struct WorthQueryLiveSubscription {
    pub(super) state: Arc<Mutex<WorthQueryLiveDeliverySourceState>>,
    pub(super) key: WorthQueryLiveProductKey,
    pub(super) cursor: u64,
}

pub struct WorthQueryLivePublicationReservation {
    pub(super) state: Arc<Mutex<WorthQueryLiveDeliverySourceState>>,
    pub(super) key: WorthQueryLiveProductKey,
    pub(super) slot: Option<Arc<WorthQueryLiveCommitBatchCell>>,
    pub(super) evictions: usize,
    pub(super) retired_batches: Vec<Arc<WorthQueryLiveCommitBatchCell>>,
    pub(super) _allocation: Option<Arc<SourceFactsAllocation>>,
    pub(super) active: bool,
}

impl WorthQueryLivePublicationReservation {
    pub(super) fn inactive(
        state: Arc<Mutex<WorthQueryLiveDeliverySourceState>>,
        key: WorthQueryLiveProductKey,
    ) -> Self {
        Self {
            state,
            key,
            slot: None,
            evictions: 0,
            retired_batches: Vec::new(),
            _allocation: None,
            active: false,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn requires_successor_observation(
        &self,
    ) -> bool {
        self.active
    }
}

impl Drop for WorthQueryLivePublicationReservation {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let remove = if let Some(partition) = state.partitions.get_mut(&self.key) {
            partition.reservation_live = false;
            partition.is_unowned()
        } else {
            false
        };
        let retired = remove.then(|| state.remove_partition(&self.key));
        self.active = false;
        drop(state);
        drop(retired);
    }
}

impl WorthQueryLiveSubscription {
    pub(in crate::domain_computation::primary_graph) const fn cursor(&self) -> u64 {
        self.cursor
    }
}

impl Drop for WorthQueryLiveSubscription {
    fn drop(&mut self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut released = false;
        let remove = state
            .partitions
            .get_mut(&self.key)
            .is_some_and(|partition| {
                released = true;
                partition.subscriber_count = partition.subscriber_count.saturating_sub(1);
                partition.is_unowned()
            });
        let retired = remove.then(|| state.remove_partition(&self.key));
        if released {
            state.subscriber_count = state.subscriber_count.saturating_sub(1);
        }
        drop(state);
        drop(retired);
    }
}
