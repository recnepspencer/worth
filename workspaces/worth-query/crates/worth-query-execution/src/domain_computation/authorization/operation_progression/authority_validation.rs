//! Exact operation-authority validation phase owner.

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationOperation,
};

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
};

use super::WorthQueryCapabilityOperationProgression;

mod precondition_binding;
pub(in crate::domain_computation) use precondition_binding::admit_capability_access;
pub use precondition_binding::WorthQueryAdmittedApplicationCapabilityAccess;
pub use precondition_binding::WorthQueryAdmittedApplicationOperation;
pub(in crate::domain_computation) use precondition_binding::WorthQueryOperationAdmissionIdentity;
pub(super) use precondition_binding::{
    bind_capability_preconditions, bind_conventional_preconditions,
    transition_capability_operation, transition_conventional_operation,
};
pub(in crate::domain_computation::authorization) use precondition_binding::{
    WorthQueryCapabilityContextKey, WorthQueryCurrentCapabilityObservation,
    WorthQueryDelegationResolvedRequest, WorthQueryExactCapabilityObservationContext,
    WorthQueryResolvedCapabilityRequest,
};

pub(in crate::domain_computation::authorization::operation_progression) struct ValidatedConventionalOperation<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Operation,
    Input,
    Scope,
> {
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &'a crate::basis::WorthQueryProductBranchLease,
    principal: &'a WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
    operation: &'a WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    request: &'a WorthQueryRequestScope,
}

pub(super) fn validate_conventional_operation<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Operation,
    Input,
    Scope,
>(
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &'a crate::basis::WorthQueryProductBranchLease,
    principal: &'a WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
    operation: &'a WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    request: &'a WorthQueryRequestScope,
) -> Result<
    ValidatedConventionalOperation<
        'a,
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
    crate::domain_computation::authorization::admission::admit_request(
        request,
        operation.operation(),
    )?;
    if operation.contracts().authorization().requires_capability() {
        return Err(WorthQueryOperationAuthorizationDenial::new(
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::CapabilityRequired,
            operation.operation(),
        ));
    }
    crate::domain_computation::authorization::admission::validate_static_authority(
        runtime, principal, scope, operation,
    )?;
    Ok(ValidatedConventionalOperation {
        runtime,
        product,
        principal,
        scope,
        operation,
        request,
    })
}

pub(in crate::domain_computation::authorization::operation_progression) struct ValidatedCapabilityOperation<
    'a,
    Schema,
    Capability,
    Operation,
    Input,
> where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    operation: &'a WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
}

pub(super) fn validate_capability_operation<'a, Schema, Capability, Operation, Input>(
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    operation: &'a WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    progression: WorthQueryCapabilityOperationProgression,
) -> Result<
    ValidatedCapabilityOperation<'a, Schema, Capability, Operation, Input>,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    access.validate_operation_authority(runtime, operation, progression)?;
    Ok(ValidatedCapabilityOperation {
        runtime,
        access,
        operation,
    })
}
