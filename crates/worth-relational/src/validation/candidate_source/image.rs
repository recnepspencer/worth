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
    ) -> Result<
        Option<Arc<ManagedIndexPublishedSourceRoot>>,
        super::ManagedIndexSourcePreparationDenial,
    > {
        let roots = self.directory();
        Ok(source_entry(&roots, program, scope)?.cloned())
    }
    /// Original-paid retention of exact immutable historical material only.
    pub(crate) fn retained_history_receipt(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        program: &Program,
        scope: EntityId,
        budget: &crate::validation::CustomPreparationBudget,
    ) -> Result<
        Option<super::opaque_custody::FundedSourceMaterialReceipt>,
        super::ManagedIndexSourcePreparationDenial,
    > {
        let roots = self.directory();
        let identity = &program.0.path.parts().identity;
        let overflow = || {
            crate::derived_relation::DerivedRelationReadDenial::Request(
                crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
            )
        };
        // SourceKey compares declared identity strings before fixed-width scope.
        // A comparison cannot inspect more than this supplied key's byte width.
        let width = identity
            .schema
            .0
            .len()
            .checked_add(identity.name.len())
            .and_then(|n| n.checked_add(size_of::<SourceKey>()))
            .ok_or_else(overflow)?;
        let height = 2 * (usize::BITS - roots.len().leading_zeros()) as usize + 2;
        let work = height.checked_mul(width.max(1)).ok_or_else(overflow)?;
        if !budget.try_visit(work) {
            return Err(
                crate::derived_relation::DerivedRelationReadDenial::Definition(
                    crate::derived_relation::DerivedRelationDenial::WorkBudgetExceeded,
                )
                .into(),
            );
        }
        let Some(entry) = source_entry(&roots, program, scope)? else {
            return Ok(None);
        };
        if entry.physical_availability() != Availability::Available {
            return Ok(None);
        }
        let Some(payload) = entry.payload.as_ref() else {
            return Ok(None);
        };
        // Borrow under the actual directory guard: no temporary strong owner
        // can become last if another thread evicts immediately after unlock.
        let receipt =
            super::opaque_custody::FundedSourceMaterialReceipt::retain(execution, payload)
                .map_err(crate::derived_relation::DerivedRelationReadDenial::Request)?;
        Ok(Some(receipt))
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
) -> Result<
    Option<&'a Arc<ManagedIndexPublishedSourceRoot>>,
    super::ManagedIndexSourcePreparationDenial,
> {
    let entry = roots.get(&SourceKey::new(program, scope));
    if entry.is_some_and(|entry| entry.association.program != *program) {
        return Err(super::ManagedIndexSourcePreparationDenial::ForeignSource);
    }
    Ok(entry)
}
