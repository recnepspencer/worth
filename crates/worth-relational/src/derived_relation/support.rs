//! Capacity-funded retention of every positive and negative inverse Source read.
use super::evaluation::SourceRead;
use super::DerivedRelationReadDenial as Denial;
use super::{DerivedRelationData as Data, DerivedRelationDenial as Definition};

pub(crate) fn push(
    read: &mut impl SourceRead,
    support: &mut Vec<Data>,
    data: Data,
) -> Result<(), Denial> {
    reserve(read, support, 1)?;
    support.push(data);
    Ok(())
}

/// Both buffers already belong to this Source stage's original payer. A custody
/// move preserves every occurrence, including repeated and empty inverse reads.
pub(crate) fn append(
    read: &mut impl SourceRead,
    support: &mut Vec<Data>,
    mut incoming: Vec<Data>,
) -> Result<(), Denial> {
    if support.is_empty() {
        *support = incoming;
    } else {
        reserve(read, support, incoming.len())?;
        support.append(&mut incoming);
    }
    Ok(())
}

fn reserve(
    read: &mut impl SourceRead,
    support: &mut Vec<Data>,
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
            .checked_mul(size_of::<Data>())
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
