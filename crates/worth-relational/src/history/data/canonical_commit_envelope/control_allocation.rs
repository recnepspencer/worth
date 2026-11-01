//! Fixed cold envelope allocation funding shared with its real weak observers.
use super::CanonicalCommitEnvelope;
use std::{alloc::Layout, fmt};
use worth_execution::ExecutionMemoryReservation;

pub(crate) struct CanonicalEnvelopeControlAllocation {
    // No body, source or runtime edge: this ticket cannot keep payload alive.
    memory: ExecutionMemoryReservation,
}
impl CanonicalEnvelopeControlAllocation {
    /// Both concrete Arc layouts use the pinned SDK's checked header convention.
    pub(crate) fn allocation_bytes() -> Option<u64> {
        arc_bytes::<CanonicalCommitEnvelope>()?.checked_add(arc_bytes::<Self>()?)
    }
    pub(crate) fn from_reserved(memory: ExecutionMemoryReservation) -> Self {
        Self { memory }
    }
}
impl fmt::Debug for CanonicalEnvelopeControlAllocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CanonicalEnvelopeControlAllocation")
            .field("reserved_bytes", &self.memory.bytes())
            .finish()
    }
}
fn arc_bytes<T>() -> Option<u64> {
    let layout = Layout::new::<(usize, usize)>()
        .extend(Layout::new::<T>())
        .ok()?
        .0
        .pad_to_align();
    u64::try_from(layout.size()).ok()
}
