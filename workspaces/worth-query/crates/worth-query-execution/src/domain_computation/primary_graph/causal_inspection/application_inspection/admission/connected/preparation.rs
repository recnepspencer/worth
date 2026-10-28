//! Both actual subjects receive fresh READ and independent complete disclosure.
use super::super::super::source_original::SourceOriginal;
use super::super::{policy, preparation, InspectionSubject};
use super::*;
use crate::domain_computation::{
    primary_graph::{
        causal_inspection::WorthQueryApplicationQueryObservation,
        WorthQueryIssuedSelectedPrincipal, WorthQueryIssuedSelectedScope,
        WorthQuerySelectedProductOperation,
    },
    provider_session::WorthQueryPreparedProductReadSessionIdentity,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryScopeBinding,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationQueryBinding,
};
type Scope<S, B> =
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;
#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn prepare<
    'a,
    'storage,
    Schema,
    Binding,
    SourceBinding,
>(
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    source_binding: &WorthQueryInstalledApplicationQueryBinding<Schema, SourceBinding>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    request: &'a WorthQueryRequestScope,
    principal: &'a WorthQueryIssuedSelectedPrincipal<
        '_,
        '_,
        Schema,
        Binding::Principal,
        Binding::PrincipalIdentity,
    >,
    scope: &'a WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope<Schema, Binding>>,
    source_principal: &WorthQueryIssuedSelectedPrincipal<
        '_,
        '_,
        Schema,
        SourceBinding::Principal,
        SourceBinding::PrincipalIdentity,
    >,
    source_scope: &WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope<Schema, SourceBinding>>,
    original: &WorthQueryApplicationQueryObservation,
    source: &SourceOriginal,
    query_prepared: PreparedConnectionQuery,
    policies: ConnectionPolicies,
    memory: &'storage mut Option<ExecutionMemoryReservation>,
) -> Result<InspectionRead<'a, 'storage, Schema, Scope<Schema, Binding>>, InspectionAdmissionDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
    SourceBinding: ApplicationQueryBinding<Schema>,
{
    let principal_id = principal.principal().principal_entity_id();
    if source_principal.principal().principal_entity_id() != principal_id
        || source_scope.scope().entity_id() != scope.scope().entity_id()
    {
        return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
    }
    let PreparedConnectionQuery {
        contract: query_contract,
        policy: query_policy,
        reserved,
    } = query_prepared;
    let (source_contract, source_policy) = policy::prepare(
        runtime,
        execution,
        source_binding,
        selected,
        source_principal,
        source_scope,
        request,
        InspectionSubject::Source(source),
        policies.source_metadata,
        Some(policies.source_diagnostic),
        &reserved,
        memory,
    )?;
    let source_target = WorthQueryApplicationInspectionTarget::capture_source(
        execution,
        source,
        source_contract,
        memory,
    )
    .map_err(InspectionAdmissionDenial::Target)?;
    let target = WorthQueryApplicationInspectionTarget::capture_connection(
        execution,
        original,
        query_contract,
        source_target,
        memory,
    )
    .map_err(InspectionAdmissionDenial::Target)?;
    preparation::start(
        runtime,
        execution,
        selected,
        principal_id,
        scope.scope(),
        request,
        target,
        query_policy,
        reserved,
        Some(source_policy),
        memory,
    )
}

/// Root READ/metadata/diagnostics precede even original source-disclosure availability.
#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn prepare_query<
    Schema,
    Binding,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    query: &WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    request: &WorthQueryRequestScope,
    principal: &WorthQueryIssuedSelectedPrincipal<
        '_,
        '_,
        Schema,
        Binding::Principal,
        Binding::PrincipalIdentity,
    >,
    scope: &WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope<Schema, Binding>>,
    original: &WorthQueryApplicationQueryObservation,
    policies: &mut ConnectionPolicies,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<PreparedConnectionQuery, InspectionAdmissionDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    let reserved = WorthQueryPreparedProductReadSessionIdentity::reserve()
        .map_err(InspectionAdmissionDenial::Session)?;
    let diagnostic = policies
        .query_diagnostic
        .take()
        .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
    let (contract, policy) = policy::prepare(
        runtime,
        execution,
        query,
        selected,
        principal,
        scope,
        request,
        InspectionSubject::Query(original),
        policies.query_metadata.take(),
        Some(diagnostic),
        &reserved,
        memory,
    )?;
    Ok(PreparedConnectionQuery {
        contract,
        policy,
        reserved,
    })
}
