//! Fixed metadata policy is selected before the original target is sized or copied.
use super::*;
use crate::domain_computation::{
    authorization::application_disclosure::WorthQueryPendingApplicationQueryGovernance,
    primary_graph::{
        admitted_capture::{checkpoint, run_retained_stage, RetainedStorageDenial},
        WorthQueryIssuedSelectedPrincipal, WorthQueryIssuedSelectedScope,
        WorthQuerySelectedProductOperation,
    },
    provider_session::{
        WorthQueryGraphWorkAccessContextAffinity, WorthQueryPreparedProductReadSessionIdentity,
    },
};
use worth_execution::MapKernelFailure;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryScopeBinding,
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
    original: InspectionSubject<'_>,
    pending: Option<WorthQueryPendingApplicationQueryGovernance>,
    comparison: Option<InspectionComparisonPreparation<'_>>,
    memory: &'storage mut Option<ExecutionMemoryReservation>,
) -> Result<InspectionRead<'a, 'storage, Schema, Scope<Schema, Binding>>, InspectionAdmissionDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    let reserved = WorthQueryPreparedProductReadSessionIdentity::reserve()
        .map_err(InspectionAdmissionDenial::Session)?;
    let before = comparison.as_ref().map(|comparison| comparison.before);
    if let Some(before) = before {
        InspectionSubject::Query(before).validate(
            runtime,
            installed,
            selected,
            scope.scope().entity_id(),
        )?;
    }
    let (contract, policy) = super::policy::prepare(
        runtime,
        execution,
        installed,
        selected,
        principal,
        scope,
        request,
        original,
        pending,
        comparison.map(|v| v.pending),
        &reserved,
        memory,
    )?;
    let target = original.capture(execution, before, contract, memory)?;
    start(
        runtime,
        execution,
        selected,
        principal.principal().principal_entity_id(),
        scope.scope(),
        request,
        target,
        policy,
        reserved,
        None,
        memory,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn start<
    'a,
    'storage,
    Schema: ApplicationSchema,
    Scope,
>(
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    principal: worth_relational::facade::identity::EntityId,
    scope: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
    request: &'a WorthQueryRequestScope,
    target: Arc<WorthQueryApplicationInspectionTarget>,
    policy: super::policy::PreparedInspectionPolicy,
    reserved: WorthQueryPreparedProductReadSessionIdentity,
    source: Option<super::policy::PreparedInspectionPolicy>,
    memory: &'storage mut Option<ExecutionMemoryReservation>,
) -> Result<InspectionRead<'a, 'storage, Schema, Scope>, InspectionAdmissionDenial> {
    let product = selected.product().read_lease();
    let graph = runtime
        .runtime
        .primary_graph()
        .ok_or(InspectionAdmissionDenial::MissingGraph)?;
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
                principal,
                WorthQueryGraphWorkAccessContextAffinity::entity(scope.entity_id()),
                basis_ref,
                product,
                runtime.graph_work_provider_identity(),
                graph.product_read_session_port(),
            ),
        ))
    })
    .map_err(InspectionAdmissionDenial::Storage)?;
    let mut session = session.map_err(InspectionAdmissionDenial::Session)?;
    let super::policy::PreparedInspectionPolicy {
        authorization,
        metadata,
        mut diagnostic,
    } = policy;
    let binding = governance_binding::capture(
        execution,
        memory,
        &session,
        &target,
        target.parameters(),
        principal,
        scope.entity_id(),
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
                target.parameters(),
                principal,
                scope.entity_id(),
            )?;
            diagnostic
                .bind_inspection(binding)
                .map_err(InspectionAdmissionDenial::Governance)
        })
        .transpose()?;
    let source = source
        .map(|policy| {
            super::connected::bind_source(
                execution,
                memory,
                &session,
                &target,
                principal,
                scope.entity_id(),
                policy,
            )
        })
        .transpose()?;
    session.set_retained_decision_facts(
        source.as_ref().map_or(0, |v| v.fact_count())
            + authorization.exact_fact_count()
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
        scope,
        request,
        principal: principal,
        basis,
        target,
        session,
        authorization,
        governance,
        diagnostic,
        source,
        memory,
    };
    drop(admitted.check(execution)?);
    Ok(admitted)
}
