use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationOperation,
};

use crate::domain_computation::authorization::{
    WorthQueryAdmissionPrincipal, WorthQueryAdmissionScope,
};
use {
    super::super::WorthQueryOperationAuthorizationDenial,
    super::super::WorthQueryOperationScopeBinding,
    crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind,
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
    runtime: &dyn crate::domain_computation::authorization::host::AuthorizationHost<Schema>,
    principal: &dyn WorthQueryAdmissionPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: &dyn WorthQueryAdmissionScope<Schema, Scope>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> WorthQueryOperationScopeBinding {
    WorthQueryOperationScopeBinding::mint(
        runtime.authority_identity(),
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
    runtime: &dyn crate::domain_computation::authorization::host::AuthorizationHost<Schema>,
    principal: &dyn WorthQueryAdmissionPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: &dyn WorthQueryAdmissionScope<Schema, Scope>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    validate_static_authority_prefix(runtime, principal, scope, operation)?;
    runtime
        .installed_packages()
        .validate_application_operation(operation)
        .map_err(|_| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::StaleInstalledOperation,
                ),
                operation.operation(),
            )
        })
}

fn validate_static_authority_prefix<Schema, Principal, PrincipalIdentity, Operation, Input, Scope>(
    runtime: &dyn crate::domain_computation::authorization::host::AuthorizationHost<Schema>,
    principal: &dyn WorthQueryAdmissionPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: &dyn WorthQueryAdmissionScope<Schema, Scope>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    if !runtime
        .authorization_clock()
        .admission_clock
        .same_clock(principal.clock())
    {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ForeignRuntime,
            ),
            principal.binding(),
        ));
    }
    if principal.is_expired() {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ExpiredAuthentication,
            ),
            principal.binding(),
        ));
    }
    let authority = runtime.authority_identity();
    if principal.runtime_authority() != authority || scope.runtime_authority() != authority {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ForeignRuntime,
            ),
            operation.operation(),
        ));
    }
    if principal.binding_identity() != operation.binding_identity()
        || scope.binding_identity() != operation.binding_identity()
        || runtime.installed_schema().binding_identity() != *operation.binding_identity()
    {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::StaleInstalledSchema,
            ),
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
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::InconsistentDecision,
            ),
            policy,
        ));
    }
    if bridge.is_allowed() {
        Ok(())
    } else {
        Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::PermissionDenied,
            ),
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
            WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
                crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::Cancelled,
            )),
            subject,
        )),
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
                crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::TimedOut,
            )),
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

/// Clock custody is checked before any validity decision or graph work.
pub(in crate::domain_computation::authorization) fn validate_admission_clock(
    runtime_clock: &crate::domain_computation::authorization::WorthQueryRuntimeClock,
    principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock,
    request: &WorthQueryRequestScope,
    subject: &str,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    if runtime_clock.admission_clock.same_clock(principal) && principal.same_clock(request.clock())
    {
        Ok(())
    } else {
        Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ForeignRuntime,
            ),
            subject,
        ))
    }
}
