//! Original-funded capacity growth and reallocation overlap for Source buffers.
use super::evaluation::SourceRead;
use super::{DerivedRelationDenial as Definition, DerivedRelationReadDenial as Denial};

pub(in crate::derived_relation) fn reserve<T>(
    read: &mut impl SourceRead,
    support: &mut Vec<T>,
    additional: usize,
) -> Result<(), Denial> {
    let required = support
        .len()
        .checked_add(additional)
        .ok_or(Definition::WorkBudgetExceeded)?;
    let old = support.capacity();
    if required <= old {
        return Ok(());
    }
    let capacity = old
        .checked_mul(2)
        .ok_or(Definition::WorkBudgetExceeded)?
        .max(required)
        .max(4);
    let bytes = |count: usize| {
        count
            .checked_mul(size_of::<T>())
            .ok_or(Definition::WorkBudgetExceeded)
    };
    // The retained delta makes the current claim equal to the new backing.
    // The OLD backing is then admitted as extra overlap before reallocation.
    read.retain(bytes(capacity - old)?)?;
    read.check_scratch_peak(bytes(old)?)?;
    support
        .try_reserve_exact(capacity - support.len())
        .map_err(|_| allocation_failed())?;
    // The pinned native allocator reports the requested exact Vec capacity.
    // Refuse an unexpected layout rather than append into unadmitted backing.
    if support.capacity() != capacity {
        return Err(allocation_failed());
    }
    Ok(())
}

fn allocation_failed() -> Denial {
    Denial::Request(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
}
