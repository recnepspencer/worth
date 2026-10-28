//! Fixed metadata policy is selected before the original target is sized or copied.
use super::*;
use crate::domain_computation::{
    authorization::application_disclosure::{
        compile_diagnostic_disclosure_contract, compile_inspection_disclosure_contract,
        prepare_application_diagnostic_governance, prepare_application_inspection_governance,
        WorthQueryPendingApplicationQueryGovernance,
    },
    primary_graph::{
        admitted_capture::{checkpoint, run_retained_stage, RetainedStorageDenial},
        application_query::{SelectedIssuedAccessRoot, WorthQueryApplicationQueryAccessContext},
        WorthQueryIssuedSelectedPrincipal, WorthQueryIssuedSelectedScope,
        WorthQueryPrincipalResolutionMode, WorthQuerySelectedProductOperation,
    },
    provider_session::{
        WorthQueryGraphWorkAccessContextAffinity, WorthQueryPreparedProductReadSessionIdentity,
    },
};
use worth_execution::MapKernelFailure;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryInspectionMetadataFacet as Facet,
    ApplicationQueryScopeBinding,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationQueryBinding,
};

type Scope<Schema, Binding> =
    <<Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<
        Schema,
    >>::Scope;

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn prepare<
    'a,
    'storage,
    Schema,
    Binding,
>(
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    principal: &'a WorthQueryIssuedSelectedPrincipal<
        '_,
        '_,
        Schema,
        Binding::Principal,
        Binding::PrincipalIdentity,
    >,
    scope: &'a WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope<Schema, Binding>>,
    request: &'a WorthQueryRequestScope,
    original: &super::super::super::application_observation::WorthQueryApplicationQueryObservation,
    pending: Option<WorthQueryPendingApplicationQueryGovernance>,
    comparison: Option<InspectionComparisonPreparation<'_>>,
    memory: &'storage mut Option<ExecutionMemoryReservation>,
) -> Result<InspectionRead<'a, 'storage, Schema, Scope<Schema, Binding>>, InspectionAdmissionDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    require_request(execution, request)?;
    let contract = validate_original_affinity(runtime, installed, original)?;
    let original_branch = |selection: &crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity| {
        matches!(selection,
            crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Product(original)
                if original.same_branch_occurrence_observation(selected.product().observation()))
    };
    if !original_branch(&original.source.selection) || !original_branch(original.basis.selection())
    {
        return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
    }
    if !std::ptr::eq(runtime, selected.application())
        || original.source.retained_scope_selector().subject() != scope.scope().entity_id()
    {
        return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
    }
    if let Some(comparison) = &comparison {
        validate_original_affinity(runtime, installed, comparison.before)?;
        if !original_branch(&comparison.before.source.selection)
            || !original_branch(comparison.before.basis.selection())
            || comparison.before.source.retained_scope_selector().subject()
                != scope.scope().entity_id()
        {
            return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
        }
    }
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
    let reserved = WorthQueryPreparedProductReadSessionIdentity::reserve()
        .map_err(InspectionAdmissionDenial::Session)?;
    let (authorization, _) = runtime
        .observe_issued_query_read_authorization(execution, installed.query(), &root, &reserved)
        .map_err(InspectionAdmissionDenial::Authorization)?;
    let disclosure = compile_inspection_disclosure_contract(&contract, execution, memory)
        .map_err(InspectionAdmissionDenial::Storage)?;
    let mut metadata = prepare_application_inspection_governance(disclosure, pending)
        .map_err(InspectionAdmissionDenial::Governance)?;
    let product = selected.product().read_lease();
    if let Some(capability) = metadata.authorization_mut() {
        runtime
            .refresh_capability_authorization_for_prepared_read(
                execution, capability, &reserved, &product,
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
        ) || comparison.is_some()
        {
            if !metadata.is_disclosed(slot) {
                return Err(InspectionAdmissionDenial::FacetDenied);
            }
        }
    }
    require_request(execution, request)?;
    let before = comparison.as_ref().map(|comparison| comparison.before);
    let mut diagnostic = match comparison {
        Some(comparison) => {
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
                comparison.pending,
                execution,
                memory,
            )
            .map_err(InspectionAdmissionDenial::DiagnosticPreparation)?;
            if let Some(capability) = diagnostic.authorization_mut() {
                runtime
                    .refresh_capability_authorization_for_prepared_read(
                        execution, capability, &reserved, &product,
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
    let target = WorthQueryApplicationInspectionTarget::capture(
        execution, original, before, contract, memory,
    )
    .map_err(InspectionAdmissionDenial::Target)?;
    let plan = plan::admit(runtime, execution, target.contract(), memory)
        .map_err(InspectionAdmissionDenial::Plan)?;
    let basis_ref = selected.application_basis().identity();
    let (basis, session) = run_retained_stage(execution, memory, |context, memory| {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        let bytes = basis_ref
            .retained_clone_heap_bytes()
            .and_then(|own| {
                own.checked_add(
                    WorthQueryManagedGraphWorkSession::inspection_start_backing_bytes(
                        basis_ref,
                        &product,
                        runtime.graph_work_provider_identity(),
                    )?,
                )
            })
            .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
        checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
        if let Some(memory) = memory {
            memory.resize(bytes).map_err(MapKernelFailure::Domain)?;
        }
        Ok((
            basis_ref.clone(),
            WorthQueryManagedGraphWorkSession::start_inspection(
                reserved,
                Arc::clone(&target),
                plan,
                runtime.runtime.authority_identity(),
                principal.principal().principal_entity_id(),
                WorthQueryGraphWorkAccessContextAffinity::entity(scope.scope().entity_id()),
                basis_ref,
                product,
                runtime.graph_work_provider_identity(),
                graph.product_read_session_port(),
            ),
        ))
    })
    .map_err(InspectionAdmissionDenial::Storage)?;
    let mut session = session.map_err(InspectionAdmissionDenial::Session)?;
    let binding = governance_binding::capture(
        execution,
        memory,
        &session,
        &target,
        original.source.parameter_binding_identity,
        principal.principal().principal_entity_id(),
        scope.scope().entity_id(),
    )?;
    let governance = metadata
        .bind_inspection(binding)
        .map_err(InspectionAdmissionDenial::Governance)?;
    let diagnostic = diagnostic
        .take()
        .map(|diagnostic| {
            let binding = governance_binding::capture(
                execution,
                memory,
                &session,
                &target,
                original.source.parameter_binding_identity,
                principal.principal().principal_entity_id(),
                scope.scope().entity_id(),
            )?;
            diagnostic
                .bind_inspection(binding)
                .map_err(InspectionAdmissionDenial::Governance)
        })
        .transpose()?;
    session.set_retained_decision_facts(
        authorization.exact_fact_count()
            + governance
                .authorization()
                .map_or(0, |capability| capability.exact_fact_count())
            + diagnostic
                .as_ref()
                .and_then(|governance| governance.authorization())
                .map_or(0, |capability| capability.exact_fact_count()),
    );
    let mut admitted = InspectionRead {
        runtime,
        scope: scope.scope(),
        request,
        principal: principal.principal().principal_entity_id(),
        basis,
        target,
        session,
        proof: None,
        authorization,
        governance,
        diagnostic,
        memory,
    };
    admitted.check(execution)?;
    Ok(admitted)
}
