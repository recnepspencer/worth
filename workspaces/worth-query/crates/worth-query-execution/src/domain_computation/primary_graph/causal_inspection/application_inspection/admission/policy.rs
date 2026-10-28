//! Fixed metadata policy is selected before the original target is sized or copied.
use super::*;
use crate::domain_computation::{
    authorization::application_disclosure::{
        compile_diagnostic_disclosure_contract, compile_inspection_disclosure_contract,
        prepare_application_diagnostic_governance, prepare_application_inspection_governance,
        WorthQueryPendingApplicationQueryGovernance,
    },
    primary_graph::{
        application_query::{SelectedIssuedAccessRoot, WorthQueryApplicationQueryAccessContext},
        WorthQueryIssuedSelectedPrincipal, WorthQueryIssuedSelectedScope,
        WorthQueryPrincipalResolutionMode, WorthQuerySelectedProductOperation,
    },
    provider_session::WorthQueryPreparedProductReadSessionIdentity,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryInspectionMetadataFacet as Facet,
    ApplicationQueryScopeBinding,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationQueryBinding,
};

pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) struct PreparedInspectionPolicy {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) metadata: crate::domain_computation::authorization::application_disclosure::WorthQueryPreparedApplicationGovernance,
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) diagnostic: Option<crate::domain_computation::authorization::application_disclosure::WorthQueryPreparedApplicationGovernance>,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn prepare<
    Schema,
    Binding,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    principal: &WorthQueryIssuedSelectedPrincipal<
        '_,
        '_,
        Schema,
        Binding::Principal,
        Binding::PrincipalIdentity,
    >,
    scope: &WorthQueryIssuedSelectedScope<'_, '_, Schema, <<Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>,
    request: &WorthQueryRequestScope,
    original: InspectionSubject<'_>,
    pending: Option<WorthQueryPendingApplicationQueryGovernance>,
    diagnostic: Option<WorthQueryPendingApplicationQueryGovernance>,
    reserved: &WorthQueryPreparedProductReadSessionIdentity,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<
    (
        worth_query_installation::facade::WorthQueryInstalledApplicationInspectionContract,
        PreparedInspectionPolicy,
    ),
    InspectionAdmissionDenial,
>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    require_request(execution, request)?;
    let contract = original.validate(runtime, installed, selected, scope.scope().entity_id())?;
    let access = WorthQueryApplicationQueryAccessContext::new(principal.principal(), scope.scope());
    let root = SelectedIssuedAccessRoot::from_issued(
        selected,
        &access,
        principal,
        scope,
        request,
        "application inspection",
    )
    .map_err(InspectionAdmissionDenial::Authorization)?;
    let (authorization, _) = runtime
        .observe_issued_query_read_authorization(execution, installed.query(), &root, reserved)
        .map_err(InspectionAdmissionDenial::Authorization)?;
    let disclosure = compile_inspection_disclosure_contract(&contract, execution, memory)
        .map_err(InspectionAdmissionDenial::Storage)?;
    let mut metadata = prepare_application_inspection_governance(disclosure, pending)
        .map_err(InspectionAdmissionDenial::Governance)?;
    let product = selected.product().read_lease();
    if let Some(capability) = metadata.authorization_mut() {
        runtime
            .refresh_capability_authorization_for_prepared_read(
                execution, capability, reserved, &product,
            )
            .map_err(InspectionAdmissionDenial::Capability)?;
    }
    let graph = runtime
        .runtime
        .primary_graph()
        .ok_or(InspectionAdmissionDenial::MissingGraph)?;
    let security = runtime
        .admit_product_security_basis(execution, &product)
        .map_err(InspectionAdmissionDenial::Security)?;
    let resolution = graph.retain_entity_resolution_context();
    graph
        .integration_handle()
        .with_runtime_mut(execution, |relational| {
            let snapshot = security.snapshot_handle();
            authorization
                .validate_currentness_in(
                    execution,
                    relational,
                    snapshot,
                    runtime.authorization.bridge(),
                )
                .map_err(|cause| InspectionAdmissionDenial::CurrentAuthorization(cause))?;
            if let Some(capability) = metadata.authorization_mut() {
                capability
                    .validate_currentness_in(
                        execution,
                        relational,
                        snapshot,
                        runtime.authorization.bridge(),
                    )
                    .map_err(InspectionAdmissionDenial::CurrentAuthorization)?;
            }
            resolution
                .at_snapshot(
                    relational,
                    snapshot,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .and_then(|truth| truth.validate_entity_freshness(execution, scope.scope()))
                .map_err(InspectionAdmissionDenial::Scope)
        })?;
    // Observed requires its five fixed metadata facets. A paired comparison
    // additionally requires DiagnosticFields before retaining either original;
    // the separate diagnostic capability still owns payload comparison.
    for (facet, slot) in Facet::ALL.iter().zip(contract.metadata_slots()) {
        if matches!(
            facet,
            Facet::ObservationIdentity
                | Facet::EvidenceLinks
                | Facet::EvidenceCounts
                | Facet::EvidenceAvailability
                | Facet::Explanation
        ) || diagnostic.is_some()
        {
            if !metadata.is_disclosed(slot) {
                return Err(InspectionAdmissionDenial::FacetDenied);
            }
        }
    }
    require_request(execution, request)?;
    let diagnostic = match diagnostic {
        Some(pending) => {
            let contract = compile_diagnostic_disclosure_contract(
                installed.query(),
                &graph.layout,
                execution,
                memory,
            )
            .map_err(InspectionAdmissionDenial::DiagnosticContract)?;
            let mut diagnostic = prepare_application_diagnostic_governance(
                installed.query(),
                contract,
                pending,
                execution,
                memory,
            )
            .map_err(InspectionAdmissionDenial::DiagnosticPreparation)?;
            if let Some(capability) = diagnostic.authorization_mut() {
                runtime
                    .refresh_capability_authorization_for_prepared_read(
                        execution, capability, reserved, &product,
                    )
                    .map_err(InspectionAdmissionDenial::Capability)?;
                graph
                    .integration_handle()
                    .with_runtime_mut(execution, |relational| {
                        capability
                            .validate_currentness_in(
                                execution,
                                relational,
                                security.snapshot_handle(),
                                runtime.authorization.bridge(),
                            )
                            .map_err(InspectionAdmissionDenial::CurrentAuthorization)
                    })?;
            }
            Some(diagnostic)
        }
        None => None,
    };
    Ok((
        contract,
        PreparedInspectionPolicy {
            authorization,
            metadata,
            diagnostic,
        },
    ))
}
