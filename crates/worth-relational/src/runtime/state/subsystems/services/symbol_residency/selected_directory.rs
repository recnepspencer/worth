//! The same admitted page directory with fresh funded demand bookkeeping.
use super::*;
use crate::durability::data::{RelationalCheckpointDirectory, RelationalRecoveryStop as Stop};
use worth_execution::ExecutionRequest;
impl RuntimeSymbolResidency {
    pub(in crate::runtime::state::subsystems::services) fn from_selected_directory(
        directory: &RelationalCheckpointDirectory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Stop> {
        let memory = request
            .reserve_retained_memory(arc_size::<Self>()? as u64)
            .map_err(|cause| Stop::Request(cause.into()))?;
        Ok(Arc::new(Self {
            by_id: directory.manifest.symbols.clone(),
            by_value: directory.manifest.symbol_buckets.clone(),
            pages: std::sync::Mutex::new(SymbolPageState {
                data: Default::default(),
                physical: crate::durability::medium::NativeSourceMemberState::new(
                    directory.source.clone(),
                    directory.member_basis(),
                ),
            }),
            _memory: memory,
        }))
    }
}

fn arc_size<T>() -> Result<usize, Stop> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map(|value| value.0.pad_to_align().size())
        .map_err(|_| {
            Stop::Request(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)
        })
}
