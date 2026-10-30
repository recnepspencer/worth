//! Apply the already-judged Source set to one unpublished directory.
use super::*;
use crate::mvcc::CompanionPreflightStop as Stop;
use worth_execution::ExecutionRequest;
impl ManagedIndexBranchImage {
    pub(crate) fn prepare_source(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        program: &Program,
        scope: EntityId,
        availability: Availability,
        revision: crate::history::data::CommitId,
        changed: bool,
        payload: Option<&Payload>,
    ) -> Result<(), Stop> {
        let roots = self
            .roots
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = SourceKey::new(program, scope);
        if roots
            .get(&key)
            .is_some_and(|previous| previous.association.program != *program)
        {
            return Err(Stop::ForeignCell);
        }
        let Some(payload) = payload else {
            roots
                .remove_for_request(&key, *execution)
                .map_err(entry::ordered_denial)?;
            return Ok(());
        };
        let previous = roots.get(&key);
        // Complete unchanged Source custody preserves the condition even when
        // a cold reconstruction creates a different immutable payload Arc.
        let association = match previous {
            Some(previous) if !changed && previous.association.availability == availability => {
                Arc::clone(&previous.association)
            }
            _ => ManagedIndexSourceAssociation::new(
                execution,
                program,
                scope,
                availability,
                revision,
            )?,
        };
        if previous.is_some_and(|previous| {
            Arc::ptr_eq(&previous.association, &association)
                && previous
                    .payload
                    .as_ref()
                    .is_some_and(|old| Arc::ptr_eq(old, payload))
        }) {
            return Ok(());
        }
        let next = ManagedIndexPublishedSourceRoot::new(
            execution,
            association,
            Some(Arc::clone(payload)),
        )?;
        roots
            .insert_cloned_native_for_request(&key, &next, *execution)
            .map_err(entry::ordered_denial)
    }
    pub(crate) fn retain_header(
        execution: &ExecutionRequest<'_, '_>,
        image: Self,
    ) -> Result<Arc<Self>, Stop> {
        // The directory's ordered cells retain their own original grants.
        let bytes = std::mem::size_of::<Self>()
            .checked_add(2 * std::mem::size_of::<usize>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Stop::PreparationMemoryCounterOverflow)?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| Stop::Execution(cause.into()))?;
        Ok(Arc::new(Self {
            roots: image.roots,
            public_caches: image.public_caches,
            _memory: memory,
        }))
    }
}
