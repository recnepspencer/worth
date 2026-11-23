//! Compare and refill exactly the selected cache entry; Source identity is stable.
use super::*;
use crate::mvcc::CompanionPreflightStop as Stop;
use worth_execution::ExecutionRequest;
impl ManagedIndexBranchImage {
    pub(crate) fn plan_cold_source(
        execution: &ExecutionRequest<'_, '_>,
        program: &Program,
        scope: EntityId,
        revision: crate::history::data::CommitId,
    ) -> Result<Arc<ManagedIndexPublishedSourceRoot>, Stop> {
        let association = ManagedIndexSourceAssociation::new(
            execution,
            program,
            scope,
            Availability::Available,
            revision,
        )?;
        ManagedIndexPublishedSourceRoot::new(execution, association, None)
    }

    pub(crate) fn refill_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        expected: &Arc<ManagedIndexPublishedSourceRoot>,
        expected_placed: bool,
        payload: &Payload,
    ) -> Result<ManagedIndexSourceCacheRetirement, Stop> {
        let key = SourceKey::new(&expected.association.program, expected.association.scope);
        // Allocate before taking the cache lock. The caller retains payload and
        // expected backing throughout refusal, including allocator failure.
        let replacement = ManagedIndexPublishedSourceRoot::new(
            execution,
            Arc::clone(&expected.association),
            Some(Arc::clone(payload)),
        )?;
        let mut roots = self.directory();
        let current = roots.get(&key);
        let matches = match current {
            Some(entry) => expected_placed && Arc::ptr_eq(entry, expected),
            None => !expected_placed,
        };
        if !matches {
            return Err(Stop::ManagedSourceChanged);
        }
        let mut next = roots.clone();
        next.insert_cloned_native_for_request(&key, &replacement, *execution)
            .map_err(entry::ordered_denial)?;
        let old = std::mem::replace(&mut *roots, next);
        drop(roots);
        Ok(ManagedIndexSourceCacheRetirement { _directory: old })
    }
}
