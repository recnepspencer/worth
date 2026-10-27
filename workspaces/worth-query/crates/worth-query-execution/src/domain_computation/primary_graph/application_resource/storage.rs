//! Finite row, Native node and retained-result storage for one installed binding.
mod allocation;
mod result_slot;
mod row_lease;
use super::occurrence::ResourceOccurrence;
use super::provider::IngressBacking;
pub(super) use result_slot::{ResourceResultReservation, RetainedResourceOutput};
pub(super) use row_lease::ResourceRowLease;
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionArray, ExecutionMemoryReservation};
use worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ResourceRowKey {
    index: usize,
    generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ResourceStorageDenial {
    LiveCapacity,
    RowCapacity,
    RetainedResultCapacity,
    ExpiredOccurrence,
    Busy,
    CleanupPending,
    RecoveryHandoffRequired,
}

pub(super) struct NativeResourceSlot {
    pub(super) declaration: LoweredBridgeAsyncSourceDeclaration,
    pub(super) ingress: Arc<IngressBacking>,
    next_free: Mutex<Option<usize>>,
}

struct RowSlot {
    generation: u64,
    next_free: Option<usize>,
    allocated: bool,
    abandoned: bool,
    occurrence: Option<ResourceOccurrence>,
}

struct ResultSlot {
    next_free: Option<usize>,
    owner: Option<ResourceRowKey>,
}

struct FreeSlots {
    row: Option<usize>,
    native: Option<usize>,
    result: Option<usize>,
}

pub(super) struct ResourceStorage {
    pub(super) nodes: ExecutionArray<NativeResourceSlot>,
    rows: ExecutionArray<Mutex<RowSlot>>,
    results: ExecutionArray<Mutex<ResultSlot>>,
    free: Mutex<FreeSlots>,
    _funding: Option<ExecutionMemoryReservation>,
}

impl ResourceStorage {
    /// Used only by the existing explicit runtime-close owner. Busy and
    /// abandoned/recovery rows stay allocated and must block that close.
    pub(in crate::domain_computation::primary_graph::application_resource) fn allocated_rows_retained(
        &self,
    ) -> bool {
        self.rows.iter().any(|slot| lock(slot).allocated)
    }
    /// Called only after the genuine fresh action/facets have been admitted.
    /// The fixed row remains allocated while an action has its payload out.
    pub(super) fn take(
        self: &Arc<Self>,
        key: ResourceRowKey,
    ) -> Result<ResourceRowLease, ResourceStorageDenial> {
        let mut slot = lock(
            self.rows
                .get(key.index)
                .ok_or(ResourceStorageDenial::ExpiredOccurrence)?,
        );
        if !slot.allocated || slot.generation != key.generation {
            return Err(ResourceStorageDenial::ExpiredOccurrence);
        }
        let occurrence = slot.occurrence.take().ok_or(ResourceStorageDenial::Busy)?;
        Ok(ResourceRowLease::new(Arc::clone(self), key, occurrence))
    }

    /// Exact-selector reclaim is Close+Availability admitted before this call.
    /// No search, replacement selector or scope-to-row discovery is performed.
    pub(super) fn take_abandoned(
        self: &Arc<Self>,
        key: ResourceRowKey,
    ) -> Result<ResourceRowLease, ResourceStorageDenial> {
        let mut slot = lock(
            self.rows
                .get(key.index)
                .ok_or(ResourceStorageDenial::ExpiredOccurrence)?,
        );
        if !slot.allocated || slot.generation != key.generation {
            return Err(ResourceStorageDenial::ExpiredOccurrence);
        }
        if !slot.abandoned {
            return Err(ResourceStorageDenial::Busy);
        }
        let occurrence = slot.occurrence.take().ok_or(ResourceStorageDenial::Busy)?;
        // Claim is made only in the final successful handoff, so a later
        // original-request refusal leaves this occurrence reclaimable.
        Ok(ResourceRowLease::new(Arc::clone(self), key, occurrence))
    }

    pub(super) fn claim(&self, key: ResourceRowKey) -> Result<(), ResourceStorageDenial> {
        let mut slot = lock(
            self.rows
                .get(key.index)
                .ok_or(ResourceStorageDenial::ExpiredOccurrence)?,
        );
        if !slot.allocated
            || slot.generation != key.generation
            || slot.occurrence.is_some()
            || !slot.abandoned
        {
            return Err(ResourceStorageDenial::ExpiredOccurrence);
        }
        slot.abandoned = false;
        Ok(())
    }

    /// No provider, cancellation or Native transition happens on public Drop.
    pub(super) fn abandon(&self, key: ResourceRowKey) {
        let Some(slot) = self.rows.get(key.index) else {
            return;
        };
        let mut slot = lock(slot);
        if slot.allocated && slot.generation == key.generation {
            slot.abandoned = true;
        }
    }

    pub(super) fn take_native(&self) -> Result<usize, ResourceStorageDenial> {
        let mut free = lock(&self.free);
        let index = free.native.ok_or(ResourceStorageDenial::LiveCapacity)?;
        free.native = lock(&self.nodes[index].next_free).take();
        Ok(index)
    }

    /// Only genuine lower release and provider/queue cleanup may reach this.
    fn return_native(&self, index: usize) {
        let mut free = lock(&self.free);
        *lock(&self.nodes[index].next_free) = free.native;
        free.native = Some(index);
    }
}

fn lock<T>(value: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    value
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
