//! Publication invalidates every root of its actual branch occurrence.
use super::*;
use crate::domain_computation::primary_graph::application_query::derived_view::{
    RegisteredDerivedView, ViewChange, ViewPublicationBasis,
};

impl<S: ApplicationSchema + 'static, I: ApplicationManagedIndexBinding<S>> RegisteredDerivedView
    for ManagedIndexState<S, I>
{
    fn dispose(&self) {
        self.close();
    }
    fn belongs_to(&self, basis: &ViewPublicationBasis<'_>) -> bool {
        self.incarnation == basis.incarnation
    }
    fn warm_at(&self, _: &CompositeCommitIdentity) -> bool {
        false
    }
    fn prepare_transition(
        &self,
        _: &CompositeCommitIdentity,
        _: &[ViewChange],
        _: usize,
    ) -> Option<(Box<dyn Any + Send>, usize)> {
        None
    }
    fn apply_transition(&self, _: Option<Box<dyn Any + Send>>, after: &CompositeCommitIdentity) {
        let mut state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.status == WorthQueryManagedIndexLifecycle::Disposed {
            return;
        }
        let next = state.epoch.checked_add(1);
        state.status = if next.is_some() {
            WorthQueryManagedIndexLifecycle::Evicted
        } else {
            WorthQueryManagedIndexLifecycle::Disposed
        };
        if let Some(next) = next {
            state.epoch = next;
        }
        state.current = after.clone();
        let old = state.root.take();
        drop(state);
        drop(old);
    }
}
