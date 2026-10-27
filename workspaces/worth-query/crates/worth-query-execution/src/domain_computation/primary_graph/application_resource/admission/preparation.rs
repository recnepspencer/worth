//! Join genuine issued principal/scope guards to the installed Resource leaf.
use super::*;
use crate::domain_computation::{
    authorization::application_disclosure::{
        admit_application_resource_governance, compile_resource_disclosure_contract,
        WorthQueryApplicationGovernanceBinding, WorthQueryPendingApplicationQueryGovernance,
    },
    primary_graph::{
        application_query::CapturedApplicationQueryParameters,
        capture_principal_currentness_admitted,
        entity_resolution::{WorthQueryIssuedSelectedScope, WorthQueryObservedScopeSelector},
        WorthQueryApplicationQueryAccessContext, WorthQueryIssuedSelectedPrincipal,
        WorthQuerySelectedProductOperation,
    },
    provider_session::{
        WorthQueryGraphWorkAccessContextAffinity, WorthQueryPreparedProductReadSessionIdentity,
    },
};
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
    application_resource::ApplicationResourceBinding,
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationResource, WorthQueryResourceAccessPurpose,
};

type AccessScope<Schema, Binding> = <<<Binding as ApplicationResourceBinding<Schema>>::Access as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope;

pub(in crate::domain_computation::primary_graph::application_resource) fn prepare<
    'a,
    Schema,
    Binding,
>(
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    resource: &WorthQueryInstalledApplicationResource<Schema, Binding>,
    prepared: &worth_query_admission::facade::graph_obligation::WorthQueryPreparedResourceReadGraphWork,
    purpose: WorthQueryResourceAccessPurpose,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    principal: &'a WorthQueryIssuedSelectedPrincipal<
        '_,
        '_,
        Schema,
        <Binding::Access as ApplicationQueryBinding<Schema>>::Principal,
        <Binding::Access as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
    >,
    scope: &'a WorthQueryIssuedSelectedScope<'_, '_, Schema, AccessScope<Schema, Binding>>,
    request: &'a WorthQueryRequestScope,
    original: Option<&WorthQueryObservedScopeSelector>,
    parameters: &CapturedApplicationQueryParameters,
    mut pending: Option<WorthQueryPendingApplicationQueryGovernance>,
) -> Result<ResourceActionRead<'a, Schema, AccessScope<Schema, Binding>>, ResourceAdmissionDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    require_request(execution, request)?;
    if !std::ptr::eq(runtime, selected.application())
        || !std::ptr::eq(principal.selected(), selected)
        || !std::ptr::eq(scope.selected(), selected)
        || !principal.request().same_request(request)
        || !scope.request().same_request(request)
        || principal.issued_root().admission_identity() != scope.issued_root().admission_identity()
        || !std::ptr::eq(principal.issued_snapshot(), scope.issued_snapshot())
        || parameters.query() != resource.access_binding().query().identity()
    {
        return Err(ResourceAdmissionDenial::ForeignIssuedAccess);
    }
    // The complete original generation is checked before protected row lookup,
    // provider contact or any cell-clock effect.
    if original.is_some_and(|original| original.subject() != scope.scope().entity_id()) {
        return Err(ResourceAdmissionDenial::OriginalScopeGenerationChanged);
    }
    let reserved = WorthQueryPreparedProductReadSessionIdentity::reserve()
        .map_err(ResourceAdmissionDenial::Session)?;
    let principal_currentness =
        capture_principal_currentness_admitted(runtime, principal.principal(), reserved.identity())
            .map_err(|_| ResourceAdmissionDenial::MissingPrincipalBinding)?;
    let (plan, admission) = plan::admit(runtime, execution, resource, purpose, prepared)?;
    let graph = runtime
        .runtime
        .primary_graph()
        .ok_or(ResourceAdmissionDenial::MissingGraph)?;
    let basis = selected.application_basis().identity().clone();
    let product = selected.product().read_lease();
    let access = WorthQueryApplicationQueryAccessContext::new(principal.principal(), scope.scope());
    let mut session = WorthQueryManagedGraphWorkSession::start_resource(
        reserved,
        resource,
        purpose,
        plan,
        runtime.runtime.authority_identity(),
        principal.principal().principal_entity_id(),
        WorthQueryGraphWorkAccessContextAffinity::entity(scope.scope().entity_id()),
        &basis,
        product.retained_clone(),
        &product,
        runtime.graph_work_provider_identity(),
        graph.product_read_session_port(),
    )
    .map_err(ResourceAdmissionDenial::Session)?;
    let (observed, proof) = session
        .execute_resource_read(execution, &basis, |relational, _| {
            runtime.observe_resource_authorization(
                execution,
                session.identity(),
                relational,
                selected.application_basis().snapshot_handle().clone(),
                resource,
                purpose.action(),
                &access,
            )
        })
        .map_err(ResourceAdmissionDenial::Read)?;
    let decisions = observed.map_err(ResourceAdmissionDenial::Authorization)?;
    let authorization =
        WorthQueryRetainedAuthorizationDecisionFacts::abilities(principal_currentness, decisions);
    let contract = resource.retain_access_contract(purpose);
    let disclosure = compile_resource_disclosure_contract(resource, purpose)
        .map_err(ResourceAdmissionDenial::Contract)?;
    if let Some(pending) = pending.as_mut() {
        runtime
            .refresh_capability_authorization_for_graph_work(
                execution,
                pending.authorization_mut(),
                &session,
            )
            .map_err(ResourceAdmissionDenial::DisclosureAuthorization)?;
    }
    let binding = WorthQueryApplicationGovernanceBinding::from_resource_session(
        &session,
        &contract,
        *parameters.parameters().identity(),
        principal.principal().principal_entity_id(),
        scope.scope().entity_id(),
    )
    .ok_or(ResourceAdmissionDenial::ForeignIssuedAccess)?;
    let governance = admit_application_resource_governance(disclosure, pending, binding)
        .map_err(ResourceAdmissionDenial::Governance)?;
    let retained = authorization.exact_fact_count()
        + governance
            .authorization()
            .map_or(0, |capability| capability.exact_fact_count());
    session.set_retained_decision_facts(retained);
    let mut admitted = ResourceActionRead {
        runtime,
        scope: scope.scope(),
        request,
        principal: principal.principal().principal_entity_id(),
        parameters: *parameters.parameters().identity(),
        basis,
        contract,
        session,
        proof: Some(proof),
        authorization,
        governance,
        _admission: admission,
    };
    admitted.check(execution)?;
    Ok(admitted)
}
