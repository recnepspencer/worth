//! Native association identity survives disposable provider material.
use super::availability::ManagedIndexSourceAvailability as Availability;
use super::denial::ManagedIndexSourcePreparationDenial;
use super::program::ManagedIndexSourceProgram as Program;
use crate::identity::data::EntityId;
use std::{
    any::Any,
    sync::{Arc, Mutex},
};
use worth_execution::ExecutionOrderedMap;

mod entry;
mod eviction;
pub(crate) mod inspection;
pub use eviction::ManagedIndexSourceCacheRetirement;
mod key;
pub(crate) mod public_cache;
mod publication;
pub(crate) use public_cache::PublicCacheEntry;
mod refill;
pub(crate) use entry::ManagedIndexSourceAssociation;
use key::SourceKey;
type Payload = Arc<dyn Any + Send + Sync>;
type Directory = ExecutionOrderedMap<SourceKey, Arc<ManagedIndexPublishedSourceRoot>>;

/// Each exact image owns its cache slot. Temporal images and forks share only
/// immutable map cells; clearing current material cannot mutate those shares.
#[derive(Default)]
pub struct ManagedIndexBranchImage {
    roots: Mutex<Directory>,
    pub(crate) public_caches: Mutex<public_cache::PublicCacheDirectory>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl Clone for ManagedIndexBranchImage {
    fn clone(&self) -> Self {
        Self {
            roots: Mutex::new(self.directory().clone()),
            // Conservative public material never becomes a successor or fork root.
            public_caches: Mutex::new(public_cache::PublicCacheDirectory::default()),
            _memory: None,
        }
    }
}
/// Disposable material for one immutable Source association.
pub struct ManagedIndexPublishedSourceRoot {
    pub(crate) association: Arc<ManagedIndexSourceAssociation>,
    payload: Option<Payload>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ManagedIndexBranchImage {
    fn directory(&self) -> std::sync::MutexGuard<'_, Directory> {
        self.roots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    /// Material retention only; this mints neither invariant nor public READ.
    pub fn retained(
        &self,
        program: &Program,
        scope: EntityId,
    ) -> Result<Option<Arc<ManagedIndexPublishedSourceRoot>>, ManagedIndexSourcePreparationDenial>
    {
        let roots = self.directory();
        Ok(source_entry(&roots, program, scope)?.cloned())
    }
    pub(super) fn history_directory(&self) -> HistorySourceDirectory<'_> {
        HistorySourceDirectory(self.directory())
    }
}
impl ManagedIndexPublishedSourceRoot {
    /// None means a cold cache, independently of physical Source availability.
    pub fn payload(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }
    pub fn physical_availability(&self) -> Availability {
        self.association.availability
    }
}

fn source_entry<'a>(
    roots: &'a Directory,
    program: &Program,
    scope: EntityId,
) -> Result<Option<&'a Arc<ManagedIndexPublishedSourceRoot>>, ManagedIndexSourcePreparationDenial> {
    let entry = roots.get(&SourceKey::new(program, scope));
    if entry.is_some_and(|entry| entry.association.program != *program) {
        return Err(ManagedIndexSourcePreparationDenial::ForeignSource);
    }
    Ok(entry)
}

/// Borrow exact immutable source material under its original native directory guard.
pub(super) struct HistorySourceDirectory<'a>(std::sync::MutexGuard<'a, Directory>);

impl HistorySourceDirectory<'_> {
    pub(super) fn len(&self) -> usize {
        self.0.len()
    }

    pub(super) fn selected(
        &self,
        program: &Program,
        scope: EntityId,
    ) -> Result<Option<&ManagedIndexPublishedSourceRoot>, ManagedIndexSourcePreparationDenial> {
        Ok(source_entry(&self.0, program, scope)?.map(Arc::as_ref))
    }

    pub(super) const fn key_storage_bytes() -> usize {
        size_of::<SourceKey>()
    }
}
