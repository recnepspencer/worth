//! Public conventional mutation authority on one issued selected Product.

use std::{marker::PhantomData, sync::Arc};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_admission::integration::WorthQueryPreparedApplicationOperationGraphWork;
use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationScopeBinding},
    application_schema::{ApplicationEntityMarkerIdentity, TypedMutationPreconditions},
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryCanonicalWorkEvidence, WorthQueryCanonicalWorkPhases,
    WorthQueryCurrentRetainedMutationBinding, WorthQueryInstalledApplicationOperationAuthorization,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::{
    identity_admitted::{mint_operation_identity_admitted, AdmittedOperationIdentityStop},
    WorthQueryAdmittedApplicationOperation, WorthQueryOperationAuthorizationBasis,
};
use crate::domain_computation::authorization::admission::{
    validate_static_authority_retained, WorthQueryRetainedMutationStaticStop,
};
use crate::domain_computation::authorization::{
    start_selected_operation_graph_work_admitted, SelectedOperationGraphWorkStop,
    WorthQueryOperationScopeBinding, WorthQueryRetainedAuthorizationDecisionFacts,
};
use crate::domain_computation::primary_graph::{
    bind_empty_mutation_preconditions_admitted, capture_principal_currentness_admitted,
    EmptyMutationPreconditionAdmissionStop, InvalidationEditAdmission,
    PrincipalCurrentnessCaptureStop, SharedSelectedProductOperation,
    WorthQueryIssuedSelectedPrincipal, WorthQueryIssuedSelectedScope,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkAccessContextAffinity;

pub(in crate::domain_computation) enum SelectedConventionalAdmissionStop {
    ForeignIssuedSource,
    UnsupportedAuthorization,
    Interrupted(WorthQueryRequestInterruption),
    ExpiredAuthentication,
    Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial),
    Preconditions(EmptyMutationPreconditionAdmissionStop),
    IdentityExhausted,
    GraphWork(SelectedOperationGraphWorkStop),
    Principal(PrincipalCurrentnessCaptureStop),
    Admission(CompanionPreflightStop),
    AccountingOverflow,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation) fn authorize_public_mutation_on_selected<Schema, Binding, Scope>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    shared: &SharedSelectedProductOperation<'_, Schema>,
    current: &WorthQueryCurrentRetainedMutationBinding<'_, Schema, Binding>,
    prepared: &WorthQueryPreparedApplicationOperationGraphWork,
    principal: &WorthQueryIssuedSelectedPrincipal<
        '_,
        '_,
        Schema,
        Binding::Principal,
        Binding::PrincipalIdentity,
    >,
    scope: &WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope>,
    request: &WorthQueryRequestScope,
    admission: &mut InvalidationEditAdmission,
) -> Result<
    WorthQueryAdmittedApplicationOperation<Schema, Binding::Operation, Binding::Input, Scope>,
    SelectedConventionalAdmissionStop,
