//! Preclaim the native collector's clones and vector coexistence on its original ticket.
use crate::execution::{denial::lease_denial, RelationalExecutionDenialCause as Cause};
use worth_execution::{ExecutionResultMemory, LeaseDenial};

pub(crate) fn claim(memory: &mut ExecutionResultMemory<'_>, bytes: u64) -> Result<(), Cause> {
    let total = memory
        .bytes()
        .checked_add(bytes)
        .ok_or(Cause::ChargedBytesOverflow)?;
    memory
        .resize(total)
        .map_err(|cause| lease_denial(LeaseDenial::MemoryExhausted(cause)))
}

pub(crate) fn release(memory: &mut ExecutionResultMemory<'_>, bytes: u64) -> Result<(), Cause> {
    let total = memory
        .bytes()
        .checked_sub(bytes)
        .ok_or(Cause::ChargedBytesOverflow)?;
    memory
        .resize(total)
        .map_err(|cause| lease_denial(LeaseDenial::MemoryExhausted(cause)))
}

pub(crate) fn vector_bytes<T>(capacity: usize) -> Result<u64, Cause> {
    u64::try_from(capacity)
        .ok()
        .and_then(|capacity| capacity.checked_mul(std::mem::size_of::<T>() as u64))
        .ok_or(Cause::ChargedBytesOverflow)
}

pub(crate) fn prepare_push<T>(
    memory: &mut ExecutionResultMemory<'_>,
    records: &mut Vec<T>,
    clone_bytes: u64,
) -> Result<(), Cause> {
    claim(memory, clone_bytes)?;
    grow(memory, records, 1)
}

pub(crate) fn grow<T>(
    memory: &mut ExecutionResultMemory<'_>,
    records: &mut Vec<T>,
    additional: usize,
) -> Result<(), Cause> {
    let required = records
        .len()
        .checked_add(additional)
        .ok_or(Cause::ChargedBytesOverflow)?;
    if required <= records.capacity() {
        return Ok(());
    }
    let capacity = records
        .capacity()
        .checked_mul(2)
        .ok_or(Cause::ChargedBytesOverflow)?
        .max(required);
    let old_bytes = vector_bytes::<T>(records.capacity())?;
    claim(memory, vector_bytes::<T>(capacity)?)?;
    let mut next = Vec::with_capacity(capacity);
    next.append(records);
    let old = std::mem::replace(records, next);
    drop(old);
    release(memory, old_bytes)
}

pub(crate) fn append<T>(
    memory: &mut ExecutionResultMemory<'_>,
    records: &mut Vec<T>,
    mut incoming: Vec<T>,
) -> Result<(), Cause> {
    grow(memory, records, incoming.len())?;
    let old_bytes = vector_bytes::<T>(incoming.capacity())?;
    records.append(&mut incoming);
    drop(incoming);
    release(memory, old_bytes)
}

pub(crate) fn set_slot(
    memory: &mut ExecutionResultMemory<'_>,
    slots: &mut crate::storage::partition::DenseSlotBitSet,
    slot: usize,
) -> Result<(), Cause> {
    let before = slots
        .owned_allocation_bytes()
        .ok_or(Cause::ChargedBytesOverflow)?;
    let peak = slots
        .set_true_allocation_bytes(slot)
        .ok_or(Cause::ChargedBytesOverflow)?;
    claim(memory, peak)?;
    slots.set(slot, true);
    let after = slots
        .owned_allocation_bytes()
        .ok_or(Cause::ChargedBytesOverflow)?;
    let retired = before
        .checked_add(peak)
        .and_then(|held| held.checked_sub(after))
        .ok_or(Cause::ChargedBytesOverflow)?;
    release(memory, retired)
}

pub(crate) fn slots<'a>(
    slots: &'a crate::storage::partition::DenseSlotBitSet,
    memory: &mut ExecutionResultMemory<'_>,
) -> Result<(crate::storage::partition::DenseSlotIterator<'a>, u64), Cause> {
    let mut held = 0;
    let iterator = slots.try_set_slots(|_navigation, bytes| {
        claim(memory, bytes)?;
        held = bytes;
        Ok::<_, Cause>(())
    })?;
    Ok((iterator, held))
}

pub(crate) fn occupied_slots<'a, K: crate::storage::substrate::RecordKind>(
    arena: &'a crate::storage::substrate::RecordArena<K>,
    memory: &mut ExecutionResultMemory<'_>,
) -> Result<(impl Iterator<Item = usize> + use<'a, K>, u64), Cause> {
    let mut held = 0;
    let iterator = arena.try_occupied_slots(|_navigation, bytes| {
        claim(memory, bytes)?;
        held = bytes;
        Ok::<_, Cause>(())
    })?;
    Ok((iterator, held))
}

pub(crate) fn partition_ids(
    state: &(impl crate::storage::overlay::PartitionAccess + ?Sized),
    memory: &mut ExecutionResultMemory<'_>,
) -> Result<(Vec<crate::identity::data::PartitionId>, u64), Cause> {
    use crate::storage::overlay::{PartitionIdMemory, PartitionIdsDenial};
    struct OriginalLoan<'a, 'scope>(&'a mut ExecutionResultMemory<'scope>);
    impl PartitionIdMemory for OriginalLoan<'_, '_> {
        fn reserve(&mut self, bytes: u64) -> Result<(), worth_execution::MemoryLimitDenial> {
            let charged = self.0.bytes();
            let total = charged.checked_add(bytes).ok_or(
                worth_execution::MemoryLimitDenial::CounterOverflow {
                    requested: bytes,
                    charged,
                    level: worth_execution::MemoryLimitLevel::Declared,
                },
            )?;
            self.0.resize(total)
        }
        fn release(&mut self, bytes: u64) {
            let total = self
                .0
                .bytes()
                .checked_sub(bytes)
                .expect("the original ID vector owns its reserved backing");
            self.0
                .resize(total)
                .expect("releasing already held memory cannot refuse");
        }
    }
    let ids = state
        .partition_ids_with_memory(&mut OriginalLoan(memory))
        .map_err(|denial| match denial {
            PartitionIdsDenial::Memory(cause) => lease_denial(LeaseDenial::MemoryExhausted(cause)),
            PartitionIdsDenial::SizeOverflow => Cause::ChargedBytesOverflow,
            PartitionIdsDenial::AllocationFailed => Cause::AllocationFailed,
        })?;
    let held = vector_bytes::<crate::identity::data::PartitionId>(ids.capacity())?;
    Ok((ids, held))
}

/// Settle the original ticket using the same owner-accounted payload measure
/// before and after Clone. This measure excludes allocator/BTree structural
/// backing; source spare payload capacity is released only after construction.
pub(crate) fn settle_clone(
    memory: &mut ExecutionResultMemory<'_>,
    quoted: u64,
    actual: u64,
) -> Result<(), Cause> {
    let unused = quoted
        .checked_sub(actual)
        .expect("the clone payload does not exceed its owner-accounted source quote");
    release(memory, unused)
}
