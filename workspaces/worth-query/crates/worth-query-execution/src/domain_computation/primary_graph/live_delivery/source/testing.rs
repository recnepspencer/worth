use std::sync::atomic::Ordering;
use std::sync::Arc;

use super::WorthQueryLiveDeliverySource;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationEmission, WorthQueryCommittedProductPublication,
};

impl WorthQueryLiveDeliverySource {
    pub(crate) fn set_limits(&self, batch_capacity: usize, byte_capacity: u64) {
        self.batch_capacity.store(batch_capacity, Ordering::Release);
        self.byte_capacity.store(byte_capacity, Ordering::Release);
    }

    pub(crate) fn set_after_reservation_hook(&self, hook: Option<Arc<dyn Fn() + Send + Sync>>) {
        *self
            .after_reservation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = hook;
    }

    pub(crate) fn emissions(
        &self,
        publication: &WorthQueryCommittedProductPublication,
    ) -> Vec<WorthQueryApplicationEmission> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .partitions
            .values()
            .flat_map(|partition| partition.batches.iter())
            .find(|batch| batch.batch().publication == *publication)
            .map(|batch| batch.batch().emissions.clone())
            .unwrap_or_default()
    }

    pub(crate) fn published_commit_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .published_commit_count
    }

    pub(crate) fn retained_payload_bytes(&self) -> u64 {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .partitions
            .values()
            .map(|partition| partition.retained_payload_bytes)
            .sum()
    }

    pub(crate) fn active_partition_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .partitions
            .len()
    }
}
