//! Public registration status attaches actual Native material without building it.
use super::super::source::service::ManagedSourceRoot;
use super::*;
use worth_relational::facade::runtime::ManagedIndexSourceObservation;
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> ManagedIndexState<S, I> {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn attach_maintained(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        MaterialCompletion::new(execution, &self.allocation)
            .arc::<ManagedIndexSourceObservation>()?;
        let image = RetainedIndexImage::Maintained(Arc::new(observation));
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
        drop(old);
        Ok(())
    }
}
