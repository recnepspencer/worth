//! Public reconstruction refills the same Native cache consumed by invariants.
use super::ProviderExecutionDenial as Denial;
use crate::domain_computation::{
    primary_graph::{
        application_query::managed_index::{
            retention::ManagedIndexState,
            source::{occurrences::PersistentSourceRows, service::retain_root},
            source_image::SourceImage,
        }, WorthQuerySelectedProductOperation,
    },
    provider_session::{WorthQueryManagedGraphWorkSession, WorthQueryManagedIndexReadSession},
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_relational::facade::runtime::ManagedIndexSourceRebuild;

#[allow(clippy::too_many_arguments)]
pub(super) fn rebuild<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>(
    execution: &ExecutionRequest<'_, '_>,
    application: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>,
    state: &Arc<ManagedIndexState<S, I>>,
    parent: &WorthQueryManagedGraphWorkSession,
    selected: &WorthQuerySelectedProductOperation<'_, S>,
    request: &WorthQueryRequestScope,
    epoch: u64,
    source: &SourceImage<S, I>,
    loan: &ManagedIndexSourceRebuild,
    session: WorthQueryManagedIndexReadSession<'_, '_, '_>,
) -> Result<(), Denial> {
    let installed = state.adapter.installed();
    let read = installed.source_read();
    let maximum = read.limits().maximum_results().get().min(
        std::num::NonZeroUsize::new(read.query().read_graph().maximum_result_count())
            .map_or(usize::MAX, std::num::NonZeroUsize::get),
    );
    let owner = session.identity();
    let rows = PersistentSourceRows::acquire(
        execution,
        &loan.rows(*execution),
        owner,
        read.query().read_graph(),
        maximum,
    )
    .map_err(|cause| Denial::NativeSource(cause.into_native()))?;
    let bindings = application.installed_schema().value_bindings();
    let (nodes, callback) = session
        .call(|actual| {
            state
                .adapter
                .build(actual, owner, rows.view(bindings, None))
        })
        .map_err(Denial::Session)?
        .map_err(Denial::Work)?;
    let proof = session.complete(callback).map_err(Denial::Session)?;
    let root = retain_root(
        execution,
        Arc::clone(&state.adapter),
        bindings.clone(),
        rows,
        nodes,
    )
    .map_err(Denial::NativeSource)?;
    state
        .admit_build(epoch, &proof, parent, request, &source.terminal)
        .map_err(Denial::Lifecycle)?;
    let graph = application
        .primary_graph()
        .ok_or(Denial::ForeignAssociation)?;
    let (observation, retirement) = graph
        .with_runtime(execution, |runtime| {
            runtime.read_truth().finish_managed_index_rebuild(
                execution,
                selected.application_basis().snapshot_handle(),
                loan,
                &root,
            )
        })
        .map_err(Denial::NativeSource)?;
    // Installation completed. No late request poll or handle lifecycle change
    // may relabel its effect. Retired backing is dropped after runtime guards.
    state.finish_maintained_build(epoch, observation);
    drop(retirement);
    Ok(())
}
