//! Point routing reuses only the exact typed installed association.
use super::index_directory::RegisteredIndex;
use super::*;
use crate::domain_computation::primary_graph::application_query::{
    managed_index::retention::{
        ManagedIndexState, WorthQueryManagedIndexLifecycleDenial as Denial,
    },
    one_shot::outcome::material::MaterialCompletion,
};
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};

impl ManagedDerivedViewRegistry {
    /// Ordinary execution never creates a missing registration or rebuilds it.
    pub(in crate::domain_computation::primary_graph::application_query) fn find_index<S, I>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &Arc<ManagedIndexState<S, I>>,
    ) -> Result<Arc<ManagedIndexState<S, I>>, Denial>
    where
        S: ApplicationSchema + 'static,
        I: ApplicationManagedIndexBinding<S>,
    {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        MaterialCompletion::new(execution, &candidate.allocation)
            .work(state.indexes.lookup_work()?)?;
        let (_, found) = state
            .indexes
            .get(&candidate.registration_key())
            .ok_or(Denial::NotReady)?;
        let actual = Arc::clone(&found.state)
            .downcast::<ManagedIndexState<S, I>>()
            .map_err(|_| Denial::ForeignAssociation)?;
        if !actual.same_association(candidate) {
            return Err(Denial::ForeignAssociation);
        }
        Ok(actual)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn register_index<S, I>(
        self: &Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: Arc<ManagedIndexState<S, I>>,
    ) -> Result<(u64, Arc<ManagedIndexState<S, I>>), Denial>
    where
        S: ApplicationSchema + 'static,
        I: ApplicationManagedIndexBinding<S>,
    {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut retired = None;
        let result = (|| {
            MaterialCompletion::new(execution, &candidate.allocation)
                .work(state.indexes.lookup_work()?)?;
            let key = candidate.registration_key();
            let old_id = if let Some((id, found)) = state.indexes.get(&key) {
                let actual = Arc::clone(&found.state)
                    .downcast::<ManagedIndexState<S, I>>()
                    .map_err(|_| Denial::ForeignAssociation)?;
                if !actual.same_association(&candidate) {
                    return Err(Denial::ForeignAssociation);
                }
                if !actual.is_disposed() {
                    return Ok((id, actual));
                }
                Some(id)
            } else {
                None
            };
            state.views.retain(|_, view| view.strong_count() > 0);
            if old_id.is_none() && state.registration_count() >= MAXIMUM_REGISTERED_VIEWS {
                return Err(Denial::RegistrationCapacityExceeded);
            }
            let id = state
                .next_id
                .checked_add(1)
                .ok_or(Denial::RegistrationIdentityExhausted)?;
            // Insert follows the same two indexed paths; replacement also
            // removes the old pair. Admit every traversal before mutation.
            let changes = if old_id.is_some() { 2 } else { 1 };
            let maintenance = state.indexes.lookup_work()?.checked_mul(changes)
                .ok_or(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow)?;
            MaterialCompletion::new(execution, &candidate.allocation).work(maintenance)?;
            let prepared =
                RegisteredIndex::prepare(execution, key, candidate.clone(), candidate.clone())?;
            // Every fallible admission precedes this atomic metadata cutover.
            if let Some(old) = old_id {
                retired = state.indexes.remove(old);
            }
            state.indexes.insert(id, prepared);
            state.next_id = id;
            Ok((id, Arc::clone(&candidate)))
        })();
        drop(state);
        drop(retired);
        result
    }
}
