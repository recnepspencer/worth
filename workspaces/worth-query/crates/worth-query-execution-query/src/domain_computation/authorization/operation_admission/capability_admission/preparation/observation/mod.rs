//! Currentness-bound capability observation owned by capability admission.

use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_installation::facade::ApplicationSchema;

use super::PreparedCapabilityAdmission;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryRetainedCapabilityAuthorization,
};

pub mod admitted_access;
mod currentness;
pub mod request_resolution;

pub use admitted_access::admit_observed_capability;
pub use admitted_access::WorthQueryAdmittedApplicationCapabilityAccess;
pub use admitted_access::{
    WorthQueryDelegationResolvedRequest, WorthQueryExactCapabilityObservationContext,
};
pub use currentness::WorthQueryCurrentCapabilityObservation;
pub use request_resolution::{
    WorthQueryCapabilityContextKey, WorthQueryResolvedCapabilityRequest,
};

pub struct ObservedCapabilityAdmission<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Capability,
    Operation,
    Input,
> where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    prepared: PreparedCapabilityAdmission<
        'a,
        Schema,
        Principal,
        PrincipalIdentity,
        Capability,
        Operation,
        Input,
    >,
    resolved: WorthQueryResolvedCapabilityRequest<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    >,
    authorization: WorthQueryRetainedCapabilityAuthorization,
    _seal: ObservedSeal,
}

struct ObservedSeal;

pub fn observe_current_capability<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Capability,
    Operation,
    Input,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    prepared: PreparedCapabilityAdmission<
        'a,
        Schema,
        Principal,
        PrincipalIdentity,
        Capability,
        Operation,
        Input,
    >,
) -> Result<
    ObservedCapabilityAdmission<
        'a,
        Schema,
        Principal,
        PrincipalIdentity,
        Capability,
        Operation,
        Input,
    >,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    currentness::observe_and_admit(execution, prepared)
}
