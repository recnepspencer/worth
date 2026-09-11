//! Capability-access progression into installed operation authority.

use crate::domain_computation::authorization::graph_work_session::transition_capability_to_operation_graph_work;
use crate::domain_computation::authorization::operation_progression::WorthQueryCapabilityOperationProgression;
use crate::domain_computation::authorization::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAdmittedApplicationOperation,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryOperationScopeBinding,
};
use crate::domain_computation::primary_graph::{
    WorthQueryBoundMutationPreconditions, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_schema::{ApplicationOperationMarkerIdentity, TypedMutationPreconditions},
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryCanonicalWorkEvidence, WorthQueryCanonicalWorkPhases,
    WorthQueryInstalledApplicationOperation,
};

mod authority_validation;
mod precondition_binding;
mod projection_equivalence;

use super::super::super::super::super::{
    WorthQueryAdmittedApplicationOperation as OwnerAdmittedOperation,
    WorthQueryOperationAuthorizationBasis,
};
use authority_validation::{
    validate_operation_graph_work_authority, validate_progression_authority,
};
use precondition_binding::bind_progression_preconditions;
use projection_equivalence::same_projection;

struct RevalidatedOperationAuthority {
    authorization:
        crate::domain_computation::authorization::WorthQueryRetainedAuthorizationDecisionFacts,
    graph_work: crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn authorize_capability_operation<Capability, Operation, Input>(
        &self,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>,
    {
        crate::domain_computation::authorization::operation_progression::progress_capability_operation(
            self,
            access,
            operation,
            preconditions,
            WorthQueryCapabilityOperationProgression::Ordinary,
        )
    }
}

pub(in crate::domain_computation::authorization) fn validate_capability_operation_authority<
    Schema,
    Capability,
    Operation,
    Input,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    progression: WorthQueryCapabilityOperationProgression,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    validate_progression_authority(runtime, access, operation, progression)?;
    validate_current_projection(access)
}

pub(in crate::domain_computation::authorization) fn bind_capability_operation_preconditions<
    Schema,
    Capability,
    Operation,
    Input,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    preconditions: TypedMutationPreconditions<
        Schema,
        Operation,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    >,
) -> Result<WorthQueryBoundMutationPreconditions, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    bind_progression_preconditions(runtime, access, operation, preconditions)
}

pub(in crate::domain_computation::authorization) fn transition_capability_operation<
    Schema,
    Capability,
    Operation,
    Input,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    preconditions: WorthQueryBoundMutationPreconditions,
) -> Result<
    OwnerAdmittedOperation<
        Schema,
        Operation,
        Input,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    >,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let resource_binding_identity = access
        .operation_admission_identity
        .resource_binding_identity();
    let resource_entity_id = access.resource_entity_id();
    let operation_scope_binding = WorthQueryOperationScopeBinding::mint(
        runtime.runtime.authority_identity(),
        operation.binding_identity(),
        operation.authority_identity(),
        access.principal_entity_id,
        resource_entity_id,
    );
    let scope_entity_kind = access.resource_entity_kind();
    let scope_entity_name = access.resource_entity_name().to_string();
    let authentication_valid_until = access.authentication_valid_until;
    let request_scope = access.request_scope.clone();
    let super::WorthQueryAdmittedApplicationCapabilityAccess {
        input,
        governed_input_identity,
        canonical_work,
        authorization,
        operation_admission_identity,
        graph_work,
        ..
    } = access;
    let graph_work = transition_capability_to_operation_graph_work(
        runtime,
        operation,
        &resource_binding_identity,
        resource_entity_id,
        graph_work,
    )?;
    validate_operation_graph_work_authority(&graph_work, operation)?;
    let revalidated = revalidate_operation_authorization(
        runtime,
        authorization,
        graph_work,
        operation.operation(),
    )?;
    let contracts = operation.contracts().clone();
    let canonical_work = WorthQueryCanonicalWorkPhases::new(
        contracts.canonical_work(),
        preconditions.canonical_work().combine(canonical_work),
        WorthQueryCanonicalWorkEvidence::zero(),
        WorthQueryCanonicalWorkEvidence::zero(),
        WorthQueryCanonicalWorkEvidence::zero(),
    );
    Ok(OwnerAdmittedOperation {
        runtime_authority: runtime.runtime.authority_identity(),
        binding_identity: operation.binding_identity().clone(),
        operation: operation.operation().to_string(),
        operation_authority_identity: operation.authority_identity().into(),
        operation_authority_identity_bytes: operation.authority_identity_bytes(),
        admission_identity: operation_admission_identity,
        resource_binding_identity,
        operation_scope_binding,
        canonical_work,
        scope_entity_id: resource_entity_id,
        scope_entity_kind,
        scope_entity_name,
        authentication_valid_until,
        request_scope,
        contracts,
        mutation_preconditions: preconditions,
        authorization: Some(revalidated.authorization),
        governed_input_identity,
        authorization_basis: WorthQueryOperationAuthorizationBasis::Capability { input },
        graph_work: revalidated.graph_work,
        _marker: std::marker::PhantomData,
    })
}

fn revalidate_operation_authorization<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    mut authorization: crate::domain_computation::authorization::WorthQueryRetainedCapabilityAuthorization,
    mut graph_work: crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    subject: &str,
) -> Result<RevalidatedOperationAuthority, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    runtime.refresh_capability_authorization_for_graph_work(&mut authorization, &graph_work)?;
    graph_work.set_retained_decision_facts(authorization.exact_fact_count());
    let session = graph_work.identity();
    let authorization =
        crate::domain_computation::authorization::WorthQueryRetainedAuthorizationDecisionFacts::capability(
            authorization,
        );
    if !authorization.belongs_to_session(session) {
        return Err(WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
            subject,
        ));
    }
    Ok(RevalidatedOperationAuthority {
        authorization,
        graph_work,
    })
}

fn validate_current_projection<Schema, Capability, Operation, Input>(
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let current = access.input.capability_request().map_err(|rejection| {
        denial(
            WorthQueryOperationAuthorizationDenialKind::CapabilityProjectionRejected,
            rejection.subject(),
        )
    })?;
    if same_projection(&access.projection, &current) {
        Ok(())
    } else {
        Err(denial(
            WorthQueryOperationAuthorizationDenialKind::CapabilityProjectionRejected,
            access.operation.as_ref(),
        ))
    }
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
