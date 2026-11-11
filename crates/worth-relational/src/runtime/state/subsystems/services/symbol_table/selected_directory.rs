//! Isolated dictionary controls share the actual admitted immutable directory.
use super::*;
use crate::durability::data::{RelationalCheckpointDirectory, RelationalRecoveryStop as Stop};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

impl RuntimeSymbolTable {
    pub(crate) fn from_selected_directory_for_request(
        &self,
        directory: &RelationalCheckpointDirectory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Stop> {
        crate::execution::check_request(request).map_err(Stop::Request)?;
        let bytes = arc_size::<RwLock<RuntimeSymbolState>>()?
            .checked_add(arc_size::<
                RwLock<Option<Arc<super::super::symbol_residency::RuntimeSymbolResidency>>>,
            >()?)
            .and_then(|n| {
                n.checked_add(
                    arc_size::<std::sync::Mutex<reservations::SymbolReservationDirectory>>()
                        .ok()?,
                )
            })
            .and_then(|n| n.checked_add(arc_size::<ExecutionMemoryReservation>().ok()?))
            .ok_or(Stop::Request(
                crate::execution::RelationalExecutionDenialCause::MemoryOverflow,
            ))?;
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| Stop::Request(cause.into()))?
            .map(Arc::new);
        // The resident persistent roots share their intrinsic nodes. No mutable
        // page ledger or size-dependent directory is copied from the live table.
        let mut state = self.read_state(Clone::clone);
        state
            .interner
            .reserve_frontier(directory.manifest.symbol_identity_frontier)
            .map_err(crate::durability::checkpoints::region_directory::corrupt)?;
        let pages =
            super::super::symbol_residency::RuntimeSymbolResidency::from_selected_directory(
                directory, request,
            )?;
        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            cold: Arc::new(RwLock::new(Some(pages))),
            reservations: Arc::default(),
            _memory: memory,
        })
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
