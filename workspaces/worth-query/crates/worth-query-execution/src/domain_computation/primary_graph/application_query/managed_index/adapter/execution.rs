//! One erased entrance on the same typed installed provider allocation.
use super::{InstalledIndexProvider, TypedIndexProvider};
use crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance;
use crate::domain_computation::primary_graph::application_query::{
    output_source_observation::SourceQueryReaderAccess, ApplicationQueryNativeAccess,
    WorthQueryObservedScopeSelector,
};
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
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters as Parameters;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
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
    fn predicate_factory(&self) -> Option<&Arc<dyn super::super::predicate::PredicateFactory>>;
    #[allow(clippy::too_many_arguments)]
    fn select<'a, 'request, 'authority>(
        self: Arc<Self>,
        execution: &ExecutionRequest<'request, 'authority>,
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
        parent: &'a WorthQueryManagedGraphWorkSession,
        request: &'a WorthQueryRequestScope,
        association: &WorthQueryInstalledManagedIndexParent,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        selector: &WorthQueryObservedScopeSelector,
        probe: &AspectValue,
        parameters: &Parameters,
        governance: &WorthQueryApplicationQueryGovernance,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<ManagedSelectionLease<'a, 'request, 'authority>, Denial>;
    #[allow(clippy::too_many_arguments)]
    fn select_admitted<'a, 'request, 'authority>(
        self: Arc<Self>,
        execution: &ExecutionRequest<'request, 'authority>,
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
        access: &SourceQueryReaderAccess<'_, S>,
        parent: &'a WorthQueryManagedGraphWorkSession,
        request: &'a WorthQueryRequestScope,
        association: &WorthQueryInstalledManagedIndexParent,
        selector: &WorthQueryObservedScopeSelector,
        probe: &AspectValue,
        parameters: &Parameters,
        governance: &WorthQueryApplicationQueryGovernance,
        allocation: &Arc<SourceFactsAllocation>,
        native: &mut ApplicationQueryNativeAccess<'_>,
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

    fn predicate_factory(&self) -> Option<&Arc<dyn super::super::predicate::PredicateFactory>> {
        self.predicate.as_ref()
    }

    fn select<'a, 'request, 'authority>(
        self: Arc<Self>,
        execution: &ExecutionRequest<'request, 'authority>,
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
        parent: &'a WorthQueryManagedGraphWorkSession,
        request: &'a WorthQueryRequestScope,
        association: &WorthQueryInstalledManagedIndexParent,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        selector: &WorthQueryObservedScopeSelector,
        probe: &AspectValue,
        parameters: &Parameters,
        governance: &WorthQueryApplicationQueryGovernance,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<ManagedSelectionLease<'a, 'request, 'authority>, Denial> {
        if !self.installed.admits_association(association) {
            return Err(Denial::ForeignAssociation);
        }
        let adapter: Arc<dyn InstalledIndexProvider<S, I>> = self;
        let graph = application
            .runtime
            .primary_graph()
            .ok_or(Denial::ForeignAssociation)?;
        if graph
            .layout
            .managed_source(&adapter.installed().definition().parts().name)
            .is_some()
        {
            let shared = selected
                .retain_selection(execution)
                .map_err(Denial::Product)?
                .prepare_shared_query_basis()
                .map_err(|(_, cause)| Denial::NativeAdmission(cause))?;
            let access = SourceQueryReaderAccess {
                application,
                shared: &shared,
                principal: external,
                request,
            };
            return super::super::maintained_read::select::<S, I>(
                execution,
                application,
                &access,
                parent,
                request,
                association,
                selector,
                &adapter,
                probe,
                parameters,
                governance,
                allocation,
                &mut ApplicationQueryNativeAccess::acquire(),
            );
        }
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
            selected,
            state,
            probe,
            allocation,
        )
    }

    fn select_admitted<'a, 'request, 'authority>(
        self: Arc<Self>,
        execution: &ExecutionRequest<'request, 'authority>,
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
        access: &SourceQueryReaderAccess<'_, S>,
        parent: &'a WorthQueryManagedGraphWorkSession,
        request: &'a WorthQueryRequestScope,
        association: &WorthQueryInstalledManagedIndexParent,
        selector: &WorthQueryObservedScopeSelector,
        probe: &AspectValue,
        parameters: &Parameters,
        governance: &WorthQueryApplicationQueryGovernance,
        allocation: &Arc<SourceFactsAllocation>,
        native: &mut ApplicationQueryNativeAccess<'_>,
    ) -> Result<ManagedSelectionLease<'a, 'request, 'authority>, Denial> {
        if !self.installed.admits_association(association) {
            return Err(Denial::ForeignAssociation);
        }
        let adapter: Arc<dyn InstalledIndexProvider<S, I>> = self;
        super::super::maintained_read::select::<S, I>(
            execution,
            application,
            access,
            parent,
            request,
            association,
            selector,
            &adapter,
            probe,
            parameters,
            governance,
            allocation,
            native,
        )
    }
}
