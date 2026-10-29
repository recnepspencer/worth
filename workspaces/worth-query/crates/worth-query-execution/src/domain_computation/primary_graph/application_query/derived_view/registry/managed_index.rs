//! Deterministic typed association reuse in the existing shared 64-entry registry.
use super::*;
use crate::domain_computation::primary_graph::application_query::{
    managed_index::retention::{
        ManagedIndexState, WorthQueryManagedIndexLifecycle as Status,
        WorthQueryManagedIndexLifecycleDenial as Denial,
    },
    one_shot::outcome::material::MaterialCompletion,
};
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

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
        let visits = state
            .owned
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(size_of::<OwnedDerivedView>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::WorkCounterOverflow)?;
        MaterialCompletion::new(execution, &candidate.allocation).work(visits)?;
        for (_, entry) in &state.owned {
            if let OwnedDerivedView::ManagedIndex { state: found, .. } = entry {
                if found
                    .downcast_ref::<ManagedIndexState<S, I>>()
                    .is_some_and(|actual| actual.same_association(candidate))
                {
                    return Arc::clone(found)
                        .downcast::<ManagedIndexState<S, I>>()
                        .map_err(|_| Denial::ForeignAssociation);
                }
            }
        }
        Err(Denial::NotReady)
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
            let visits = state
                .owned
                .len()
                .checked_add(state.views.len())
                .and_then(|n| n.checked_add(1))
                .and_then(|n| n.checked_mul(size_of::<OwnedDerivedView>()))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(Cause::WorkCounterOverflow)?;
            MaterialCompletion::new(execution, &candidate.allocation).work(visits)?;
            if let Some(position) = state.owned.iter().position(|(_, entry)| match entry {
                OwnedDerivedView::ManagedIndex { state, .. } => state
                    .downcast_ref::<ManagedIndexState<S, I>>()
                    .is_some_and(|found| found.same_association(&candidate)),
                OwnedDerivedView::NativeCount(_) => false,
            }) {
                let (id, OwnedDerivedView::ManagedIndex { state: actual, .. }) =
                    &state.owned[position]
                else {
                    unreachable!("matched Index association");
                };
                let actual = Arc::clone(actual)
                    .downcast::<ManagedIndexState<S, I>>()
                    .map_err(|_| Denial::ForeignAssociation)?;
                if actual.lifecycle() != Status::Disposed {
                    return Ok((*id, actual));
                }
                let id = *id;
                retired = Some(state.owned.remove(position));
                state.views.remove(&id);
            }
            state.views.retain(|_, view| view.strong_count() > 0);
            if state.views.len() >= MAXIMUM_REGISTERED_VIEWS {
                return Err(Denial::RegistrationCapacityExceeded);
            }
            let id = state
                .next_id
                .checked_add(1)
                .ok_or(Denial::RegistrationIdentityExhausted)?;
            state.admit_index(execution, &candidate.allocation)?;
            let publication: Arc<dyn RegisteredDerivedView> = candidate.clone();
            let erased: Arc<dyn Any + Send + Sync> = candidate.clone();
            state.views.insert(id, Arc::downgrade(&publication));
            state.owned.push((
                id,
                OwnedDerivedView::ManagedIndex {
                    state: erased,
                    _publication: publication,
                },
            ));
            state.next_id = id;
            Ok((id, Arc::clone(&candidate)))
        })();
        let empty = state.take_empty_material_storage();
        drop(state);
        // This includes every failed/unused candidate and a disposed prior
        // registration; none can run final root destruction under the mutex.
        drop(retired);
        drop(empty);
        result
    }
}
