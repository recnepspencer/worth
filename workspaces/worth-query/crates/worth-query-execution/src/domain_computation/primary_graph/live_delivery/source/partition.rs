//! One partition directory, with backing retired before its final funding.
use super::*;

pub(super) struct RetiredPartition {
    _empty_backing: Option<BTreeMap<WorthQueryLiveProductKey, WorthQueryLiveProductPartition>>,
    _partition: WorthQueryLiveProductPartition,
}
pub(super) struct PreparedPartition {
    candidate: Option<WorthQueryLiveProductPartition>,
    upgrade: Option<Arc<SourceFactsAllocation>>,
    pub(super) capacity: usize,
}
impl WorthQueryLiveProductPartition {
    pub(super) fn new(
        capacity: usize,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Self {
        Self {
            batches: VecDeque::with_capacity(capacity),
            allocated_batch_capacity: capacity,
            next_sequence: 0,
            last_product_commit: Some(observation.selected_commit().clone()),
            subscriber_count: 0,
            reservation_live: false,
            retired: false,
            preparation_count: 0,
            opening_count: 0,
            resync_epoch: 0,
            _allocation: None,
        }
    }
    pub(super) fn is_unowned(&self) -> bool {
        self.subscriber_count == 0
            && !self.reservation_live
            && self.preparation_count == 0
            && self.opening_count == 0
    }
}
impl WorthQueryLiveDeliverySourceState {
    pub(super) fn remove_partition(
        &mut self,
        key: &WorthQueryLiveProductKey,
    ) -> Option<RetiredPartition> {
        let partition = self.partitions.remove(key)?;
        Some(RetiredPartition {
            _empty_backing: self
                .partitions
                .is_empty()
                .then(|| std::mem::take(&mut self.partitions)),
            _partition: partition,
        })
    }
}
impl Drop for WorthQueryLiveDeliverySourceState {
    fn drop(&mut self) {
        while let Some((_, partition)) = self.partitions.pop_first() {
            if self.partitions.is_empty() {
                drop(std::mem::take(&mut self.partitions));
            }
            drop(partition);
        }
    }
}
impl WorthQueryLiveDeliverySource {
    pub(super) fn funded_partition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<PreparedPartition, WorthQueryLiveAdmissionDenial> {
        let key = WorthQueryLiveProductKey::from_observation(observation);
        let configured = self.batch_capacity.load(Ordering::Acquire);
        let (capacity, backing, upgrade) = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            Self::lookup_work(execution, &state, &key)?;
            match state.partitions.get(&key) {
                Some(partition) if partition._allocation.is_some() => {
                    return Ok(PreparedPartition {
                        candidate: None,
                        upgrade: None,
                        capacity: partition.allocated_batch_capacity,
                    })
                }
                Some(partition) => (
                    partition.allocated_batch_capacity,
                    partition.batches.capacity(),
                    true,
                ),
                None => (configured, configured, false),
            }
        };
        let allocation = SourceFactsAllocation::open(execution)?;
        // Two maximal fixed-layout BTree nodes cover insertion/split backing.
        // Branch names are shared Arc<str>; key clones allocate no new text.
        let bytes = size_of::<(WorthQueryLiveProductKey, WorthQueryLiveProductPartition)>()
            .checked_mul(11)
            .and_then(|n| n.checked_add(14 * size_of::<usize>()))
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        allocation.claim(execution, bytes)?;
        allocation.claim_slice::<Arc<WorthQueryLiveCommitBatchCell>>(execution, backing)?;
        if upgrade {
            return Ok(PreparedPartition {
                candidate: None,
                upgrade: Some(allocation),
                capacity,
            });
        }
        let mut batches = VecDeque::new();
        batches
            .try_reserve_exact(capacity)
            .map_err(|_| Cause::AllocationFailed)?;
        // VecDeque can round its requested capacity; fund the actual excess.
        allocation.claim_slice::<Arc<WorthQueryLiveCommitBatchCell>>(
            execution,
            batches.capacity().saturating_sub(capacity),
        )?;
        Ok(PreparedPartition {
            candidate: Some(WorthQueryLiveProductPartition {
                batches,
                allocated_batch_capacity: capacity,
                _allocation: Some(allocation),
                ..WorthQueryLiveProductPartition::new(0, observation)
            }),
            upgrade: None,
            capacity,
        })
    }
    pub(super) fn install_funded_partition(
        &self,
        state: &mut WorthQueryLiveDeliverySourceState,
        key: &WorthQueryLiveProductKey,
        prepared: &mut PreparedPartition,
    ) -> Result<Arc<SourceFactsAllocation>, WorthQueryLiveAdmissionDenial> {
        if let Some(partition) = state.partitions.get_mut(key) {
            if partition._allocation.is_none() {
                let backing = prepared
                    .upgrade
                    .as_ref()
                    .ok_or("live partition funding changed during admission")?;
                if partition.allocated_batch_capacity > prepared.capacity {
                    return Err("live partition capacity changed during admission".into());
                }
                partition._allocation = Some(backing.clone());
            }
        } else {
            let candidate = prepared
                .candidate
                .take()
                .ok_or("live partition retired during admission")?;
            state.partitions.insert(key.clone(), candidate);
        }
        Ok(state
            .partitions
            .get(key)
            .expect("actual partition")
            ._allocation
            .clone()
            .expect("funded actual partition"))
    }
    pub(super) fn lookup_work(
        execution: &ExecutionRequest<'_, '_>,
        state: &WorthQueryLiveDeliverySourceState,
        key: &WorthQueryLiveProductKey,
    ) -> Result<(), WorthQueryLiveAdmissionDenial> {
        let count = state
            .partitions
            .len()
            .checked_add(1)
            .ok_or(Cause::WorkCounterOverflow)?;
        let levels = u64::from(usize::BITS - count.leading_zeros());
        let width =
            u64::try_from(key.branch.name().as_str().len() + size_of::<WorthQueryLiveProductKey>())
                .map_err(|_| Cause::WorkCounterOverflow)?;
        let work = levels
            .checked_mul(11 * 4)
            .and_then(|n| n.checked_mul(width))
            .ok_or(Cause::WorkCounterOverflow)?;
        SourceFactsAllocation::work(execution, work)?;
        Ok(())
    }
}
