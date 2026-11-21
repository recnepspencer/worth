//! Re-execute recorded query DATA under the installed typed source and fresh authority.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandInterest;
use crate::domain_computation::primary_graph::query_obligations::QueryOutputDemandObligation;
use crate::domain_computation::primary_graph::ReadObservationRuntimeExt;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationReadObservation,
    WorthQueryObservedScopeSelector, WorthQueryProducerOutputFamily,
    WorthQueryProductQueryControls,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_execution_runtime::facade::ports::ProductBranchInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductObservationLeaseInternalPort as _;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

pub fn readmit<S, B>(
    installed: &TypedInstalledProducer<S, B>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    external: &WorthQueryAuthenticatedExternalPrincipal<S>,
    request: &WorthQueryRequestScope,
    row: &QueryOutputDemandObligation,
) -> Result<WorthQueryOutputDemandInterest, WorthQueryOutputDemandDenial>
where
    S: ApplicationSchema + 'static,
    B: WorthQueryApplicationProducerBinding<S>,
    SourceValue<S, B>: crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            S,
            SourceQuery<S, B>,
        > + 'static,
    SourceQuery<S, B>: 'static,
{
    if row.producer != B::IDENTITY {
        return Err(unavailable());
    }
    let (source, retained) = readmit_source::<S, B::OutputFamily>(
        &installed.source_query,
        execution,
        runtime,
        external,
        request,
        row,
    )?;
    let demand = runtime.admit_dependent_output_demand::<B::OutputFamily>(
        execution,
        source,
        runtime.output_demand_resource_profile().limits(),
        retained,
    )?;
    demand.adopt_recorded_membership(runtime, row)?;
    demand.into_durable_interest().ok_or_else(unavailable)
}

type SourceValueFor<S, F> = <<<F as WorthQueryProducerOutputFamily<S>>::Source as ApplicationQueryBinding<S>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value;
type SourceQueryFor<S, F> =
    <<F as WorthQueryProducerOutputFamily<S>>::Source as ApplicationQueryBinding<S>>::Query;

pub fn readmit_source<S, F>(
    binding: &WorthQueryInstalledApplicationQueryBinding<S, F::Source>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    external: &WorthQueryAuthenticatedExternalPrincipal<S>,
    request: &WorthQueryRequestScope,
    row: &QueryOutputDemandObligation,
) -> Result<
    (
        crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            SourceQueryFor<S, F>,
            SourceValueFor<S, F>,
        >,
        std::sync::Arc<WorthQueryApplicationReadObservation>,
    ),
    WorthQueryOutputDemandDenial,
>
where
    S: ApplicationSchema + 'static,
    F: WorthQueryProducerOutputFamily<S>,
    SourceValueFor<S, F>: crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            S,
            SourceQueryFor<S, F>,
        > + 'static,
    SourceQueryFor<S, F>: 'static,
{
    let query = binding.query();
    if query.identity().as_bytes() != &row.query || row.family != F::IDENTITY {
        return Err(unavailable());
    }
    let basis = runtime.readmit_output_observation(execution, row.basis, &row.branch)?;
    let branch = WorthQueryProductBranch::from_occurrence(basis.lifecycle_incarnation());
    let observation = WorthQueryApplicationReadObservation::from_product(
        runtime,
        crate::basis::WorthQueryProductObservationLease::from_observation(basis),
    );
    let retained = runtime
        .select_application_read_observation_with_request(execution, &observation)
        .map_err(|cause| WorthQueryOutputDemandDenial::product_selection(cause, F::IDENTITY))?;
    let current = runtime
        .on_branch(branch)
        .select_with_request(execution)
        .map_err(|cause| WorthQueryOutputDemandDenial::product_selection(cause, F::IDENTITY))?;
    let principal = current
        .resolve_authenticated_principal_admitted(
            execution,
            binding.principal_binding(),
            external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|cause| principal_rejected(F::IDENTITY, cause).into_output_denial())?;
    let (aspect, fields, value, subject): (
        String,
        Vec<String>,
        worth_foundational::facade::AspectValue,
        worth_relational::facade::identity::EntityId,
    ) = rmp_serde::from_slice(&row.scope).map_err(|_| unavailable())?;
    let selector = WorthQueryObservedScopeSelector::readmit_recorded(
        &aspect, &fields, value, subject, execution,
    )
    .map_err(|_| unavailable())?;
    let scope = current
        .resolve_retained_query_scope(
            execution,
            <F::Source as ApplicationQueryBinding<S>>::scope_field(),
            &selector,
            request,
        )
        .map_err(|cause| scope_rejected(F::IDENTITY, cause).into_output_denial())?;
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let values = rmp_serde::from_slice(&row.parameters).map_err(|_| unavailable())?;
    let parameters = worth_query_admission::facade::application_query::readmit_recorded_application_query_parameters(query,values).map_err(|_| unavailable())?;
    let graph = runtime.runtime.primary_graph().ok_or_else(unavailable)?;
    crate::domain_computation::primary_graph::application_query::basis::recover_recorded_query_indexes(
        execution, graph, query, scope.identity_index_id(), retained.product().relational_basis(),
    ).map_err(|cause| query_admission_denied(F::IDENTITY, cause).into_output_denial())?;
    let maximum = runtime
        .resolve_application_query_limits(binding.limits())
        .maximum_results();
    let (_, controls) = current
        .retained_query_controls(
            execution,
            retained,
            query.name(),
            WorthQueryProductQueryControls::new(maximum, request),
        )
        .map_err(|_| unavailable())?;
    let plan = runtime
        .finish_application_query_readmission(execution, query, &access, parameters, controls, None)
        .map_err(|cause| query_admission_denied(F::IDENTITY, cause).into_output_denial())?;
    let source = runtime
        .execute_application_query_one_shot_with_request(execution, plan)
        .map_err(|cause| query_execution_denied(F::IDENTITY, cause).into_output_denial())?
        .into_admitted_disclosed()
        .into_output_demand_source();
    Ok((source, observation))
}
fn unavailable() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
        "recorded required query input",
    )
}
