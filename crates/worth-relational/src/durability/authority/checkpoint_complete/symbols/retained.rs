//! Concrete im15.1 traversal scratch, admitted before its private Vec paths grow.
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
use crate::symbols::data::{StoredSymbolText, StringInterner, Symbol};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(super) struct RetainedSymbols<'a> {
    entries: im::ordmap::Iter<'a, Symbol, StoredSymbolText>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl<'a> RetainedSymbols<'a> {
    pub(super) fn new(
        source: &'a StringInterner,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let bytes = scratch_bound(source.retained_entry_count()).ok_or_else(overflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(|cause| Denial::Request(cause.into()))?;
        Ok(Self {
            entries: source.retained_entries(),
            _memory: memory,
        })
    }
}
impl<'a> Iterator for RetainedSymbols<'a> {
    type Item = (&'a Symbol, &'a StoredSymbolText);
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next()
    }
}
// Pinned im15.1 btree::Iter owns two Vec<(&Node,usize)> paths. Every internal
// node has at least two children, so bit_width(len) bounds path depth. Vec's
// actual frame is a thin reference plus usize; capacities grow from four by
// doubling. Include one old half-capacity path during realloc overlap. This is
// a conservative scratch bound, not a claim about this tree's actual height.
fn scratch_bound(len: usize) -> Option<u64> {
    if len == 0 {
        return Some(0);
    }
    let depth = usize::BITS.checked_sub(len.leading_zeros())? as usize;
    let capacity = depth.checked_next_power_of_two()?.max(4);
    let overlap = if capacity == 4 { 0 } else { capacity / 2 };
    let frames = capacity.checked_mul(2)?.checked_add(overlap)?;
    u64::try_from(
        std::alloc::Layout::array::<(&(), usize)>(frames)
            .ok()?
            .size(),
    )
    .ok()
}
fn overflow() -> Denial {
    Denial::Request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}
#[cfg(test)]
mod tests;
