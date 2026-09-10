use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

mod custody;
mod reservation;
#[cfg(test)]
mod testing;

pub(in crate::domain_computation::primary_graph) use custody::{
    WorthQueryLivePublicationReservation, WorthQueryLiveSubscription,
};

use super::super::application_attempt::{
    WorthQueryAdmittedApplicationEmissionBatch, WorthQueryApplicationEmission,
};
use crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication;

const RETAINED_COMMIT_BATCH_CAPACITY: usize = 64;
const RETAINED_COMMIT_BATCH_BYTES: u64 = 262_144;
const RETAINED_DELIVERY_BYTES: u64 =
    RETAINED_COMMIT_BATCH_BYTES * RETAINED_COMMIT_BATCH_CAPACITY as u64;

#[derive(Clone, Eq, Hash, PartialEq)]
struct WorthQueryLiveProductKey {
    branch: worth_runtime_world::facade::ProductBranchIdentity,
    incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
}

impl WorthQueryLiveProductKey {
    fn from_observation(
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Self {
        Self {
            branch: observation.branch_identity().clone(),
            incarnation: observation.lifecycle_incarnation(),
        }
    }

    fn admits(&self, publication: &WorthQueryCommittedProductPublication) -> bool {
        self.branch == *publication.product_branch()
            && self.incarnation == publication.product_incarnation()
    }
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryLiveDeliverySource {
    state: Arc<Mutex<WorthQueryLiveDeliverySourceState>>,
    batch_capacity: AtomicUsize,
    byte_capacity: AtomicU64,
    #[cfg(test)]
    after_reservation: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

#[derive(Default)]
struct WorthQueryLiveDeliverySourceState {
    partitions: HashMap<WorthQueryLiveProductKey, WorthQueryLiveProductPartition>,
    published_commit_count: usize,
    closed: bool,
}

struct WorthQueryLiveProductPartition {
    batches: VecDeque<Arc<WorthQueryLiveCommitBatchCell>>,
    allocated_batch_capacity: usize,
    next_sequence: u64,
    last_product_commit: Option<worth_runtime_world::facade::CompositeCommitIdentity>,
    retained_payload_bytes: u64,
    subscriber_count: usize,
    reservation_live: bool,
    retired: bool,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryLiveCommitBatch {
    pub(super) sequence: u64,
    pub(super) publication: WorthQueryCommittedProductPublication,
    pub(super) product: crate::basis::WorthQueryProductObservationLease,
    pub(super) emissions: Vec<WorthQueryApplicationEmission>,
    retained_payload_bytes: u64,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryLiveCommitBatchCell(
    OnceLock<WorthQueryLiveCommitBatch>,
);

impl WorthQueryLiveCommitBatchCell {
    pub(super) fn batch(&self) -> &WorthQueryLiveCommitBatch {
        self.0.get().expect("published live batch is initialized")
    }
}

pub(in crate::domain_computation::primary_graph) enum WorthQueryLiveSourcePoll {
    Batch(Arc<WorthQueryLiveCommitBatchCell>),
    Pending,
    Overflow { missed: u64 },
    Closed,
}

impl Default for WorthQueryLiveDeliverySource {
    fn default() -> Self {
        Self::with_limits(RETAINED_COMMIT_BATCH_CAPACITY, RETAINED_DELIVERY_BYTES)
    }
}

impl WorthQueryLiveDeliverySource {
    fn with_limits(batch_capacity: usize, byte_capacity: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(WorthQueryLiveDeliverySourceState::default())),
            batch_capacity: AtomicUsize::new(batch_capacity),
            byte_capacity: AtomicU64::new(byte_capacity),
            #[cfg(test)]
            after_reservation: Mutex::new(None),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn open(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> WorthQueryLiveSubscription {
        let key = WorthQueryLiveProductKey::from_observation(observation);
        let batch_capacity = self.batch_capacity.load(Ordering::Acquire);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let partition =
            state
                .partitions
                .entry(key.clone())
                .or_insert_with(|| WorthQueryLiveProductPartition {
                    batches: VecDeque::with_capacity(batch_capacity),
                    allocated_batch_capacity: batch_capacity,
                    next_sequence: 0,
                    last_product_commit: None,
                    retained_payload_bytes: 0,
                    subscriber_count: 0,
                    reservation_live: false,
                    retired: false,
                });
        partition.subscriber_count = partition
            .subscriber_count
            .checked_add(1)
            .expect("live subscriber count is bounded by process memory");
        WorthQueryLiveSubscription {
            state: Arc::clone(&self.state),
            key,
            cursor: partition.next_sequence,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn reserve(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        retained_payload_bytes: u64,
    ) -> Result<WorthQueryLivePublicationReservation, &'static str> {
        let key = WorthQueryLiveProductKey::from_observation(observation);
        let configured_batch_capacity = self.batch_capacity.load(Ordering::Acquire);
        let byte_capacity = self.byte_capacity.load(Ordering::Acquire);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.closed {
            return Ok(WorthQueryLivePublicationReservation::inactive(
                Arc::clone(&self.state),
                key,
            ));
        }
        let created = !state.partitions.contains_key(&key);
        let result = reservation::reserve_partition(
            state
                .partitions
                .entry(key.clone())
                .or_insert_with(|| WorthQueryLiveProductPartition {
                    batches: VecDeque::with_capacity(configured_batch_capacity),
                    allocated_batch_capacity: configured_batch_capacity,
                    next_sequence: 0,
                    last_product_commit: None,
                    retained_payload_bytes: 0,
                    subscriber_count: 0,
                    reservation_live: false,
                    retired: false,
                }),
            configured_batch_capacity,
            byte_capacity,
            retained_payload_bytes,
        );
        let (slot, evictions) = match result {
            Ok(reservation) => reservation,
            Err(denial) => {
                if created {
                    state.partitions.remove(&key);
                }
                return Err(denial);
            }
        };
        let reservation = WorthQueryLivePublicationReservation {
            state: Arc::clone(&self.state),
            key,
            slot: Some(slot),
            retained_payload_bytes,
            evictions,
            active: true,
        };
        drop(state);
        #[cfg(test)]
        if let Some(hook) = self
            .after_reservation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
        {
            hook();
        }
        Ok(reservation)
    }

    pub(in crate::domain_computation::primary_graph) fn publish(
        &self,
        mut reservation: WorthQueryLivePublicationReservation,
        publication: WorthQueryCommittedProductPublication,
        emissions: WorthQueryAdmittedApplicationEmissionBatch,
    ) -> usize {
        let emitted = emissions.len();
        let (emissions, retained_payload_bytes) = emissions.into_parts();
        if reservation.active {
            assert_eq!(retained_payload_bytes, reservation.retained_payload_bytes);
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.published_commit_count = state.published_commit_count.saturating_add(1);
        if !reservation.active {
            return emitted;
        }
        assert!(reservation.key.admits(&publication));
        let closed = state.closed;
        let partition = state
            .partitions
            .get_mut(&reservation.key)
            .expect("an active reservation retains its product partition");
        assert!(partition.reservation_live);
        if partition
            .last_product_commit
            .as_ref()
            .is_some_and(|last| publication.composite_commit() <= last)
        {
            panic!("application product publication causality is not strictly ordered");
        }
        let observation = publication
            .take_successor_observation()
            .expect("reserved live delivery carries World's exact successor observation");
        assert_eq!(observation.branch_identity(), publication.product_branch());
        assert_eq!(
            observation.lifecycle_incarnation(),
            publication.product_incarnation()
        );
        assert_eq!(
            observation.selected_commit(),
            publication.composite_commit()
        );
        for _ in 0..reservation.evictions {
            let evicted = partition
                .batches
                .pop_front()
                .expect("reserved eviction exists");
            partition.retained_payload_bytes = partition
                .retained_payload_bytes
                .checked_sub(evicted.batch().retained_payload_bytes)
                .expect("reserved eviction has accounted bytes");
        }
        partition.reservation_live = false;
        reservation.active = false;
        if partition.subscriber_count == 0 {
            state.partitions.remove(&reservation.key);
            return emitted;
        }
        if closed {
            return emitted;
        }
        let sequence = partition.next_sequence;
        partition.next_sequence = partition
            .next_sequence
            .checked_add(1)
            .expect("reservation proved live sequence capacity");
        let slot = reservation
            .slot
            .take()
            .expect("active reservation owns a batch slot");
        slot.0
            .set(WorthQueryLiveCommitBatch {
                sequence,
                publication,
                product: crate::basis::WorthQueryProductObservationLease::new(observation),
                emissions,
                retained_payload_bytes,
            })
            .unwrap_or_else(|_| panic!("reserved live batch initializes once"));
        partition.last_product_commit = Some(slot.batch().publication.composite_commit().clone());
        partition.retained_payload_bytes = partition
            .retained_payload_bytes
            .checked_add(retained_payload_bytes)
            .expect("reservation proved retained-byte arithmetic");
        partition.batches.push_back(slot);
        emitted
    }

    pub(in crate::domain_computation::primary_graph) fn poll(
        &self,
        subscription: &WorthQueryLiveSubscription,
        cursor: u64,
    ) -> WorthQueryLiveSourcePoll {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(partition) = state.partitions.get(&subscription.key) else {
            return if state.closed {
                WorthQueryLiveSourcePoll::Closed
            } else {
                WorthQueryLiveSourcePoll::Pending
            };
        };
        let first = partition
            .batches
            .front()
            .map_or(partition.next_sequence, |batch| batch.batch().sequence);
        if cursor < first {
            return WorthQueryLiveSourcePoll::Overflow {
                missed: first - cursor,
            };
        }
        let offset = usize::try_from(cursor - first).ok();
        if let Some(batch) = offset.and_then(|offset| partition.batches.get(offset)) {
            return WorthQueryLiveSourcePoll::Batch(Arc::clone(batch));
        }
        if state.closed || partition.retired {
            WorthQueryLiveSourcePoll::Closed
        } else {
            WorthQueryLiveSourcePoll::Pending
        }
    }

    pub(in crate::domain_computation::primary_graph) fn retire_product_occurrence(
        &self,
        branch: &worth_runtime_world::facade::ProductBranchIdentity,
        incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        let key = WorthQueryLiveProductKey {
            branch: branch.clone(),
            incarnation,
        };
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let remove = state.partitions.get_mut(&key).is_some_and(|partition| {
            partition.retired = true;
            partition.subscriber_count == 0 && !partition.reservation_live
        });
        if remove {
            state.partitions.remove(&key);
        }
    }

    pub(in crate::domain_computation::primary_graph) fn close(&self) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .closed = true;
    }

    #[cfg(feature = "test-world-operation-control")]
    pub(in crate::domain_computation::primary_graph) fn active_subscriber_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .partitions
            .values()
            .map(|partition| partition.subscriber_count)
            .sum()
    }
}
