//! Isolated dictionary controls share the actual admitted immutable directory.
use super::*;
use crate::durability::medium::NativeSourceMemberState;
use crate::storage::data::symbol_page_roster::SymbolPageRoster;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

impl RuntimeSymbolTable {
    pub fn from_selected_directory_for_request(
        &self,
        roster: SymbolPageRoster<'_>,
        physical: NativeSourceMemberState,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, ColdSymbolDirectoryDenial> {
        crate::execution::request_custody::check_request(request)
            .map_err(ColdSymbolDirectoryDenial::Request)?;
        let bytes = arc_size::<RwLock<RuntimeSymbolState>>()?
            .checked_add(arc_size::<
                RwLock<Option<Arc<crate::storage::data::symbol_residency::RuntimeSymbolResidency>>>,
            >()?)
            .and_then(|n| {
                n.checked_add(
                    arc_size::<std::sync::Mutex<reservations::SymbolReservationDirectory>>()
                        .ok()?,
                )
            })
            .and_then(|n| n.checked_add(arc_size::<ExecutionMemoryReservation>().ok()?))
            .ok_or(ColdSymbolDirectoryDenial::Request(
                crate::execution::denial_cause::RelationalExecutionDenialCause::MemoryOverflow,
            ))?;
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| ColdSymbolDirectoryDenial::Request(cause.into()))?
            .map(Arc::new);
        // The resident persistent roots share their intrinsic nodes. No mutable
        // page ledger or size-dependent directory is copied from the live table.
        let mut state = self.read_state(Clone::clone);
        state
            .interner
            .reserve_frontier(roster.frontier)
            .map_err(ColdSymbolDirectoryDenial::Invalid)?;
        let pages =
            crate::storage::data::symbol_residency::RuntimeSymbolResidency::from_selected_directory(
                roster, physical, request,
            )
            .map_err(ColdSymbolDirectoryDenial::Request)?;
        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            cold: Arc::new(RwLock::new(Some(pages))),
            reservations: Arc::default(),
            _memory: memory,
        })
    }
}

fn arc_size<T>() -> Result<usize, ColdSymbolDirectoryDenial> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map(|value| value.0.pad_to_align().size())
        .map_err(|_| {
            ColdSymbolDirectoryDenial::Request(
                crate::execution::denial_cause::RelationalExecutionDenialCause::MemoryOverflow,
            )
        })
}
