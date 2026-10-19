use crate::identity::data::PartitionId;
use worth_execution::MemoryLimitDenial;

/// An original request's live memory loan. Returned vectors keep their backing
/// reserved until their owner drops the vector and releases its capacity.
pub(crate) trait PartitionIdMemory {
    fn reserve(&mut self, additional_bytes: u64) -> Result<(), MemoryLimitDenial>;
    fn release(&mut self, bytes: u64);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PartitionIdsDenial {
    Memory(MemoryLimitDenial),
    SizeOverflow,
    AllocationFailed,
}

pub(crate) fn partition_id_bytes(count: usize) -> Result<u64, PartitionIdsDenial> {
    let count = u64::try_from(count).map_err(|_| PartitionIdsDenial::SizeOverflow)?;
    count
        .checked_mul(std::mem::size_of::<PartitionId>() as u64)
        .ok_or(PartitionIdsDenial::SizeOverflow)
}

pub(crate) fn collect_partition_ids(
    count: usize,
    ids: impl IntoIterator<Item = PartitionId>,
    memory: &mut dyn PartitionIdMemory,
) -> Result<Vec<PartitionId>, PartitionIdsDenial> {
    let mut output = allocate_ids(count, memory)?;
    for id in ids {
        if output.len() == count {
            release_ids(output, memory);
            return Err(PartitionIdsDenial::SizeOverflow);
        }
        output.push(id);
    }
    Ok(output)
}

fn allocate_ids(
    count: usize,
    memory: &mut dyn PartitionIdMemory,
) -> Result<Vec<PartitionId>, PartitionIdsDenial> {
    let bytes = partition_id_bytes(count)?;
    memory.reserve(bytes).map_err(PartitionIdsDenial::Memory)?;
    let mut output = Vec::new();
    if output.try_reserve_exact(count).is_err() {
        memory.release(bytes);
        return Err(PartitionIdsDenial::AllocationFailed);
    }
    // The vector owns the allocator-reported backing, even if the allocator
    // provides more elements than the requested exact reservation.
    let actual = partition_id_bytes(output.capacity())?;
    if let Err(error) = memory.reserve(actual - bytes) {
        drop(output);
        memory.release(bytes);
        return Err(PartitionIdsDenial::Memory(error));
    }
    Ok(output)
}

pub(crate) fn release_ids(ids: Vec<PartitionId>, memory: &mut dyn PartitionIdMemory) {
    let bytes = partition_id_bytes(ids.capacity()).expect("funded partition ID capacity");
    drop(ids);
    memory.release(bytes);
}

pub(super) fn merge_partition_ids(
    base: Vec<PartitionId>,
    staged: Vec<PartitionId>,
    memory: &mut dyn PartitionIdMemory,
) -> Result<Vec<PartitionId>, PartitionIdsDenial> {
    let result = (|| {
        let count = base
            .len()
            .checked_add(staged.len())
            .ok_or(PartitionIdsDenial::SizeOverflow)?;
        let mut output = allocate_ids(count, memory)?;
        let mut left = base.iter().copied().peekable();
        let mut right = staged.iter().copied().peekable();
        loop {
            let next = match (left.peek(), right.peek()) {
                (Some(a), Some(b)) => match a.cmp(b) {
                    std::cmp::Ordering::Less => left.next(),
                    std::cmp::Ordering::Greater => right.next(),
                    std::cmp::Ordering::Equal => {
                        right.next();
                        left.next()
                    }
                },
                (Some(_), None) => left.next(),
                (None, Some(_)) => right.next(),
                (None, None) => break,
            };
            output.push(next.expect("nonempty partition merge side"));
        }
        Ok(output)
    })();
    release_ids(base, memory);
    release_ids(staged, memory);
    result
}
