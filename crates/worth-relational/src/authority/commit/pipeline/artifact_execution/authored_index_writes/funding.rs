//! Original-request custody for the new canonical index-write backing.
use super::*;

pub(super) struct Output {
    entries: Vec<Entry>,
    bytes: u64,
    memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl Output {
    pub(super) fn new(
        execution: ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Self, Denial> {
        let bytes = Evidence::root_bytes().ok_or(Denial::SizeOverflow)?;
        work(context, bytes)?;
        let memory = execution.reserve_retained_memory(bytes)?;
        Ok(Self {
            entries: Vec::new(),
            bytes,
            memory,
        })
    }

    pub(super) fn push(
        &mut self,
        context: &mut MapKernelContext<'_, '_>,
        variable_bytes: u64,
        make: impl FnOnce() -> Entry,
    ) -> Result<(), Denial> {
        let count = self
            .entries
            .len()
            .checked_add(1)
            .ok_or(Denial::SizeOverflow)?;
        let old_capacity = self.entries.capacity();
        let growing = count > old_capacity;
        let capacity = if growing {
            old_capacity
                .checked_mul(2)
                .ok_or(Denial::SizeOverflow)?
                .max(count)
        } else {
            old_capacity
        };
        let slots = std::alloc::Layout::array::<Entry>(capacity)
            .map_err(|_| Denial::SizeOverflow)?
            .size() as u64;
        let old_slots = std::alloc::Layout::array::<Entry>(old_capacity)
            .map_err(|_| Denial::SizeOverflow)?
            .size() as u64;
        let bytes = self
            .bytes
            .checked_add(variable_bytes)
            .and_then(|bytes| bytes.checked_add(slots))
            .ok_or(Denial::SizeOverflow)?;
        let copy_work = if growing {
            (self.entries.len() as u64)
                .checked_mul(std::mem::size_of::<Entry>() as u64)
                .and_then(|n| n.checked_add(slots))
                .ok_or_else(overflow)?
        } else {
            0
        };
        work(
            context,
            variable_bytes.checked_add(copy_work).ok_or_else(overflow)?,
        )?;
        let peak = bytes
            .checked_add(if growing { old_slots } else { 0 })
            .ok_or(Denial::SizeOverflow)?;
        resize(&mut self.memory, peak)?;
        if growing {
            self.entries
                .try_reserve_exact(capacity - self.entries.len())
                .map_err(Denial::BackingAllocation)?;
        }
        let entry = make();
        if entry.owned_bytes() > variable_bytes {
            return Err(Denial::SizeOverflow);
        }
        self.bytes = self
            .bytes
            .checked_add(entry.owned_bytes())
            .ok_or(Denial::SizeOverflow)?;
        self.entries.push(entry);
        let retained = self
            .bytes
            .checked_add(
                (self.entries.capacity() as u64)
                    .checked_mul(std::mem::size_of::<Entry>() as u64)
                    .ok_or(Denial::SizeOverflow)?,
            )
            .ok_or(Denial::SizeOverflow)?;
        resize(&mut self.memory, retained)
    }

    pub(super) fn finish(self, coverage: crate::runtime::IndexCatalogCoverage) -> Evidence {
        Evidence::prepared(self.entries, self.memory, coverage)
    }
}

fn resize(
    memory: &mut Option<worth_execution::ExecutionMemoryReservation>,
    bytes: u64,
) -> Result<(), Denial> {
    if let Some(memory) = memory {
        memory.resize(bytes).map_err(|cause| {
            Denial::Request(worth_execution::WorkCeilingDenial::Admission(
                worth_execution::LeaseDenial::MemoryExhausted(cause),
            ))
        })?;
    }
    Ok(())
}
