//! Conservative cache custody grants no Source membership, invariant audience or READ.
use super::*;
use crate::validation::candidate_source::{
    ManagedIndexBranchImage, ManagedIndexSourcePreparationDenial as Denial, PublicCacheEntry,
};
use std::{collections::BTreeMap, sync::Weak};
use worth_execution::ExecutionRequest;

type Payload = Arc<dyn Any + Send + Sync>;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct ManagedIndexPublicCache(Arc<CacheCustody>);
struct CacheCustody {
    core: Weak<CompanionBranchCellCore>,
    affinity: Arc<managed_source::affinity::ManagedSourceCellAffinity>,
    image: Weak<ManagedIndexBranchImage>,
    entry: Arc<PublicCacheEntry>,
    selected: PositionedRelationalSnapshot,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
    _position_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Drop beyond the caller's runtime, registry and cache-state guards.
#[must_use = "release opaque cache cleanup after leaving owner guards"]
pub struct ManagedIndexPublicCacheRetirement {
    // Release directory backing before its last original entry grant.
    _empty: Option<BTreeMap<u64, Arc<PublicCacheEntry>>>,
    _payload: Option<Payload>,
    _entry: Option<Arc<PublicCacheEntry>>,
}
impl ManagedSourceRootImage {
    pub(crate) fn retain_public_cache(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        selected: PositionedRelationalSnapshot,
        generation: u64,
        payload: &Payload,
        position_memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<ManagedIndexPublicCache, Denial> {
        self.admit_selected(&selected, generation)?;
        let image = Arc::clone(&self.image.payload)
            .downcast::<ManagedIndexBranchImage>()
            .map_err(|_| Denial::ForeignSource)?;
        let bytes = size_of::<CacheCustody>()
            .checked_add(2 * size_of::<usize>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::Publication(
                CompanionPreflightStop::PreparationMemoryCounterOverflow,
            ))?;
        let memory = execution.reserve_retained_memory(bytes).map_err(|cause| {
            Denial::Publication(CompanionPreflightStop::Execution(cause.into()))
        })?;
        let entry = image.retain_public_cache(execution, payload)?;
        Ok(ManagedIndexPublicCache(Arc::new(CacheCustody {
            core: self.core.clone(),
            affinity: Arc::clone(&self.affinity),
            image: Arc::downgrade(&image),
            entry,
            selected,
            _memory: memory,
            _position_memory: position_memory,
        })))
    }
    pub(crate) fn public_cache_payload(
        &self,
        selected: &PositionedRelationalSnapshot,
        generation: u64,
        cache: &ManagedIndexPublicCache,
    ) -> Result<Payload, Denial> {
        self.admit_selected(selected, generation)?;
        cache.0.affinity.admit(selected, generation)?;
        if cache.0.selected != *selected || !Arc::ptr_eq(&self.affinity, &cache.0.affinity) {
            return Err(Denial::RootUnavailable);
        }
        let image = self
            .image
            .payload
            .downcast_ref::<ManagedIndexBranchImage>()
            .ok_or(Denial::ForeignSource)?;
        if !std::ptr::eq(cache.0.image.as_ptr(), image) {
            return Err(Denial::RootUnavailable);
        }
        // This is the exact Native-issued slot, not a reconstructed directory key.
        // Its payload is never replaced/refilled; retirement's take is the sole
        // later mutation and linearizes eviction. Reuse the carried custody.
        cache.payload().ok_or(Denial::RootUnavailable)
    }
}
impl ManagedIndexPublicCache {
    /// Retained material only. Callers must separately validate selected-image custody.
    pub fn payload(&self) -> Option<Payload> {
        self.0
            .entry
            .payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
    /// Cache status at the live Native head; this is neither availability nor READ.
    pub fn is_at_head(&self) -> bool {
        let Some(core) = self.0.core.upgrade() else {
            return false;
        };
        let current = core
            .current
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        current
            .payload
            .downcast_ref::<ManagedIndexBranchImage>()
            .is_some_and(|image| {
                std::ptr::eq(self.0.image.as_ptr(), image)
                    && self
                        .0
                        .entry
                        .payload
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .is_some()
            })
    }
    /// Allocation-free exact-slot retirement. Cleanup must leave all owner guards first.
    pub fn retire(&self) -> ManagedIndexPublicCacheRetirement {
        let mut removed = None;
        let mut empty = None;
        if let Some(image) = self.0.image.upgrade() {
            let mut caches = image
                .public_caches
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if caches
                .entries
                .get(&self.0.entry.id)
                .is_some_and(|entry| Arc::ptr_eq(entry, &self.0.entry))
            {
                removed = caches.entries.remove(&self.0.entry.id);
                if caches.entries.is_empty() {
                    empty = Some(std::mem::take(&mut caches.entries));
                }
            }
        }
        let payload = self
            .0
            .entry
            .payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        ManagedIndexPublicCacheRetirement {
            _payload: payload,
            _entry: removed,
            _empty: empty,
        }
    }
}
