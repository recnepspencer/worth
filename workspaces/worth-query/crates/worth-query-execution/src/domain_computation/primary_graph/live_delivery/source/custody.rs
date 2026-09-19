use std::sync::{Arc, Mutex};

use super::{
    WorthQueryLiveCommitBatchCell, WorthQueryLiveDeliverySourceState, WorthQueryLiveProductKey,
};

pub(in crate::domain_computation::primary_graph) struct WorthQueryLiveSubscription {
    pub(super) state: Arc<Mutex<WorthQueryLiveDeliverySourceState>>,
    pub(super) key: WorthQueryLiveProductKey,
    pub(super) cursor: u64,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryLivePublicationReservation {
    pub(super) state: Arc<Mutex<WorthQueryLiveDeliverySourceState>>,
    pub(super) key: WorthQueryLiveProductKey,
    pub(super) slot: Option<Arc<WorthQueryLiveCommitBatchCell>>,
    pub(super) retained_payload_bytes: u64,
    pub(super) evictions: usize,
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
            retained_payload_bytes: 0,
            evictions: 0,
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
        if let Some(partition) = state.partitions.get_mut(&self.key) {
            partition.reservation_live = false;
            if partition.subscriber_count == 0 {
                state.partitions.remove(&self.key);
            }
        }
        self.active = false;
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
        let remove = state
            .partitions
            .get_mut(&self.key)
            .is_some_and(|partition| {
                partition.subscriber_count = partition
                    .subscriber_count
                    .checked_sub(1)
                    .expect("live subscription releases exactly once");
                partition.subscriber_count == 0 && !partition.reservation_live
            });
        if remove {
            state.partitions.remove(&self.key);
        }
    }
}
