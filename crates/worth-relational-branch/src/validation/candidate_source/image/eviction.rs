//! Current-image cache edits retain retired material until Native guards end.
use super::*;
use crate::mvcc::publication::companion::preflight_stop::CompanionPreflightStop as Stop;
use worth_execution::ExecutionRequest;

/// Completed cleanup retains old backing solely for destruction after the
/// caller releases its runtime/registry guards. This grants no Source READ.
#[must_use = "release retired backing after leaving runtime guards"]
pub struct ManagedIndexSourceCacheRetirement {
    pub(super) _directory: Directory,
}
impl ManagedIndexBranchImage {
    pub(crate) fn evict_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &Program,
        scope: EntityId,
    ) -> Result<Option<ManagedIndexSourceCacheRetirement>, Stop> {
        let mut roots = self.directory();
        let key = SourceKey::new(program, scope);
        let previous = roots.get(&key).ok_or(Stop::ManagedSourceNotPrepared)?;
        if previous.association.program != *program {
            return Err(Stop::ForeignCell);
        }
        if previous.association.availability != Availability::Available {
            return Err(Stop::ManagedSourceNotPrepared);
        }
        if previous.payload.is_none() {
            return Ok(None);
        }
        let cold = ManagedIndexPublishedSourceRoot::new(
            execution,
            Arc::clone(&previous.association),
            None,
        )?;
        // Keep the complete original map through replacement/error. No provider
        // payload's final Drop can run under this mutex or the caller's guards.
        let mut next = roots.clone();
        next.insert_cloned_native_for_request(&key, &cold, *execution)
            .map_err(entry::ordered_denial)?;
        let old = std::mem::replace(&mut *roots, next);
        drop(roots);
        Ok(Some(ManagedIndexSourceCacheRetirement { _directory: old }))
    }
}

impl ManagedIndexBranchImage {
    #[doc(hidden)]
    pub(crate) fn try_directory_for_test(&self) -> bool {
        self.roots.try_lock().is_ok()
    }

    #[doc(hidden)]
    pub(crate) fn evict_source_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &Program,
        scope: EntityId,
    ) -> Result<Option<ManagedIndexSourceCacheRetirement>, Stop> {
        self.evict_source(execution, program, scope)
    }
}

#[doc(hidden)]
pub trait ManagedIndexBranchImageFixturePort: Sized {
    fn try_directory_for_test(&self) -> bool;
    fn evict_source_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &Program,
        scope: EntityId,
    ) -> Result<Option<ManagedIndexSourceCacheRetirement>, Stop>;
}

impl ManagedIndexBranchImageFixturePort for ManagedIndexBranchImage {
    fn try_directory_for_test(&self) -> bool {
        self.try_directory_for_test()
    }

    fn evict_source_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &Program,
        scope: EntityId,
    ) -> Result<Option<ManagedIndexSourceCacheRetirement>, Stop> {
        self.evict_source_for_test(execution, program, scope)
    }
}
