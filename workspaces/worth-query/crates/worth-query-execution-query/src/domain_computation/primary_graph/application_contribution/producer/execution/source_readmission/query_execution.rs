//! Phase-isolated ordinary and selected source-query readmission execution.
use super::*;

#[inline(never)]
#[allow(clippy::too_many_arguments)]
pub(super) fn readmit_ordinary_source<'runtime, Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    query: &worth_query_installation::facade::WorthQueryInstalledBoundQuery<
        Schema,
        BoundQuery<Schema, Binding>,
    >,
    access: &WorthQueryApplicationQueryAccessContext<
        '_,
        Schema,
        BoundPrincipal<Schema, Binding>,
        BoundPrincipalIdentity<Schema, Binding>,
        BoundScope<Schema, Binding>,
    >,
    retained: &WorthQueryObservedSource<SourceQuery<Schema, Binding>>,
    selected: WorthQuerySelectedProductOperation<'runtime, Schema>,
    maximum_results: std::num::NonZeroUsize,
    request: &WorthQueryRequestScope,
) -> Result<
    crate::domain_computation::primary_graph::WorthQueryApplicationOneShotResult<
        SourceQuery<Schema, Binding>,
        SourceValue<Schema, Binding>,
    >,
    ProducerExecutionStop,
>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Binding>>,
{
    // The ordinary current executor refreshes governed authorization. It is
    // a different supported admission purpose from selected read-only disclosure.
    let selection_origin = selected.selection_origin();
    let (_, product, application_basis) = selected.into_parts();
    let controls = WorthQueryApplicationQueryControls::product_one_shot(
        product,
        application_basis,
        maximum_results,
        request,
    )
    .with_selection_origin(selection_origin);
    let plan = runtime
        .readmit_application_query_from_observed(execution, query, access, retained, controls)
        .map_err(|error| query_admission_denied(Binding::IDENTITY, error))?;
    runtime
        .execute_application_query_one_shot_with_request(execution, plan)
        .map_err(|error| query_execution_denied(Binding::IDENTITY, error))
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
pub(super) fn readmit_selected_source<'runtime, Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    query: &worth_query_installation::facade::WorthQueryInstalledBoundQuery<
        Schema,
        BoundQuery<Schema, Binding>,
    >,
    access: &WorthQueryApplicationQueryAccessContext<
        '_,
        Schema,
        BoundPrincipal<Schema, Binding>,
        BoundPrincipalIdentity<Schema, Binding>,
        BoundScope<Schema, Binding>,
    >,
    retained: &WorthQueryObservedSource<SourceQuery<Schema, Binding>>,
    selected: WorthQuerySelectedProductOperation<'runtime, Schema>,
    maximum_results: std::num::NonZeroUsize,
    request: &WorthQueryRequestScope,
) -> Result<
    crate::domain_computation::primary_graph::WorthQueryApplicationOneShotResult<
        SourceQuery<Schema, Binding>,
        SourceValue<Schema, Binding>,
    >,
    ProducerExecutionStop,
>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Binding>>,
{
    let shared = selected
        .prepare_shared_query_basis()
        .map_err(|(_, stop)| ready::ready_resource_denial(Binding::IDENTITY, stop))?;
    let (product, application_basis) = runtime
        .retain_selected_query_basis_admitted(&shared)
        .map_err(|stop| match stop {
            SelectedQueryBasisRetentionStop::Admission(stop) => {
                ready::ready_resource_denial(Binding::IDENTITY, stop)
            }
            SelectedQueryBasisRetentionStop::Basis => {
                ProducerExecutionStop::ExecutionStopped(denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    Binding::IDENTITY,
                ))
            }
        })?;
    let controls = WorthQueryApplicationQueryControls::selected_read_one_shot(
        product,
        application_basis,
        maximum_results,
        request,
    )
    .map_err(|stop| ready::ready_resource_denial(Binding::IDENTITY, stop))?
    .with_selection_origin(shared.selected().selection_origin());
    let plan = runtime
        .readmit_application_query_from_observed(execution, query, access, retained, controls)
        .map_err(|error| query_admission_denied(Binding::IDENTITY, error))?;
    let reader =
        crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess::ordinary(
            runtime,
            &shared,
            external_principal,
            request,
        );
    reader
        .read_admitted_result::<SourceBinding<Schema, Binding>>(
            execution,
            plan,
            &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire(),
        )
        .map_err(|stop| super::super::denial::selected_source_read_denied(Binding::IDENTITY, stop))
}
