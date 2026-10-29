//! Explicit cold reconstruction; no ordinary query silently invokes this lane.
use super::{purpose, ProviderExecutionDenial as Denial};
use crate::domain_computation::{
    primary_graph::{
        application_query::{
            managed_index::{
                retention::{ManagedIndexState, ReadyIndex},
                source_acquisition,
            },
            WorthQueryObservedScopeSelector,
        },
        source_facts::SourceFactsAllocation,
        WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
    },
    provider_session::WorthQueryManagedGraphWorkSession,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn rebuild<
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
>(
    execution: &ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    state: &Arc<ManagedIndexState<S, I>>,
    parent: &WorthQueryManagedGraphWorkSession,
    selected: &WorthQuerySelectedProductOperation<'_, S>,
    selector: &WorthQueryObservedScopeSelector,
    external: &WorthQueryAuthenticatedExternalPrincipal<S>,
    request: &WorthQueryRequestScope,
    allocation: &Arc<SourceFactsAllocation>,
) -> Result<(), Denial> {
    let epoch = state
        .begin_build(execution, selected)
        .map_err(Denial::Lifecycle)?;
    let result = (|| {
        let source_allocation = SourceFactsAllocation::open(execution)?;
        let source = source_acquisition::acquire(
            execution,
            application,
            state.adapter.as_ref(),
            parent,
            &state.association,
            selected,
            selector,
            external,
            request,
            source_allocation,
        )
        .map_err(Denial::Acquisition)?;
        let session = purpose::start(
            execution,
            application,
            parent,
            request,
            &state.association,
            Arc::clone(&source.terminal),
            allocation,
        )?;
        let build_identity = session.identity();
        let (nodes, callback) = session
            .call(|actual| {
                state.adapter.build(
                    actual,
                    build_identity,
                    source.view(application.installed_schema().value_bindings()),
                )
            })
            .map_err(Denial::Session)?
            .map_err(Denial::Work)?;
        let proof = session.complete(callback).map_err(Denial::Session)?;
        let root = ReadyIndex::retain(execution, source, nodes).map_err(Denial::Lifecycle)?;
        state
            .finish_build(epoch, root, &proof, parent, request)
            .map_err(Denial::Lifecycle)
    })();
    if result.is_err() {
        state.failed(epoch);
    }
    result
}
