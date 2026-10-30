//! Disposable opaque material belongs to one exact Native image, never its clone.
use super::*;
use crate::mvcc::CompanionPreflightStop as Stop;
use crate::validation::candidate_source::ManagedIndexSourcePreparationDenial as Denial;
use std::collections::BTreeMap;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Default)]
pub(crate) struct PublicCacheDirectory {
    next_id: u64,
    pub(crate) entries: BTreeMap<u64, Arc<PublicCacheEntry>>,
}
pub(crate) struct PublicCacheEntry {
    pub(crate) id: u64,
    pub(crate) payload: Mutex<Option<Payload>>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl ManagedIndexBranchImage {
    pub(crate) fn retain_public_cache(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        payload: &Payload,
    ) -> Result<Arc<PublicCacheEntry>, Denial> {
        // Each entry funds two full ordered nodes. This dominates both leaf and
        // internal backing through deletion/merging, independent of insertion order.
        let node = 11 * size_of::<(u64, Arc<PublicCacheEntry>)>() + 14 * size_of::<usize>();
        let bytes = size_of::<PublicCacheEntry>()
            .checked_add(2 * size_of::<usize>())
            .and_then(|bytes| bytes.checked_add(2 * node))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::Publication(Stop::PreparationMemoryCounterOverflow))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| Denial::Publication(Stop::Execution(cause.into())))?;
        let mut caches = self
            .public_caches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let visits = u64::from(caches.entries.len().checked_ilog2().unwrap_or(0)) + 1;
        execution
            .run_native_with_checkpoint(|context| {
                crate::execution::request_stage::checkpoint(context, visits)
            })
            .map_err(|cause| Denial::Publication(Stop::Execution(cause.into())))?
            .0
            .map_err(|cause| Denial::Publication(Stop::Execution(cause.into())))?;
        let id = caches
            .next_id
            .checked_add(1)
            .ok_or(Denial::PublicCacheIdentityExhausted)?;
        let entry = Arc::new(PublicCacheEntry {
            id,
            payload: Mutex::new(Some(Arc::clone(payload))),
            _memory: memory,
        });
        caches.entries.insert(id, Arc::clone(&entry));
        caches.next_id = id;
        Ok(entry)
    }
}
