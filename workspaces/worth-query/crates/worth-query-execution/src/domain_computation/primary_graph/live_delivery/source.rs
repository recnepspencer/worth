use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

mod custody;
mod partition;
mod predicate;
mod publication;
mod reservation;
pub(crate) mod subscription;
pub(in crate::domain_computation::primary_graph) use subscription::WorthQueryLiveSubscriptionCapacityExhausted;
mod testing;

use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
pub(in crate::domain_computation::primary_graph) use custody::{
    WorthQueryLivePublicationReservation, WorthQueryLiveSubscription,
};
pub(in crate::domain_computation::primary_graph) use predicate::{
    WorthQueryLivePredicateOpening, WorthQueryLivePredicatePreparation,
    WorthQueryLivePredicateSubscription,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryLiveAdmissionDenial {
    Execution(Cause),
    Unavailable(&'static str),
}
impl From<&'static str> for WorthQueryLiveAdmissionDenial {
    fn from(detail: &'static str) -> Self {
        Self::Unavailable(detail)
    }
}
impl From<Cause> for WorthQueryLiveAdmissionDenial {
    fn from(cause: Cause) -> Self {
        Self::Execution(cause)
    }
}

use super::super::application_attempt::WorthQueryApplicationEmission;
use crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication;

const RETAINED_COMMIT_BATCH_CAPACITY: usize = 64;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryLiveProductKey {
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

pub struct WorthQueryLiveDeliverySource {
    state: Arc<Mutex<WorthQueryLiveDeliverySourceState>>,
    batch_capacity: AtomicUsize,
    after_reservation: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

#[derive(Default)]
struct WorthQueryLiveDeliverySourceState {
    partitions: BTreeMap<WorthQueryLiveProductKey, WorthQueryLiveProductPartition>,
    published_commit_count: usize,
    subscriber_count: usize,
    closed: bool,
}

struct WorthQueryLiveProductPartition {
    batches: VecDeque<Arc<WorthQueryLiveCommitBatchCell>>,
    allocated_batch_capacity: usize,
    next_sequence: u64,
    last_product_commit: Option<worth_runtime_world::facade::CompositeCommitIdentity>,
    subscriber_count: usize,
    reservation_live: bool,
    retired: bool,
    preparation_count: usize,
    opening_count: usize,
    resync_epoch: u64,
    _allocation: Option<Arc<SourceFactsAllocation>>,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryLiveCommitBatch {
    pub(in crate::domain_computation::primary_graph) sequence: u64,
    pub(super) publication: WorthQueryCommittedProductPublication,
    pub(super) product: crate::basis::WorthQueryProductObservationLease,
    pub(super) emissions: Vec<WorthQueryApplicationEmission>,
    pub(super) predicate: Option<Arc<crate::domain_computation::primary_graph::application_query::derived_view::PreparedPredicateNotices>>,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryLiveCommitBatchCell {
    batch: OnceLock<WorthQueryLiveCommitBatch>,
    // Last original payer closes after all retained event and predicate payloads.
    allocation_custody: Option<Arc<SourceFactsAllocation>>,
}
impl Drop for WorthQueryLiveCommitBatchCell {
    fn drop(&mut self) {
        drop(self.batch.take());
        drop(self.allocation_custody.take());
    }
}

impl WorthQueryLiveCommitBatchCell {
    pub(in crate::domain_computation::primary_graph) fn batch(&self) -> &WorthQueryLiveCommitBatch {
        self.batch
            .get()
            .expect("published live batch is initialized")
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
        Self::with_limits(RETAINED_COMMIT_BATCH_CAPACITY)
    }
}

impl WorthQueryLiveDeliverySource {
    fn with_limits(batch_capacity: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(WorthQueryLiveDeliverySourceState::default())),
            batch_capacity: AtomicUsize::new(batch_capacity),
            after_reservation: Mutex::new(None),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn reserve(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<WorthQueryLivePublicationReservation, WorthQueryLiveAdmissionDenial> {
        let key = WorthQueryLiveProductKey::from_observation(observation);
        let configured_batch_capacity = self.batch_capacity.load(Ordering::Acquire);
        let mut candidate = self.funded_partition(execution, observation)?;
        let admitted_capacity = candidate.capacity;
        let mut storage = reservation::admit_storage(execution, admitted_capacity)?;
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
        Self::lookup_work(execution, &state, &key)?;
        let created = !state.partitions.contains_key(&key);
        if state
            .partitions
            .get(&key)
            .is_some_and(|partition| partition.allocated_batch_capacity > admitted_capacity)
        {
            return Err("live partition capacity changed during reservation admission".into());
        }
        self.install_funded_partition(&mut state, &key, &mut candidate)?;
        let result = reservation::reserve_partition(
            state
                .partitions
                .get_mut(&key)
                .expect("funded product partition"),
            configured_batch_capacity,
        );
        let evictions = match result {
            Ok(evictions) => evictions,
            Err(denial) => {
                let retired = created.then(|| state.remove_partition(&key));
                drop(state);
                drop(retired);
                return Err(denial.into());
            }
        };
        let reservation = WorthQueryLivePublicationReservation {
            state: Arc::clone(&self.state),
            key,
            slot: storage.slot.take(),
            evictions,
            retired_batches: storage.retired_batches,
            _allocation: storage.allocation,
            active: true,
        };
        drop(state);
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
            partition.is_unowned()
        });
        let retired = remove.then(|| state.remove_partition(&key));
        drop(state);
        drop(retired);
    }

    pub(in crate::domain_computation::primary_graph) fn close(&self) -> usize {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.closed = true;
        state.subscriber_count
    }

    pub(in crate::domain_computation::primary_graph) fn active_subscriber_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .subscriber_count
    }
}
