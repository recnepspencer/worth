//! Conventional-operation admission phases.

use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryCanonicalWorkEvidence, WorthQueryCanonicalWorkPhases,
    WorthQueryInstalledApplicationOperation,
};

use super::{
    WorthQueryAdmittedApplicationOperation, WorthQueryOperationAdmissionIdentity,
    WorthQueryOperationAuthorizationBasis,
};
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryRetainedAuthorizationDecisionFacts,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
    WorthQueryBoundMutationPreconditions, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::provider_session::{
    WorthQueryGraphWorkAccessContextAffinity, WorthQueryManagedGraphWorkSession,
};

struct ConventionalAdmissionPreparation<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Operation,
    Input,
    Scope,
> {
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &'a WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
    operation: &'a WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    request: &'a worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    preconditions: WorthQueryBoundMutationPreconditions,
    admission_identity: WorthQueryOperationAdmissionIdentity,
    resource_binding_identity: Arc<str>,
    graph_work: WorthQueryManagedGraphWorkSession,
}

struct ObservedConventionalAdmission<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Operation,
    Input,
    Scope,
> {
    preparation: ConventionalAdmissionPreparation<
        'a,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
    authorization: WorthQueryRetainedAuthorizationDecisionFacts,
}

pub(super) fn transition<Schema, Principal, PrincipalIdentity, Operation, Input, Scope>(
    bound: super::super::super::PreconditionBoundConventionalOperation<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
) -> Result<
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
{
    let prepared = prepare_identity_and_graph_work(bound)?;
    let observed = observe_current_authorization(prepared)?;
    Ok(construct_admitted_operation(observed))
}

fn prepare_identity_and_graph_work<Schema, Principal, PrincipalIdentity, Operation, Input, Scope>(
    bound: super::super::super::PreconditionBoundConventionalOperation<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
) -> Result<
    ConventionalAdmissionPreparation<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
{
    let super::super::super::PreconditionBoundConventionalOperation {
        validated,
        preconditions,
    } = bound;
    let super::super::super::super::ValidatedConventionalOperation {
        runtime,
        product,
        principal,
        scope,
        operation,
        request,
    } = validated;
    let admission_identity = WorthQueryOperationAdmissionIdentity::mint().ok_or_else(|| {
        WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::AdmissionIdentityExhausted,
            operation.operation(),
        )
    })?;
    let resource_binding_identity = admission_identity.resource_binding_identity();
    let graph_work =
        crate::domain_computation::authorization::graph_work_session::start_operation_graph_work(
            runtime,
            product.retained_clone(),
            operation,
            &resource_binding_identity,
            principal.principal_entity_id(),
            WorthQueryGraphWorkAccessContextAffinity::entity(scope.entity_id()),
        )?;
    Ok(ConventionalAdmissionPreparation {
        runtime,
        principal,
        scope,
        operation,
        request,
        preconditions,
        admission_identity,
        resource_binding_identity,
        graph_work,
    })
}

fn observe_current_authorization<Schema, Principal, PrincipalIdentity, Operation, Input, Scope>(
    mut preparation: ConventionalAdmissionPreparation<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
) -> Result<
    ObservedConventionalAdmission<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
{
    let authorization = preparation.runtime.observe_operation_authorization(
        preparation.principal,
        preparation.scope,
        preparation.operation,
        &preparation.graph_work,
    )?;
    crate::domain_computation::authorization::admission::admit_request(
        preparation.request,
        preparation.operation.operation(),
    )?;
    if preparation.principal.is_expired() {
        return Err(WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::ExpiredAuthentication,
            preparation.principal.binding(),
        ));
    }
    preparation
        .graph_work
        .record_decision_facts(authorization.exact_fact_count());
    Ok(ObservedConventionalAdmission {
        preparation,
        authorization,
    })
}

fn construct_admitted_operation<Schema, Principal, PrincipalIdentity, Operation, Input, Scope>(
    observed: ObservedConventionalAdmission<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
) -> WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope> {
    let ObservedConventionalAdmission {
        preparation,
        authorization,
    } = observed;
    let contracts = preparation.operation.contracts().clone();
    let canonical_work = WorthQueryCanonicalWorkPhases::new(
        contracts.canonical_work(),
        preparation.preconditions.canonical_work(),
        WorthQueryCanonicalWorkEvidence::zero(),
        WorthQueryCanonicalWorkEvidence::zero(),
        WorthQueryCanonicalWorkEvidence::zero(),
    );
    WorthQueryAdmittedApplicationOperation {
        runtime_authority: preparation.runtime.runtime.authority_identity(),
        binding_identity: preparation.operation.binding_identity().clone(),
        operation: preparation.operation.operation().to_string(),
        operation_authority_identity: preparation.operation.authority_identity().into(),
        operation_authority_identity_bytes: preparation.operation.authority_identity_bytes(),
        admission_identity: preparation.admission_identity,
        resource_binding_identity: preparation.resource_binding_identity,
        operation_scope_binding:
            crate::domain_computation::authorization::admission::operation_scope_binding(
                preparation.runtime,
                preparation.principal,
                preparation.scope,
                preparation.operation,
            ),
        canonical_work,
        scope_entity_id: preparation.scope.entity_id(),
        scope_entity_kind: preparation.scope.entity_kind(),
        scope_entity_name: preparation.scope.entity_name().to_string(),
        authentication_valid_until: preparation.principal.valid_until(),
        request_scope: preparation.request.clone(),
        contracts,
        mutation_preconditions: preparation.preconditions,
        authorization: Some(authorization),
        governed_input_identity: None,
        authorization_basis: WorthQueryOperationAuthorizationBasis::Conventional,
        graph_work: preparation.graph_work,
        _marker: PhantomData,
    }
}
