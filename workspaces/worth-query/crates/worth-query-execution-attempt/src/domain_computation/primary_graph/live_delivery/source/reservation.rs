//! Original request funds publication and out-of-gate eviction custody.
use super::*;

pub(super) struct AdmittedStorage {
    pub(super) slot: Option<Arc<WorthQueryLiveCommitBatchCell>>,
    pub(super) retired_batches: Vec<Arc<WorthQueryLiveCommitBatchCell>>,
    pub(super) allocation: Option<Arc<SourceFactsAllocation>>,
}
pub(super) fn admit_storage(
    execution: &ExecutionRequest<'_, '_>,
    capacity: usize,
) -> Result<AdmittedStorage, WorthQueryLiveAdmissionDenial> {
    let cell_allocation = SourceFactsAllocation::open(execution)?;
    cell_allocation.claim_arc::<WorthQueryLiveCommitBatchCell>(execution)?;
    let scratch_allocation = SourceFactsAllocation::open(execution)?;
    scratch_allocation.claim_slice::<Arc<WorthQueryLiveCommitBatchCell>>(execution, capacity)?;
    let mut retired_batches = Vec::new();
    retired_batches
        .try_reserve_exact(capacity)
        .map_err(|_| Cause::AllocationFailed)?;
    Ok(AdmittedStorage {
        slot: Some(Arc::new(WorthQueryLiveCommitBatchCell {
            batch: OnceLock::new(),
            allocation_custody: Some(cell_allocation),
        })),
        retired_batches,
        allocation: Some(scratch_allocation),
    })
}

pub(super) fn reserve_partition(
    partition: &mut WorthQueryLiveProductPartition,
    configured_batch_capacity: usize,
) -> Result<usize, &'static str> {
    if partition.retired {
        return Err("the product live-delivery occurrence is retired");
    }
    if partition.opening_count != 0 {
        return Err("a managed predicate opening owns the selected image");
    }
    if partition.reservation_live {
        return Err("a product live-delivery reservation is already active");
    }
    let batch_capacity = configured_batch_capacity.min(partition.allocated_batch_capacity);
    if batch_capacity == 0 {
        return Err("live delivery has no bounded capacity for this commit batch");
    }
    partition
        .next_sequence
        .checked_add(1)
        .ok_or("live delivery sequence space is exhausted")?;
    let pending = partition
        .batches
        .len()
        .checked_add(1)
        .ok_or("live delivery batch count overflowed")?;
    let evictions = pending.saturating_sub(batch_capacity);
    partition.reservation_live = true;
    Ok(evictions)
}
