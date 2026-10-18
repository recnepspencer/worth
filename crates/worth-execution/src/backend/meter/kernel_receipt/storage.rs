//! Dynamic receipt backing is admitted before allocation and retained by clones.
use super::{ExecutionKernelReceipt, Receipt};
use crate::{ExecutionMemoryReservation, MemoryLimitDenial};
use allocator_api2::{alloc::Global, vec::Vec};
use std::sync::{Arc, Mutex};

/// The native reservation for captures on one run. This has no cost-minting API.
#[derive(Clone, Debug)]
pub struct ExecutionKernelReceiptStorage(Arc<Budget>);

#[derive(Debug)]
struct Budget {
    memory: Mutex<ExecutionMemoryReservation>,
}

impl ExecutionKernelReceiptStorage {
    /// Take over a genuine reservation for this storage. Its lineage is held until
    /// the final receipt/capture drops; failed admission retains no storage.
    pub fn new(mut memory: ExecutionMemoryReservation) -> Result<Self, MemoryLimitDenial> {
        memory.resize((size_of::<Budget>() + 2 * size_of::<usize>()) as u64)?;
        Ok(Self(Arc::new(Budget {
            memory: Mutex::new(memory),
        })))
    }

    pub(super) fn capture(&self) -> Capture {
        Capture {
            steps: Vec::new_in(Global),
            work: 0,
            valid: true,
            memory: ReceiptMemory {
                budget: Arc::clone(&self.0),
                bytes: 0,
            },
        }
    }
}

#[derive(Debug)]
pub(super) struct ReceiptMemory {
    budget: Arc<Budget>,
    bytes: u64,
}

impl ReceiptMemory {
    pub(super) fn bytes(&self) -> u64 {
        self.bytes + (size_of::<Budget>() + 2 * size_of::<usize>()) as u64
    }

    fn release(&mut self, bytes: u64) {
        let mut memory = self
            .budget
            .memory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let remaining = memory
            .bytes()
            .checked_sub(bytes)
            .expect("native receipt backing release");
        memory
            .resize(remaining)
            .expect("release cannot exceed a memory limit");
        self.bytes = self
            .bytes
            .checked_sub(bytes)
            .expect("this receipt's backing");
    }

    fn grow(&mut self, bytes: u64) -> bool {
        let mut memory = self
            .budget
            .memory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(total) = memory.bytes().checked_add(bytes) else {
            return false;
        };
        if memory.resize(total).is_err() {
            return false;
        }
        self.bytes += bytes;
        true
    }
}

impl Drop for ReceiptMemory {
    fn drop(&mut self) {
        let mut memory = self
            .budget
            .memory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let remaining = memory
            .bytes()
            .checked_sub(self.bytes)
            .expect("native receipt funding");
        memory
            .resize(remaining)
            .expect("releasing receipt bytes cannot exceed a limit");
    }
}

pub(in super::super) struct Capture {
    steps: Vec<(u64, u64)>,
    work: u64,
    valid: bool,
    memory: ReceiptMemory,
}

impl Capture {
    pub(in super::super) fn invalidate(&mut self) {
        self.valid = false;
    }

    pub(in super::super) fn accepted(&mut self, units: u64) {
        if !self.valid {
            return;
        }
        let Some(work) = self.work.checked_add(units) else {
            self.invalidate();
            return;
        };
        if let Some((previous, count)) = self.steps.last_mut()
            && *previous == units
        {
            if let Some(next) = count.checked_add(1) {
                *count = next;
                self.work = work;
            } else {
                self.invalidate();
            }
            return;
        }
        if self.steps.len() == self.steps.capacity() {
            let capacity = self.steps.capacity().max(1).checked_mul(2);
            let bytes = capacity
                .and_then(|n| n.checked_mul(size_of::<(u64, u64)>()))
                .and_then(|n| u64::try_from(n).ok());
            let Some((capacity, bytes)) = capacity.zip(bytes) else {
                self.invalidate();
                return;
            };
            // Fresh pinned backing has the quoted capacity. Both allocations
            // remain admitted until every old element moved and backing drops.
            if !self.memory.grow(bytes) {
                self.invalidate();
                return;
            }
            let mut replacement: Vec<(u64, u64)> = Vec::new_in(Global);
            if replacement.try_reserve_exact(capacity).is_err() {
                self.memory.release(bytes);
                self.invalidate();
                return;
            }
            if replacement.capacity() != capacity {
                drop(replacement);
                self.memory.release(bytes);
                self.invalidate();
                return;
            }
            replacement.append(&mut self.steps);
            let old = std::mem::replace(&mut self.steps, replacement);
            let old_bytes = (old.capacity() * size_of::<(u64, u64)>()) as u64;
            drop(old);
            self.memory.release(old_bytes);
        }
        self.steps.push((units, 1));
        self.work = work;
    }

    pub(super) fn finish(
        mut self,
        before: (u64, u64),
        after: (u64, u64),
    ) -> Option<ExecutionKernelReceipt> {
        if !self.valid
            || after.0.checked_sub(before.0) != Some(self.work)
            || after.1.checked_sub(before.1) != Some(self.work)
            || !self
                .memory
                .grow((size_of::<Receipt>() + 2 * size_of::<usize>()) as u64)
        {
            return None;
        }
        Some(ExecutionKernelReceipt(Arc::new(Receipt {
            steps: self.steps,
            work: self.work,
            _memory: self.memory,
        })))
    }
}
