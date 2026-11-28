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

pub mod entry;
pub mod eviction;
pub mod inspection;
pub use eviction::ManagedIndexSourceCacheRetirement;
pub mod key;
pub mod public_cache;
pub mod publication;
pub use public_cache::PublicCacheEntry;
pub mod refill;
pub use entry::ManagedIndexSourceAssociation;
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
pub struct HistorySourceDirectory<'a>(std::sync::MutexGuard<'a, Directory>);

impl HistorySourceDirectory<'_> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn selected(
        &self,
        program: &Program,
        scope: EntityId,
    ) -> Result<Option<&ManagedIndexPublishedSourceRoot>, ManagedIndexSourcePreparationDenial> {
        Ok(source_entry(&self.0, program, scope)?.map(Arc::as_ref))
    }

    pub const fn key_storage_bytes() -> usize {
        size_of::<SourceKey>()
    }
}

pub trait ManagedIndexPublishedSourceRootFieldsPort {
    fn owner_association(&self) -> &Arc<ManagedIndexSourceAssociation>;
}

impl ManagedIndexPublishedSourceRootFieldsPort for ManagedIndexPublishedSourceRoot {
    fn owner_association(&self) -> &Arc<ManagedIndexSourceAssociation> {
        &self.association
    }
}

impl ManagedIndexBranchImage {
    pub(crate) fn history_directory(&self) -> HistorySourceDirectory<'_> {
        HistorySourceDirectory(self.directory())
    }
}

pub trait ManagedIndexBranchImageFieldsPort {
    fn owner_public_caches(&self) -> &Mutex<public_cache::PublicCacheDirectory>;
}

impl ManagedIndexBranchImageFieldsPort for ManagedIndexBranchImage {
    fn owner_public_caches(&self) -> &Mutex<public_cache::PublicCacheDirectory> {
        &self.public_caches
    }
}

use crate::mvcc::publication::companion::preflight_stop::CompanionPreflightStop as Stop;
use worth_execution::ExecutionRequest;

#[doc(hidden)]
pub trait ManagedIndexBranchImageOwnerPort: Sized {
    fn history_directory(&self) -> HistorySourceDirectory<'_>;
    fn prepare_source(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        program: &Program,
        scope: EntityId,
        availability: Availability,
        revision: crate::history::data::CommitId,
        changed: bool,
        payload: Option<&Payload>,
    ) -> Result<(), Stop>;
    fn retain_header(execution: &ExecutionRequest<'_, '_>, image: Self) -> Result<Arc<Self>, Stop>;
}

impl ManagedIndexBranchImageOwnerPort for ManagedIndexBranchImage {
    fn history_directory(&self) -> HistorySourceDirectory<'_> {
        self.history_directory()
    }

    fn prepare_source(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        program: &Program,
        scope: EntityId,
        availability: Availability,
        revision: crate::history::data::CommitId,
        changed: bool,
        payload: Option<&Payload>,
    ) -> Result<(), Stop> {
        self.prepare_source(
            execution,
            program,
            scope,
            availability,
            revision,
            changed,
            payload,
        )
    }

    fn retain_header(execution: &ExecutionRequest<'_, '_>, image: Self) -> Result<Arc<Self>, Stop> {
        ManagedIndexBranchImage::retain_header(execution, image)
    }
}
