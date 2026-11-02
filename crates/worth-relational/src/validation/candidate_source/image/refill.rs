//! Compare and refill exactly the selected cache entry; Source identity is stable.
use super::*;
use crate::mvcc::CompanionPreflightStop as Stop;
use worth_execution::ExecutionRequest;
impl ManagedIndexBranchImage {
    pub(crate) fn refill_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        expected: &Arc<ManagedIndexPublishedSourceRoot>,
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
        if !roots
            .get(&key)
            .is_some_and(|entry| Arc::ptr_eq(entry, expected))
        {
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
