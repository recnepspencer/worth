//! Native association identity survives disposable provider material.
use super::{ManagedIndexSourceAvailability as Availability, ManagedIndexSourceProgram as Program};
use crate::identity::data::EntityId;
use std::{
    any::Any,
    sync::{Arc, Mutex},
};
use worth_execution::ExecutionOrderedMap;

mod entry;
mod eviction;
pub use eviction::ManagedIndexSourceCacheRetirement;
mod key;
mod publication;
pub(crate) use entry::ManagedIndexSourceAssociation;
use key::SourceKey;
type Payload = Arc<dyn Any + Send + Sync>;
type Directory = ExecutionOrderedMap<SourceKey, Arc<ManagedIndexPublishedSourceRoot>>;

/// Each exact image owns its cache slot. Temporal images and forks share only
/// immutable map cells; clearing current material cannot mutate those shares.
#[derive(Default)]
pub struct ManagedIndexBranchImage {
    roots: Mutex<Directory>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl Clone for ManagedIndexBranchImage {
    fn clone(&self) -> Self {
        Self {
            roots: Mutex::new(self.directory().clone()),
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
    ) -> Result<
        Option<Arc<ManagedIndexPublishedSourceRoot>>,
        super::ManagedIndexSourcePreparationDenial,
    > {
        let entry = self
            .directory()
            .get(&SourceKey::new(program, scope))
            .cloned();
        if entry
            .as_ref()
            .is_some_and(|entry| entry.association.program != *program)
        {
            return Err(super::ManagedIndexSourcePreparationDenial::ForeignSource);
        }
        Ok(entry)
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
