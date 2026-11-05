//! Public registration status attaches actual Native material without building it.
use super::super::source::service::ManagedSourceRoot;
use super::*;
use worth_relational::facade::runtime::ManagedIndexSourceObservation;
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> ManagedIndexState<S, I> {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn attach_maintained(
        &self,
        observation: ManagedIndexSourceObservation,
    ) -> Result<(), WorthQueryManagedIndexLifecycleDenial> {
        use WorthQueryManagedIndexLifecycleDenial as Denial;
        let root = observation
            .payload()
            .downcast_ref::<ManagedSourceRoot<S, I>>()
            .ok_or(Denial::ForeignAssociation)?;
        if !root.installed.same_installation(self.adapter.installed())
            || !Arc::ptr_eq(&root.adapter, &self.adapter)
        {
            return Err(Denial::ForeignAssociation);
        }
        let image = RetainedIndexImage::Maintained {
            native_custody: observation,
        };
        let mut state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.status == WorthQueryManagedIndexLifecycle::Disposed {
            return Err(Denial::Disposed);
        }
        state.epoch = state.epoch.checked_add(1).ok_or(Denial::EpochExhausted)?;
        let old = state.root.replace(image);
        state.status = WorthQueryManagedIndexLifecycle::Ready;
        drop(state);
        if let Some(old) = old {
            old.retire();
        }
        Ok(())
    }
}

impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> ManagedIndexState<S, I> {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn admit_build(
        &self,
        epoch: u64,
        proof: &crate::domain_computation::provider_session::WorthQueryManagedIndexSelectionProof<
            '_,
            '_,
            '_,
        >,
        parent: &WorthQueryManagedGraphWorkSession,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        terminal: &Arc<crate::domain_computation::provider_session::WorthQueryCompletedSourceRead>,
    ) -> Result<(), WorthQueryManagedIndexLifecycleDenial> {
        use WorthQueryManagedIndexLifecycleDenial as Denial;
        proof.consult().map_err(Denial::Session)?;
        if !proof.admits(parent, request, &self.association, terminal)
            || terminal.runtime() != self.runtime
            || terminal.principal() != self.principal
            || terminal.scope() != self.scope
        {
            return Err(Denial::ForeignAssociation);
        }
        let super::super::super::WorthQueryApplicationBasisSelectionIdentity::Product(product) =
            terminal.basis().selection()
        else {
            return Err(Denial::ForeignAssociation);
        };
        let state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.status == WorthQueryManagedIndexLifecycle::Disposed {
            return Err(Denial::Disposed);
        }
        if state.epoch != epoch
            || state.status != WorthQueryManagedIndexLifecycle::Rebuilding
            || product.lifecycle_incarnation() != self.incarnation
            || product.selected_commit() != &state.current
        {
            return Err(Denial::StaleImage);
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn finish_maintained_build(
        &self,
        epoch: u64,
        observation: ManagedIndexSourceObservation,
    ) {
        let image = RetainedIndexImage::Maintained {
            native_custody: observation,
        };
        let mut state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let old = if state.epoch == epoch
            && state.status == WorthQueryManagedIndexLifecycle::Rebuilding
        {
            state.status = WorthQueryManagedIndexLifecycle::Ready;
            state.root.replace(image)
        } else {
            Some(image)
        };
        drop(state);
        if let Some(old) = old {
            old.retire();
        }
    }
}
