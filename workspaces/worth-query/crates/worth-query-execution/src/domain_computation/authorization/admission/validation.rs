use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationOperation,
};

use super::super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryOperationScopeBinding,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrimaryGraphApplicationRuntime,
};

mod retained_mutation;
pub(in crate::domain_computation::authorization) use retained_mutation::{
    validate_static_authority_retained, WorthQueryRetainedMutationStaticStop,
};

pub(in crate::domain_computation::authorization) fn operation_scope_binding<
    Schema,
    Principal,
    PrincipalIdentity,
    Operation,
    Input,
    Scope,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> WorthQueryOperationScopeBinding {
    WorthQueryOperationScopeBinding::mint(
        runtime.runtime.authority_identity(),
        operation.binding_identity(),
        operation.authority_identity(),
        principal.principal_entity_id(),
        scope.entity_id(),
    )
}

pub(in crate::domain_computation::authorization) fn validate_static_authority<
    Schema,
    Principal,
    PrincipalIdentity,
    Operation,
    Input,
    Scope,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    validate_static_authority_prefix(runtime, principal, scope, operation)?;
    runtime
        .runtime
        .installed_packages()
        .validate_application_operation(operation)
        .map_err(|_| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::StaleInstalledOperation,
                operation.operation(),
            )
        })
}

fn validate_static_authority_prefix<Schema, Principal, PrincipalIdentity, Operation, Input, Scope>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    if principal.is_expired() {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::ExpiredAuthentication,
            principal.binding(),
        ));
    }
    let authority = runtime.runtime.authority_identity();
    if principal.runtime_authority() != authority || scope.runtime_authority() != authority {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::ForeignRuntime,
            operation.operation(),
        ));
    }
    if principal.binding_identity() != operation.binding_identity()
        || scope.binding_identity() != operation.binding_identity()
        || runtime.installed_schema.binding_identity() != *operation.binding_identity()
    {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::StaleInstalledSchema,
            operation.operation(),
        ));
    }
    Ok(())
}

pub(super) fn validate_decision(
    bridge_runtime: &worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    relational: &worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    bridge: &worth_runtime_bridge::facade::BridgeAuthorizationDecisionEvidence,
    dependency_identity: [u8; 32],
    policy: &str,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    if relational.observation_identity().bytes() != &dependency_identity
        || bridge.dependency_identity() != &dependency_identity
        || !bridge_runtime.retains(bridge)
    {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
            policy,
        ));
    }
    if bridge.is_allowed() {
        Ok(())
    } else {
        Err(denial(
            WorthQueryOperationAuthorizationDenialKind::PermissionDenied,
            policy,
        ))
    }
}

pub(in crate::domain_computation::authorization) fn admit_request(
    scope: &WorthQueryRequestScope,
    subject: &str,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    match scope.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Cancelled,
            subject,
        )),
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => Err(denial(
            WorthQueryOperationAuthorizationDenialKind::DeadlineExceeded,
            subject,
        )),
        None => Ok(()),
    }
}

pub(super) fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
