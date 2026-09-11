use std::time::Instant;

use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationOperation,
};

use super::super::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::authorization::admission::admit_request;
use crate::domain_computation::authorization::operation_progression::WorthQueryCapabilityOperationProgression;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(super) fn validate_progression_authority<Schema, Capability, Operation, Input>(
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
    validate_access_lifecycle(access)?;
    validate_installed_operation_identity(runtime, access, operation, progression)?;
    validate_capability_graph_work_authority(access, operation)
}

fn validate_installed_operation_identity<Schema, Capability, Operation, Input>(
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
    let lifecycle = runtime
        .authorization
        .elevation_lifecycle_operation::<Schema, Operation>(
            operation.operation(),
            operation.input_type(),
        )
        .map_err(|()| stale_operation(operation.operation()))?;
    if lifecycle.is_some()
        && progression != WorthQueryCapabilityOperationProgression::ElevationLifecycle
    {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::ElevationTransitionRequired,
            operation.operation(),
        ));
    }
    if operation
        .execution_posture()
        .requires_delegation_activation()
        && progression != WorthQueryCapabilityOperationProgression::DelegationActivation
    {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::DelegationTransitionRequired,
            operation.operation(),
        ));
    }
    if operation
        .execution_posture()
        .requires_capability_revocation()
        && progression != WorthQueryCapabilityOperationProgression::CapabilityRevocation
    {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::DelegationTransitionRequired,
            operation.operation(),
        ));
    }
    if access.runtime_authority != runtime.runtime.authority_identity() {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::ForeignRuntime,
            access.operation.as_ref(),
        ));
    }
    if access.binding_identity != *operation.binding_identity()
        || runtime.installed_schema.binding_identity() != *operation.binding_identity()
    {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::StaleInstalledSchema,
            access.operation.as_ref(),
        ));
    }
    if access.operation.as_ref() != operation.operation() {
        return Err(stale_operation(access.operation.as_ref()));
    }
    runtime
        .runtime
        .installed_packages()
        .validate_application_operation(operation)
        .map_err(|_| stale_operation(operation.operation()))
}

fn validate_capability_graph_work_authority<Schema, Capability, Operation, Input>(
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    if access.graph_work.runtime_authority() != access.runtime_authority
        || access.graph_work.binding() != &access.binding_identity
        || access.graph_work.principal() != access.principal_entity_id
        || access.graph_work.capability_access_context()
            != Some(access.authorization.installed_capability_identity())
        || access.authorization.exact_fact_count() != access.graph_work.retained_decision_facts()
    {
        return Err(inconsistent(operation.operation()));
    }
    if !operation.contracts().authorization().requires_capability() {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::CapabilityNotRequired,
            operation.operation(),
        ));
    }
    Ok(())
}

pub(super) fn validate_operation_graph_work_authority<Schema, Operation, Input>(
    graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    if graph_work.binding() != operation.binding_identity()
        || graph_work.obligation() != operation.graph_obligations().identity()
        || graph_work.subject_authority() != operation.authority_identity()
    {
        return Err(inconsistent(operation.operation()));
    }
    Ok(())
}

fn validate_access_lifecycle<Schema, Capability, Operation, Input>(
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    admit_request(&access.request_scope, &access.operation)?;
    if Instant::now() >= access.authentication_valid_until {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::ExpiredAuthentication,
            access.operation.as_ref(),
        ));
    }
    Ok(())
}

fn stale_operation(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::StaleInstalledOperation,
        subject,
    )
}

fn inconsistent(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
        subject,
    )
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
