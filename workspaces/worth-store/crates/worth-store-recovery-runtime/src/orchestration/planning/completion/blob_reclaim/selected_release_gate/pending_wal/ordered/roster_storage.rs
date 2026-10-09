//! Fallible, resident-charged backing for ordered batch and replay rosters.

use super::{Denial, ResidentAllowance, Storage};

pub(super) fn reserve_roster<T>(
    values: &mut Vec<T>,
    count: usize,
    storage: Storage,
    resident: &mut ResidentAllowance,
) -> Result<(), Denial> {
    let requested = u64::try_from(count)
        .ok()
        .and_then(|count| count.checked_mul(std::mem::size_of::<T>() as u64))
        .unwrap_or(u64::MAX);
    resident
        .transient(requested)
        .map_err(|_| Denial::ResidentBoundExceeded)?;
    values
        .try_reserve_exact(count)
        .map_err(|cause| Denial::RosterAllocation {
            storage,
            requested,
            cause,
        })?;
    let actual = u64::try_from(values.capacity())
        .ok()
        .and_then(|capacity| capacity.checked_mul(std::mem::size_of::<T>() as u64))
        .unwrap_or(u64::MAX);
    resident
        .bytes(actual)
        .map_err(|_| Denial::ResidentBoundExceeded)
}
