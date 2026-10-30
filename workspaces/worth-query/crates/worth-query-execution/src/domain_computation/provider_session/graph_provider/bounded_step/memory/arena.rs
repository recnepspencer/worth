use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::WorthQueryGraphProviderRetainedMemory;
use crate::domain_computation::{
    WorthQueryGraphProviderStepDenial, WorthQueryGraphProviderStepDenialKind,
};

static NEXT_PROVIDER_MEMORY_ARENA: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub(crate) struct WorthQueryGraphProviderMemoryArena {
    state: Arc<WorthQueryGraphProviderMemoryState>,
}

pub(super) struct WorthQueryGraphProviderMemoryState {
    identity: u64,
    retained_bytes_ceiling: u64,
    retained_bytes: AtomicU64,
    retained_allocation_count: AtomicU64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryGraphProviderMemorySnapshot {
    arena_identity: u64,
    retained_bytes: u64,
    retained_allocation_count: u64,
}

impl WorthQueryGraphProviderMemoryArena {
    pub(crate) fn new(retained_bytes_ceiling: u64) -> Self {
        Self {
            state: Arc::new(WorthQueryGraphProviderMemoryState {
                identity: NEXT_PROVIDER_MEMORY_ARENA.fetch_add(1, Ordering::Relaxed),
                retained_bytes_ceiling,
                retained_bytes: AtomicU64::new(0),
                retained_allocation_count: AtomicU64::new(0),
            }),
        }
    }

    pub(crate) fn retain_bytes(
        &self,
        byte_count: usize,
    ) -> Result<WorthQueryGraphProviderRetainedMemory, WorthQueryGraphProviderStepDenial> {
        let bytes = allocate_bytes(byte_count)?;
        let charged_bytes =
            u64::try_from(bytes.capacity()).map_err(|_| retained_budget_denial())?;
        self.reserve(charged_bytes)?;
        self.state.record_allocation();
        Ok(WorthQueryGraphProviderRetainedMemory::new(
            Arc::clone(&self.state),
            bytes,
            charged_bytes,
        ))
    }

    pub(crate) fn snapshot(&self) -> WorthQueryGraphProviderMemorySnapshot {
        WorthQueryGraphProviderMemorySnapshot {
            arena_identity: self.state.identity(),
            retained_bytes: self.state.retained_bytes(),
            retained_allocation_count: self.state.retained_allocation_count(),
        }
    }

    fn reserve(&self, byte_count: u64) -> Result<(), WorthQueryGraphProviderStepDenial> {
        {
            let update_counter = &self.state.retained_bytes;
            let mut current = update_counter.load(Ordering::Acquire);
            loop {
                let Some(updated_value) = current
                    .checked_add(byte_count)
                    .filter(|total| *total <= self.state.retained_bytes_ceiling)
                else {
                    break Err(current);
                };
                match update_counter.compare_exchange_weak(
                    current,
                    updated_value,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => current = reloaded_value,
                }
            }
        }
        .map(|_| ())
        .map_err(|_| retained_budget_denial())
    }
}

impl WorthQueryGraphProviderMemorySnapshot {
    pub(crate) const fn arena_identity(self) -> u64 {
        self.arena_identity
    }

    pub(crate) const fn retained_bytes(self) -> u64 {
        self.retained_bytes
    }

    pub(crate) const fn retained_allocation_count(self) -> u64 {
        self.retained_allocation_count
    }
}

impl WorthQueryGraphProviderMemoryState {
    pub(super) const fn identity(&self) -> u64 {
        self.identity
    }

    pub(super) fn record_allocation(&self) {
        self.retained_allocation_count
            .fetch_add(1, Ordering::AcqRel);
    }

    pub(super) fn release(&self, byte_count: u64) {
        let previous = self.retained_bytes.fetch_sub(byte_count, Ordering::AcqRel);
        debug_assert!(previous >= byte_count);
        let previous_count = self
            .retained_allocation_count
            .fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous_count > 0);
    }

    fn retained_bytes(&self) -> u64 {
        self.retained_bytes.load(Ordering::Acquire)
    }

    fn retained_allocation_count(&self) -> u64 {
        self.retained_allocation_count.load(Ordering::Acquire)
    }
}

fn allocate_bytes(byte_count: usize) -> Result<Vec<u8>, WorthQueryGraphProviderStepDenial> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(byte_count).map_err(|_| {
        WorthQueryGraphProviderStepDenial::new(
            WorthQueryGraphProviderStepDenialKind::MemoryAllocationFailed,
            "provider retained-memory allocation failed",
        )
    })?;
    bytes.resize(byte_count, 0);
    Ok(bytes)
}

fn retained_budget_denial() -> WorthQueryGraphProviderStepDenial {
    WorthQueryGraphProviderStepDenial::new(
        WorthQueryGraphProviderStepDenialKind::RetainedBudgetExceeded,
        "provider retained memory exceeds the installed retained budget",
    )
}
