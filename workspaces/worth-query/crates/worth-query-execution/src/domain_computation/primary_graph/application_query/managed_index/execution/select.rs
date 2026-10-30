//! A ready registration lends genuine source slots, never a provider root handle.
use super::{purpose, ProviderExecutionDenial as Denial};
use crate::domain_computation::{
    primary_graph::{
        application_query::managed_index::{
            retention::ManagedIndexState, selection::ManagedSelectionLease,
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
    let session = purpose::start(
        execution,
        application,
        parent,
        request,
        association,
        Arc::clone(&root.source.terminal),
        allocation,
    )?;
    let (slots, callback) = session
        .call(|actual| {
            state.adapter.candidates(
                actual,
                root.source
                    .view(application.installed_schema().value_bindings()),
                root.nodes.as_ref(),
                probe,
            )
        })
        .map_err(Denial::Session)?
        .map_err(Denial::Work)?;
    let proof = session.complete(callback).map_err(Denial::Session)?;
    ManagedSelectionLease::issue(proof, parent, request, state, root, epoch, slots)
        .map_err(Denial::Selection)
}
