use std::sync::{Arc, OnceLock};

use super::{WorthQueryLiveCommitBatchCell, WorthQueryLiveProductPartition};

pub(super) fn reserve_partition(
    partition: &mut WorthQueryLiveProductPartition,
    configured_batch_capacity: usize,
) -> Result<(Arc<WorthQueryLiveCommitBatchCell>, usize), &'static str> {
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
    Ok((
        Arc::new(WorthQueryLiveCommitBatchCell(OnceLock::new())),
        evictions,
    ))
}
