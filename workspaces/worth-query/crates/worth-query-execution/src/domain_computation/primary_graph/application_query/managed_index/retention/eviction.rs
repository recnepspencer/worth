//! Admit cleanup before effects; completion cannot fail after Native eviction.
use super::*;
use WorthQueryManagedIndexLifecycle as Status;
use WorthQueryManagedIndexLifecycleDenial as Denial;
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> ManagedIndexState<S, I> {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn admit_eviction(
        &self,
    ) -> Result<(u64, u64), Denial> {
        let state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.status == Status::Disposed {
            return Err(Denial::Disposed);
        }
        if state.status == Status::Rebuilding {
            return Err(Denial::BuildInProgress);
        }
        let next = state.epoch.checked_add(1).ok_or(Denial::EpochExhausted)?;
        Ok((state.epoch, next))
    }
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn finish_eviction(
        &self,
        epoch: u64,
        next: u64,
    ) {
        let mut state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // A concurrent close or a freshly attached successor cannot turn
        // completed Native cleanup into a no-effect refusal or dispose it.
        let old = if state.epoch == epoch && state.status != Status::Disposed {
            state.epoch = next;
            state.status = Status::Evicted;
            state.root.take()
        } else {
            None
        };
        drop(state);
        if let Some(old) = old {
            old.retire();
        }
    }
}
