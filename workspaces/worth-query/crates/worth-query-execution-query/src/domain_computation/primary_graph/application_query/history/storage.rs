//! Fixed history wrapper bounds accompany the existing Native/World backing owners.
use super::WorthQueryApplicationHistoryDenial as Denial;

pub const fn arc_bytes<T>() -> usize {
    size_of::<T>() + 2 * size_of::<usize>()
}

pub fn add(parts: &[usize]) -> Result<usize, Denial> {
    parts.iter().try_fold(0usize, |sum, part| {
        sum.checked_add(*part).ok_or(Denial::SizeOverflow)
    })
}

pub fn require(bytes: usize, maximum: usize) -> Result<(), Denial> {
    if bytes > maximum {
        Err(Denial::StorageLimitExceeded)
    } else {
        Ok(())
    }
}

pub fn retain(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    bytes: usize,
) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Denial> {
    execution
        .reserve_retained_memory(u64::try_from(bytes).map_err(|_| Denial::SizeOverflow)?)
        .map_err(Denial::Request)
}
