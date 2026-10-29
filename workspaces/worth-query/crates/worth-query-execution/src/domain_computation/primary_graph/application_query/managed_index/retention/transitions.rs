//! Checked cache transitions; ready roots and callbacks never drop under its lock.
use super::*;
use crate::domain_computation::provider_session::WorthQueryManagedIndexSelectionProof;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use WorthQueryManagedIndexLifecycle as Status;
use WorthQueryManagedIndexLifecycleDenial as Denial;

impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> ManagedIndexState<S, I> {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn holds_ready(
        &self,
        epoch: u64,
        root: &Arc<ReadyIndex<S, I>>,
    ) -> bool {
        let state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.status == Status::Ready
            && state.epoch == epoch
            && state
                .root
                .as_ref()
                .is_some_and(|actual| Arc::ptr_eq(actual, root))
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn begin_build(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
    ) -> Result<u64, Denial> {
        execution
            .consult()
            .map_err(|cause| Denial::Execution(cause.into()))?;
        if selected.application().runtime.authority_identity() != self.runtime
            || selected.product().observation().lifecycle_incarnation() != self.incarnation
            || !self
                .adapter
                .installed()
                .is_installed_in(selected.application().installed_schema())
        {
            return Err(Denial::ForeignAssociation);
        }
        let mut state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.status == Status::Disposed {
            return Err(Denial::Disposed);
        }
        if state.status == Status::Rebuilding {
            return Err(Denial::BuildInProgress);
        }
        let epoch = state.epoch.checked_add(1).ok_or(Denial::EpochExhausted)?;
        state.epoch = epoch;
        // Explicit cold rebuild binds the genuinely fresh selected occurrence;
        // no old Source completion or cached node is reinterpreted as current.
        state.current = selected.product().selected_commit().clone();
        state.status = Status::Rebuilding;
        let old = state.root.take();
        drop(state);
        drop(old);
        Ok(epoch)
    }

    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn finish_build(
        &self,
        epoch: u64,
        root: Arc<ReadyIndex<S, I>>,
        proof: &WorthQueryManagedIndexSelectionProof<'_, '_, '_>,
        parent: &WorthQueryManagedGraphWorkSession,
        request: &WorthQueryRequestScope,
    ) -> Result<(), Denial> {
        proof.consult().map_err(Denial::Session)?;
        if !proof.admits(parent, request, &self.association, &root.source.terminal)
            || !root
                .source
                .installed
                .same_installation(self.adapter.installed())
            || root.source.terminal.runtime() != self.runtime
            || root.source.terminal.principal() != self.principal
            || root.source.terminal.scope() != self.scope
        {
            return Err(Denial::ForeignAssociation);
        }
        let super::super::super::WorthQueryApplicationBasisSelectionIdentity::Product(product) =
            root.source.terminal.basis().selection()
        else {
            return Err(Denial::ForeignAssociation);
        };
        let mut state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.status == Status::Disposed {
            return Err(Denial::Disposed);
        }
        if state.epoch != epoch
            || state.status != Status::Rebuilding
            || product.lifecycle_incarnation() != self.incarnation
            || product.selected_commit() != &state.current
        {
            return Err(Denial::StaleImage);
        }
        let old = state.root.replace(root);
        state.status = Status::Ready;
        drop(state);
        drop(old);
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn failed(
        &self,
        epoch: u64,
    ) {
        let mut state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.epoch == epoch && state.status == Status::Rebuilding {
            state.status = Status::Failed;
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn ready(
        &self,
    ) -> Result<(u64, Arc<ReadyIndex<S, I>>), Denial> {
        let state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.status == Status::Disposed {
            return Err(Denial::Disposed);
        }
        if state.status != Status::Ready {
            return Err(Denial::NotReady);
        }
        Ok((
            state.epoch,
            Arc::clone(state.root.as_ref().ok_or(Denial::NotReady)?),
        ))
    }

    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn evict(
        &self,
    ) -> Result<(), Denial> {
        let mut state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.status == Status::Disposed {
            return Err(Denial::Disposed);
        }
        let next = state.epoch.checked_add(1);
        let old = state.root.take();
        state.status = if next.is_some() {
            Status::Evicted
        } else {
            Status::Disposed
        };
        if let Some(next) = next {
            state.epoch = next;
        }
        drop(state);
        drop(old);
        next.map(|_| ()).ok_or(Denial::EpochExhausted)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn close(&self) {
        let mut state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.status = Status::Disposed;
        // Terminal disposal is irreversible even when the epoch cannot advance.
        if let Some(next) = state.epoch.checked_add(1) {
            state.epoch = next;
        }
        let old = state.root.take();
        drop(state);
        drop(old);
    }
}
