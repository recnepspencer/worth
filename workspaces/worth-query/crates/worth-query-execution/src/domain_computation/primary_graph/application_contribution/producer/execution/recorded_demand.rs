//! Re-execute recorded query DATA under the installed typed source and fresh authority.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandInterest;
use crate::domain_computation::primary_graph::query_obligations::QueryOutputDemandObligation;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationReadObservation,
    WorthQueryObservedScopeSelector, WorthQueryProductQueryControls,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
pub(super) fn readmit<S, B>(
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
    let binding = &installed.source_query;
    let query = binding.query();
    if query.identity().as_bytes() != &row.query
        || row.producer != B::IDENTITY
        || row.family
            != <B::OutputFamily as super::super::WorthQueryProducerOutputFamily<S>>::IDENTITY
    {
        return Err(unavailable());
    }
    let basis = runtime.readmit_output_observation(execution, row.basis, &row.branch)?;
    let branch = WorthQueryProductBranch::from_occurrence(basis.lifecycle_incarnation());
    let observation = WorthQueryApplicationReadObservation::from_product(
        runtime,
        crate::basis::WorthQueryProductObservationLease::new(basis),
    );
    let retained = runtime
        .select_application_read_observation_with_request(execution, &observation)
        .map_err(|cause| WorthQueryOutputDemandDenial::product_selection(cause, B::IDENTITY))?;
    let current = runtime
        .on_branch(branch)
        .select_with_request(execution)
        .map_err(|cause| WorthQueryOutputDemandDenial::product_selection(cause, B::IDENTITY))?;
    let principal = current
        .resolve_authenticated_principal_admitted(
            execution,
            binding.principal_binding(),
            external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|cause| principal_rejected(B::IDENTITY, cause).into_output_denial())?;
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
            SourceBinding::<S, B>::scope_field(),
            &selector,
            request,
        )
        .map_err(|cause| scope_rejected(B::IDENTITY, cause).into_output_denial())?;
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let values = rmp_serde::from_slice(&row.parameters).map_err(|_| unavailable())?;
    let parameters = worth_query_admission::facade::application_query::readmit_recorded_application_query_parameters(query,values).map_err(|_| unavailable())?;
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
        .map_err(|cause| query_admission_denied(B::IDENTITY, cause).into_output_denial())?;
    let source = runtime
        .execute_application_query_one_shot_with_request(execution, plan)
        .map_err(|cause| query_execution_denied(B::IDENTITY, cause).into_output_denial())?
        .into_admitted_disclosed()
        .into_output_demand_source();
    let demand = runtime.admit_required_output_demand::<B::OutputFamily>(
        execution,
        source,
        runtime.output_demand_resource_profile().limits(),
    )?;
    let interest = demand.into_durable_interest().ok_or_else(unavailable)?;
    let history = runtime
        .installed_program_support()
        .ok_or_else(unavailable)?
        .obligation_history();
    interest
        .durable_readmission()
        .ok_or_else(unavailable)?
        .adopt_durable(history, row.acquire_position);
    Ok(interest)
}
fn unavailable() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
        "recorded required query input",
    )
}
