//! A ready registration lends genuine source slots, never a provider root handle.
use super::{purpose, ProviderExecutionDenial as Denial};
use crate::domain_computation::{
    primary_graph::{
        application_query::managed_index::{
            retention::{ManagedIndexState, ReadyIndexMaterial},
            selection::ManagedSelectionLease,
        },
        source_facts::SourceFactsAllocation,
        WorthQueryPrimaryGraphApplicationRuntime,
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

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn select<
    'a,
    'request,
    'authority,
    S,
    I,
>(
    execution: &ExecutionRequest<'request, 'authority>,
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
    parent: &'a WorthQueryManagedGraphWorkSession,
    request: &'a WorthQueryRequestScope,
    association: &WorthQueryInstalledManagedIndexParent,
    selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'_, S>,
    state: Arc<ManagedIndexState<S, I>>,
    probe: &AspectValue,
    allocation: &Arc<SourceFactsAllocation>,
) -> Result<ManagedSelectionLease<'a, 'request, 'authority>, Denial>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
{
    if !state.association.same_association(association)
        || !state
            .adapter
            .installed()
            .is_installed_in(application.installed_schema())
    {
        return Err(Denial::ForeignAssociation);
    }
    let (epoch, root) = state.ready().map_err(Denial::Lifecycle)?;
    let graph = application
        .runtime
        .primary_graph()
        .ok_or(Denial::ForeignAssociation)?;
    let material = graph.with_runtime(execution, |runtime| {
        runtime.read_truth().public_managed_index_cache_payload(
            execution, selected.application_basis().snapshot_handle(), &root.cache,
        )
    }).map_err(|cause| match cause {
        worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial::RootUnavailable =>
            Denial::Lifecycle(super::super::retention::WorthQueryManagedIndexLifecycleDenial::NotReady),
        cause => Denial::NativeSource(cause),
    })?.downcast::<ReadyIndexMaterial>().map_err(|_| Denial::ForeignAssociation)?;
    if !Arc::ptr_eq(&material.source, &root.source.material) {
        return Err(Denial::ForeignAssociation);
    }
    let session = purpose::start(
        execution,
        application,
        parent,
        request,
        association,
        Arc::clone(&root.source.terminal),
        allocation,
    )?;
    let frame =
        crate::domain_computation::primary_graph::access_strategy::coverage::CoverageFrame::public(
            root.source
                .view(application.installed_schema().value_bindings()),
            &material.nodes,
            root.source.material.as_ref(),
            probe,
            parent,
            association.access_contract().meaning().match_posture(),
        );
    let (selection, callback) = session
        .call(|actual| state.adapter.candidates(actual, frame))
        .map_err(Denial::Session)?
        .map_err(Denial::Work)?;
    let slots = super::super::invocation::controlled(execution, |control| selection.receive(
        frame, state.adapter.coverage_authority(),
        association.definition().parts().membership != worth_query_declaration::facade::application_query::ApplicationManagedIndexMembership::Candidates,
        control,
    )).map_err(Denial::Work)?;
    let proof = session.complete(callback).map_err(Denial::Session)?;
    ManagedSelectionLease::issue(proof, parent, request, state, root, epoch, slots)
        .map_err(Denial::Selection)
}