>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Scope: ApplicationEntityMarkerIdentity<Schema>,
    Binding::ScopeBinding: ApplicationMutationScopeBinding<Schema, Scope = Scope>,
{
    use SelectedConventionalAdmissionStop as Stop;
    admission.charge_external_work(6).map_err(Stop::Admission)?;
    if !std::ptr::eq(principal.selected(), shared.selected())
        || !std::ptr::eq(scope.selected(), shared.selected())
        || !principal.request().same_request(request)
        || !scope.request().same_request(request)
    {
        return Err(Stop::ForeignIssuedSource);
    }
    // Preserve the request-first refusal before static authority, identity
    // minting, or capacity and session preparation.
    admission.charge_external_work(1).map_err(Stop::Admission)?;
    if let Some(interruption) = request.interruption() {
        return Err(Stop::Interrupted(interruption));
    }
    let operation = current.operation();
    validate_static_authority_retained(
        runtime,
        principal.principal(),
        scope.scope(),
        current,
        admission,
    )
    .map_err(|stop| match stop {
        WorthQueryRetainedMutationStaticStop::Admission(stop) => Stop::Admission(stop),
        WorthQueryRetainedMutationStaticStop::Authorization(denial) => Stop::Authorization(denial),
        WorthQueryRetainedMutationStaticStop::AccountingOverflow => Stop::AccountingOverflow,
    })?;
    // Two contracts() accesses, authorization, requirements and the empty
    // cardinality check all precede the unsupported-authority refusal.
    admission.charge_external_work(5).map_err(Stop::Admission)?;
    if operation.contracts().authorization()
        != WorthQueryInstalledApplicationOperationAuthorization::Principal
        || !operation.contracts().ability_requirements().is_empty()
    {
        return Err(Stop::UnsupportedAuthorization);
    }
    let empty_preconditions_work = u64::try_from(std::mem::size_of::<
        TypedMutationPreconditions<Schema, Binding::Operation, Scope>,
    >())
    .map_err(|_| Stop::AccountingOverflow)?;
    admission
        .charge_external_work(empty_preconditions_work)
        .map_err(Stop::Admission)?;
    let preconditions = bind_empty_mutation_preconditions_admitted(
        TypedMutationPreconditions::<Schema, Binding::Operation, Scope>::default(),
        admission,
    )
    .map_err(Stop::Preconditions)?;
    let (admission_identity, resource_binding_identity) =
        mint_operation_identity_admitted(admission).map_err(|stop| match stop {
            AdmittedOperationIdentityStop::Admission(stop) => Stop::Admission(stop),
            AdmittedOperationIdentityStop::Exhausted => Stop::IdentityExhausted,
            AdmittedOperationIdentityStop::AccountingOverflow => Stop::AccountingOverflow,
        })?;
    // These getter arguments and the access-context scalar are formed by
    // this caller before the selected graph-session owner begins preflight.
    admission.charge_external_work(4).map_err(Stop::Admission)?;
    let mut graph_work = start_selected_operation_graph_work_admitted(
        runtime,
        shared,
        current,
        prepared,
        &resource_binding_identity,
        principal.principal().principal_entity_id(),
        WorthQueryGraphWorkAccessContextAffinity::entity(scope.scope().entity_id()),
        request,
        admission,
    )
    .map_err(Stop::GraphWork)?;
    admission.charge_external_work(2).map_err(Stop::Admission)?;
    let principal_dependency = capture_principal_currentness_admitted(
        runtime,
        principal.principal(),
        graph_work.identity(),
        admission,
    )
    .map_err(Stop::Principal)?;
    let decision_carrier_work = u64::try_from(std::mem::size_of::<
        WorthQueryRetainedAuthorizationDecisionFacts,
    >())
    .map_err(|_| Stop::AccountingOverflow)?;
    admission
        .charge_external_work(decision_carrier_work)
        .map_err(Stop::Admission)?;
    let authorization =
        WorthQueryRetainedAuthorizationDecisionFacts::principal(principal_dependency);
    // Principal currentness checks the enum, issuing session, decision
    // iterator, and its empty-case completion before returning a verdict.
    admission.charge_external_work(5).map_err(Stop::Admission)?;
    if !authorization.belongs_to_session(graph_work.identity()) {
        return Err(Stop::ForeignIssuedSource);
    }
    admission.charge_external_work(2).map_err(Stop::Admission)?;
    if let Some(interruption) = request.interruption() {
        return Err(Stop::Interrupted(interruption));
    }
    if principal.principal().is_expired() {
        return Err(Stop::ExpiredAuthentication);
    }
    // The one principal fact is counted and added to the session's retained
    // decision-fact count; neither operation is an authorization shortcut.
    admission.charge_external_work(3).map_err(Stop::Admission)?;
    graph_work.record_decision_facts(authorization.exact_fact_count());

    // Fund the exact owned subject strings, two authority Arc<str> backings,
    // and initialized carrier before constructing the admitted operation.
    admission.charge_external_work(4).map_err(Stop::Admission)?;
    let name_len = operation.operation().len();
    let authority_len = operation.authority_identity().len();
    let scope_len = scope.scope().entity_name().len();
    let authority_backing = crate::domain_computation::arc_str_layout::backing_bytes(authority_len)
        .ok_or(Stop::AccountingOverflow)?;
    let carrier_work = std::mem::size_of::<
        WorthQueryAdmittedApplicationOperation<Schema, Binding::Operation, Binding::Input, Scope>,
    >()
    .checked_add(name_len)
    .and_then(|n| n.checked_add(scope_len))
    .and_then(|n| n.checked_add(authority_len.checked_mul(2)?))
    .and_then(|n| {
        n.checked_add(
            crate::domain_computation::arc_str_layout::initialized_header_work().checked_mul(2)?,
        )
    })
    .and_then(|n| u64::try_from(n).ok())
    .ok_or(Stop::AccountingOverflow)?;
    let carrier_bytes = name_len
        .checked_add(scope_len)
        .and_then(|n| n.checked_add(authority_backing.checked_mul(2)?))
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(Stop::AccountingOverflow)?;
    admission
        .charge_external_work(carrier_work)
        .and_then(|()| admission.admit_read_scratch(carrier_bytes))
        .map_err(Stop::Admission)?;
    let contracts = operation
        .retain_compiled_contracts_admitted(&mut |work, bytes| {
            admission.charge_external_work(work)?;
            admission.admit_read_scratch(bytes)
        })
        .map_err(Stop::Admission)?;
    let canonical_work = WorthQueryCanonicalWorkPhases::new(
        contracts.canonical_work(),
        preconditions.canonical_work(),
        WorthQueryCanonicalWorkEvidence::zero(),
        WorthQueryCanonicalWorkEvidence::zero(),
        WorthQueryCanonicalWorkEvidence::zero(),
    );
    // The local construction below can also wait on allocator service. Fund
    // the last safe-point checks now and run them after construction, before
    // handing authority to the executor.
    admission.charge_external_work(2).map_err(Stop::Admission)?;
    let admitted = WorthQueryAdmittedApplicationOperation {
        runtime_authority: runtime.runtime.authority_identity(),
        binding_identity: operation.binding_identity().clone(),
        operation: operation.operation().to_owned(),
        operation_authority_identity: Arc::from(operation.authority_identity()),
        operation_authority_identity_bytes: operation.authority_identity_bytes(),
        operation_definition_identity: operation.definition_identity_bytes(),
        admission_identity,
        resource_binding_identity,
        operation_scope_binding: WorthQueryOperationScopeBinding::mint(
            runtime.runtime.authority_identity(),
            operation.binding_identity(),
            operation.authority_identity(),
            principal.principal().principal_entity_id(),
            scope.scope().entity_id(),
        ),
        canonical_work,
        scope_entity_id: scope.scope().entity_id(),
        scope_entity_kind: scope.scope().entity_kind(),
        scope_entity_name: scope.scope().entity_name().to_owned(),
        authentication_valid_until: principal.principal().valid_until(),
        request_scope: request.clone(),
        contracts,
        mutation_preconditions: preconditions,
        authorization: Some(authorization),
        governed_input_identity: None,
        authorization_basis: WorthQueryOperationAuthorizationBasis::Conventional,
        graph_work,
        source_partition_identity: None,
        source_facts: Vec::new(),
        required_output_demand: None,
        _marker: PhantomData,
    };
    if let Some(interruption) = request.interruption() {
        return Err(Stop::Interrupted(interruption));
    }
    if principal.principal().is_expired() {
        return Err(Stop::ExpiredAuthentication);
    }
    Ok(admitted)
}
