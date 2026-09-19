use std::sync::{Arc, OnceLock};

use super::{WorthQueryLiveCommitBatchCell, WorthQueryLiveProductPartition};

pub(super) fn reserve_partition(
    partition: &mut WorthQueryLiveProductPartition,
    configured_batch_capacity: usize,
    byte_capacity: u64,
    retained_payload_bytes: u64,
) -> Result<(Arc<WorthQueryLiveCommitBatchCell>, usize), &'static str> {
    if partition.reservation_live {
        return Err("a product live-delivery reservation is already active");
    }
    let batch_capacity = configured_batch_capacity.min(partition.allocated_batch_capacity);
    if batch_capacity == 0 || retained_payload_bytes > byte_capacity {
        return Err("live delivery has no bounded capacity for this commit batch");
    }
    partition
        .next_sequence
        .checked_add(1)
        .ok_or("live delivery sequence space is exhausted")?;
    let mut evictions = 0;
    let mut retained_after = partition.retained_payload_bytes;
    while partition.batches.len() - evictions + 1 > batch_capacity
        || retained_after
            .checked_add(retained_payload_bytes)
            .is_none_or(|bytes| bytes > byte_capacity)
    {
        let Some(evicted) = partition.batches.get(evictions) else {
            return Err("live delivery cannot reserve the commit batch");
        };
        retained_after = retained_after
            .checked_sub(evicted.batch().retained_payload_bytes)
            .ok_or("live delivery retained-byte accounting is inconsistent")?;
        evictions += 1;
    }
    retained_after
        .checked_add(retained_payload_bytes)
        .ok_or("live delivery retained-byte count overflowed")?;
    partition.reservation_live = true;
    Ok((
        Arc::new(WorthQueryLiveCommitBatchCell(OnceLock::new())),
        evictions,
    ))
}
