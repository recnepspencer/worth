//! The same admitted page directory with fresh funded demand bookkeeping.
use super::*;
use crate::durability::medium::NativeSourceMemberState;
use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::storage::data::symbol_page_roster::SymbolPageRoster;
use worth_execution::ExecutionRequest;
impl RuntimeSymbolResidency {
    pub fn from_selected_directory(
        roster: SymbolPageRoster<'_>,
        physical: NativeSourceMemberState,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Cause> {
        let memory = request.reserve_retained_memory(arc_size::<Self>()? as u64)?;
        Ok(Arc::new(Self {
            by_id: roster.by_id.clone(),
            by_value: roster.by_value.clone(),
            pages: std::sync::Mutex::new(SymbolPageState {
                data: Default::default(),
                physical,
            }),
            _memory: memory,
        }))
    }
}

fn arc_size<T>() -> Result<usize, Cause> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map(|value| value.0.pad_to_align().size())
        .map_err(|_| Cause::MemoryOverflow)
}
