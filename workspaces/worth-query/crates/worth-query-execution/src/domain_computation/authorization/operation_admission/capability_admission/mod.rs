//! Current capability request admission progression.

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationCapability,
};

use crate::domain_computation::authorization::admission::admit_request;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::{
    WorthQueryApprovedElevation, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrimaryGraphApplicationRuntime,
};

mod preparation;

pub use preparation::WorthQueryAdmittedApplicationCapabilityAccess;
use preparation::{complete_capability_admission, prepare_capability_admission};
pub(in crate::domain_computation::authorization) use preparation::{
    WorthQueryCapabilityContextKey, WorthQueryResolvedCapabilityRequest,
};
pub(in crate::domain_computation::authorization) use preparation::{
    WorthQueryCurrentCapabilityObservation, WorthQueryDelegationResolvedRequest,
    WorthQueryExactCapabilityObservationContext,
};

pub(in crate::domain_computation) fn admit_capability_access<
    Schema,
    Principal,
    PrincipalIdentity,
    Capability,
    Operation,
    Input,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &crate::basis::WorthQueryProductBranchLease,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    input: Input,
    request: &WorthQueryRequestScope,
    approved: Option<&WorthQueryApprovedElevation>,
) -> Result<
    WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
    Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
            Schema,
        > + 'static,
    <Operation as worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
            Value = Input,
        >,
    Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
{
    admit_request(request, capability.contract().operation())?;
    let prepared = prepare_capability_admission(
        runtime, product, principal, capability, input, request, approved,
    )?;
    complete_capability_admission(prepared)
}
