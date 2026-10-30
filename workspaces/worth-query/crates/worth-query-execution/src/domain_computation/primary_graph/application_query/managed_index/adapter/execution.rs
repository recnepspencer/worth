//! One erased entrance on the same typed installed provider allocation.
use super::{InstalledIndexProvider, TypedIndexProvider};
use crate::domain_computation::{
    primary_graph::{
        application_query::managed_index::{
            execution::{self, ProviderExecutionDenial as Denial},
            retention::ManagedIndexState,
            selection::ManagedSelectionLease,
        },
        source_facts::SourceFactsAllocation,
        WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
    },
    provider_session::WorthQueryManagedGraphWorkSession,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::AspectValue;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent;

pub(in crate::domain_computation::primary_graph::application_query::managed_index) trait ErasedIndexExecutor<
    S,
>:
    Send + Sync
{
    fn admits(&self, association: &WorthQueryInstalledManagedIndexParent) -> bool;
    #[allow(clippy::too_many_arguments)]
    fn select<'a, 'request, 'authority>(
        self: Arc<Self>,
        execution: &ExecutionRequest<'request, 'authority>,
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
        parent: &'a WorthQueryManagedGraphWorkSession,
        request: &'a WorthQueryRequestScope,
        association: &WorthQueryInstalledManagedIndexParent,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        probe: &AspectValue,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<ManagedSelectionLease<'a, 'request, 'authority>, Denial>;
}

impl<S, I, P> ErasedIndexExecutor<S> for TypedIndexProvider<S, I, P>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
    P: super::super::WorthQueryManagedIndexProvider<S, I>,
{
    fn admits(&self, association: &WorthQueryInstalledManagedIndexParent) -> bool {
        self.installed.admits_association(association)
    }

    fn select<'a, 'request, 'authority>(
        self: Arc<Self>,
        execution: &ExecutionRequest<'request, 'authority>,
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
        parent: &'a WorthQueryManagedGraphWorkSession,
        request: &'a WorthQueryRequestScope,
        association: &WorthQueryInstalledManagedIndexParent,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        probe: &AspectValue,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<ManagedSelectionLease<'a, 'request, 'authority>, Denial> {
        if !self.installed.admits_association(association) {
            return Err(Denial::ForeignAssociation);
        }
        let adapter: Arc<dyn InstalledIndexProvider<S, I>> = self;
        let candidate =
            ManagedIndexState::new(execution, association.clone(), adapter, parent, selected)
                .map_err(Denial::Lifecycle)?;
        let graph = application
            .runtime
            .primary_graph()
            .ok_or(Denial::ForeignAssociation)?;
        let state = graph
            .managed_derived_views()
            .find_index(execution, &candidate)
            .map_err(Denial::Lifecycle)?;
        // Only the same configured allocation can match this candidate. The
        // registry owns the live state; this temporary header is not a root.
        execution::select(
            execution,
            application,
            parent,
            request,
            association,
            state,
            probe,
            allocation,
        )
    }
}
