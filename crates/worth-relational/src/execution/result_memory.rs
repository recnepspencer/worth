//! Resize the original stage reservation without creating another account.
use super::{denial::lease_denial, RelationalExecutionDenialCause as Cause};
use worth_execution::{ExecutionResultMemory, LeaseDenial};

/// Only the effective original stage can select this optional borrowed loan.
/// Absence means the caller supplied no accounting, not a new free account.
pub(crate) struct OriginalResultMemory<'loan, 'scope> {
    memory: Option<&'loan mut ExecutionResultMemory<'scope>>,
}

impl<'loan, 'scope> OriginalResultMemory<'loan, 'scope> {
    pub(in crate::execution) fn new(
        memory: Option<&'loan mut ExecutionResultMemory<'scope>>,
    ) -> Self {
        Self { memory }
    }
    pub(crate) fn loan(&mut self) -> Option<&mut ExecutionResultMemory<'scope>> {
        self.memory.as_deref_mut()
    }
    pub(crate) fn bytes(&self) -> u64 {
        self.memory.as_ref().map_or(0, |memory| memory.bytes())
    }
    pub(crate) fn claim(&mut self, bytes: u64) -> Result<(), Cause> {
        match self.memory.as_deref_mut() {
            Some(memory) => claim(memory, bytes),
            None => Ok(()),
        }
    }
    pub(crate) fn release(&mut self, bytes: u64) -> Result<(), Cause> {
        match self.memory.as_deref_mut() {
            Some(memory) => release(memory, bytes),
            None => Ok(()),
        }
    }
}

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
