//! Merge declarations follow the actual retained payload and one live heap key per stream.
use super::*;
pub(super) fn declaration<K: Ord, T, O>(
    streams: &Vec<OrderedReductionStream<K, T>>,
    item_bytes: &impl Fn(&K, &T) -> u64,
    key_clone_bytes: &impl Fn(&K) -> u64,
) -> Result<(u64, u64), PacketExecutionStop> {
    let mut result_count = 0_u64;
    let result_payload = std::cell::Cell::new(Some(0_u64));
    let mut cloned_key_peak = 0_u64;
    let mut owned_bytes =
        (streams.capacity() as u64).checked_mul(size_of::<OrderedReductionStream<K, T>>() as u64);
    for stream in streams {
        result_count = result_count.checked_add(stream.items.len() as u64).ok_or(
            PacketExecutionStop::Admission(crate::execution::PacketAdmissionDenial::MemoryOverflow),
        )?;
        let stream_key_peak = std::cell::Cell::new(0_u64);
        let stream_bytes = stream.owned_allocation_capacity_bytes(|key, item| {
            stream_key_peak.set(stream_key_peak.get().max(key_clone_bytes(key)));
            let nested = item_bytes(key, item);
            result_payload.set(
                result_payload
                    .get()
                    .and_then(|total| total.checked_add(nested)),
            );
            nested
        });
        owned_bytes = owned_bytes.and_then(|bytes| bytes.checked_add(stream_bytes));
        cloned_key_peak = cloned_key_peak.checked_add(stream_key_peak.get()).ok_or(
            PacketExecutionStop::Admission(crate::execution::PacketAdmissionDenial::MemoryOverflow),
        )?;
    }
    let owned_bytes = owned_bytes.ok_or(PacketExecutionStop::Admission(
        crate::execution::PacketAdmissionDenial::MemoryOverflow,
    ))?;
    let result_bound = result_count
        .checked_mul(size_of::<O>() as u64)
        .and_then(|bytes| bytes.checked_add(result_payload.get()?));
    let scratch_bound = (streams.len() as u64)
        .checked_mul(
            (size_of::<std::vec::IntoIter<(K, T)>>()
                + size_of::<Option<(K, T)>>()
                + size_of::<MergeHead<K>>()) as u64,
        )
        .and_then(|bytes| bytes.checked_add(cloned_key_peak));
    let declared_bytes = result_bound
        .zip(scratch_bound)
        .map(|(result, scratch)| result.max(scratch))
        .ok_or(PacketExecutionStop::Admission(
            crate::execution::PacketAdmissionDenial::MemoryOverflow,
        ))?;
    Ok((owned_bytes, declared_bytes))
}
