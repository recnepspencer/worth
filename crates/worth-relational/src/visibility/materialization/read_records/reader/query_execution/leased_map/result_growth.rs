//! Result vectors preclaim replacement backing while the original buffer lives.
use super::{ChargedBytes, MapKernelFailure, QueryPacketMemory, QueryReadPacketDenial};

type Failure = MapKernelFailure<QueryReadPacketDenial>;

pub(super) fn reserve_record<T: ChargedBytes>(
    memory: &QueryPacketMemory,
    result_bytes: &mut u64,
    records: &mut Vec<T>,
    record: &T,
) -> Result<(), Failure> {
    let payload = record.additional_charged_bytes();
    let with_payload = memory.add(*result_bytes, payload)?;
    if records.len() < records.capacity() {
        memory.result(with_payload)?;
        *result_bytes = with_payload;
        return Ok(());
    }
    let required = records
        .len()
        .checked_add(1)
        .ok_or_else(|| overflow(memory))?;
    let old_backing = vector_bytes::<T>(memory, records.capacity())?;
    let replacement_backing = vector_bytes::<T>(memory, required)?;
    // result_bytes still includes the complete old backing and every existing
    // row payload. The incoming payload and complete replacement coexist here.
    let peak = memory.add(with_payload, replacement_backing)?;
    memory.result(peak)?;
    let mut replacement = Vec::new();
    if replacement.try_reserve_exact(required).is_err() {
        memory.settle_result(*result_bytes)?;
        return Err(MapKernelFailure::ResultCapacityExceeded);
    }
    let actual_backing = match vector_bytes::<T>(memory, replacement.capacity()) {
        Ok(bytes) => bytes,
        Err(error) => {
            drop(replacement);
            memory.settle_result(*result_bytes)?;
            return Err(error);
        }
    };
    let actual_peak = match memory.add(with_payload, actual_backing) {
        Ok(bytes) => bytes,
        Err(error) => {
            drop(replacement);
            memory.settle_result(*result_bytes)?;
            return Err(error);
        }
    };
    if let Err(error) = memory.result(actual_peak) {
        drop(replacement);
        memory.settle_result(*result_bytes)?;
        return Err(error);
    }
    replacement.append(records);
    let old = std::mem::replace(records, replacement);
    drop(old);
    // All saved values now live in the new vector. Only the original old
    // backing retires; existing payloads and the incoming payload stay funded.
    let final_bytes = actual_peak
        .checked_sub(old_backing)
        .ok_or_else(|| overflow(memory))?;
    memory.settle_result(final_bytes)?;
    *result_bytes = final_bytes;
    Ok(())
}

fn vector_bytes<T>(memory: &QueryPacketMemory, count: usize) -> Result<u64, Failure> {
    u64::try_from(count)
        .ok()
        .and_then(|count| count.checked_mul(std::mem::size_of::<T>() as u64))
        .ok_or_else(|| overflow(memory))
}
fn overflow(memory: &QueryPacketMemory) -> Failure {
    if memory.ceiling.is_some() {
        MapKernelFailure::ResultCapacityExceeded
    } else {
        MapKernelFailure::Domain(QueryReadPacketDenial::ReadRequest(
            crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
        ))
    }
}

#[cfg(test)]
mod tests;
