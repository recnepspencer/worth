//! Named preflight checks for capability admission preparation.

use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityRequest, ApplicationCapabilityRequestProjection,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableType;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationCapability,
    WorthQueryInstalledApplicationOperationGraphAuthority,
};

use super::denial;
use crate::domain_computation::authorization::capability_elevation_projection::validate_elevation_projection;
use crate::domain_computation::authorization::capability_registry::WorthQueryInstalledCapabilityPlan;
use crate::domain_computation::authorization::graph_work_session::start_capability_graph_work;
use crate::domain_computation::authorization::{
    bridge_authorization_binding_identity, WorthQueryOperationAdmissionIdentity,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryRuntimeTimeSample,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApprovedElevation, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::provider_session::{
    WorthQueryGraphWorkAccessContextAffinity, WorthQueryManagedGraphWorkSession,
};

pub(super) fn validate_static_authority<
    Schema,
    Principal,
    PrincipalIdentity,
    Capability,
    Operation,
    Input,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: worth_query_declaration::facade::portable_identity::WorthQueryPortableType + 'static,
{
    if principal.is_expired() {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::ExpiredAuthentication,
            principal.binding(),
        ));
    }
    if principal.runtime_authority() != runtime.runtime.authority_identity() {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::ForeignRuntime,
            capability.contract().name(),
        ));
    }
    if principal.binding_identity() != capability.binding_identity()
        || runtime.installed_schema.binding_identity() != *capability.binding_identity()
    {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::StaleInstalledSchema,
            capability.contract().name(),
        ));
    }
    runtime
        .installed_schema
        .validate_installed_capability(capability)
        .map_err(|rejection| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::StaleInstalledOperation,
                rejection.subject(),
            )
        })
}

pub(super) fn admit_installed_plan<'a, Schema, Capability, Operation, Input>(
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    approved: Option<&WorthQueryApprovedElevation>,
) -> Result<&'a WorthQueryInstalledCapabilityPlan, WorthQueryOperationAuthorizationDenial> {
    let installed = runtime
        .authorization
        .capability_plan(capability)
        .ok_or_else(|| policy_not_installed(capability))?;
    validate_approved_applicability(installed, capability, approved)?;
    let rules = installed.rules().iter().map(|rule| rule.bridge());
    if !runtime.authorization.bridge().matches_installed_policy(
        installed.correspondence(),
        &bridge_authorization_binding_identity(capability.binding_identity()),
        installed.contract().name(),
        &installed.request().resource_entity,
        installed.contract().operation(),
        rules,
    ) {
        return Err(policy_not_installed(capability));
    }
    Ok(installed)
}

pub(super) fn project_request<Schema, Capability, Operation, Input>(
    input: &Input,
    installed: &WorthQueryInstalledCapabilityPlan,
    capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    approved: Option<&WorthQueryApprovedElevation>,
) -> Result<
    ApplicationCapabilityRequestProjection<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Context,
    >,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let projection = input.capability_request().map_err(|rejection| {
        denial(
            WorthQueryOperationAuthorizationDenialKind::CapabilityProjectionRejected,
            rejection.subject(),
        )
    })?;
    validate_elevation_projection(installed.contract(), &projection)?;
    require_approved_transition(installed, capability, approved)?;
    Ok(projection)
}

pub(super) fn sample_trusted_time<Schema, Capability, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    installed: &WorthQueryInstalledCapabilityPlan,
) -> Result<WorthQueryRuntimeTimeSample, WorthQueryOperationAuthorizationDenial> {
    runtime
        .authorization_clock
        .sample(installed.request().timeline)
        .map_err(|_| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::TrustedTimeUnavailable,
                capability.contract().name(),
            )
        })
}

pub(super) fn resolve_installed_operation<Schema, Capability, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
) -> Result<
    WorthQueryInstalledApplicationOperationGraphAuthority<Schema, Operation, Input>,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: WorthQueryPortableType + 'static,
{
    runtime
        .installed_schema
        .installed_operation_for_capability(capability)
        .map_err(|_| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::StaleInstalledOperation,
                capability.contract().operation(),
            )
        })
}

pub(super) fn mint_operation_admission<Schema, Capability, Operation, Input>(
    capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
) -> Result<WorthQueryOperationAdmissionIdentity, WorthQueryOperationAuthorizationDenial> {
    WorthQueryOperationAdmissionIdentity::mint().ok_or_else(|| {
        denial(
            WorthQueryOperationAuthorizationDenialKind::AdmissionIdentityExhausted,
            capability.contract().operation(),
        )
    })
}

pub(super) fn start_graph_work<Schema, Principal, PrincipalIdentity, Capability, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: crate::basis::WorthQueryProductBranchLease,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    operation: &WorthQueryInstalledApplicationOperationGraphAuthority<Schema, Operation, Input>,
) -> Result<WorthQueryManagedGraphWorkSession, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    start_capability_graph_work(
        runtime,
        product,
        operation,
        principal.principal_entity_id(),
        WorthQueryGraphWorkAccessContextAffinity::installed_capability(
            *capability.identity().bytes(),
        ),
    )
}

fn validate_approved_applicability<Schema, Capability, Operation, Input>(
    installed: &WorthQueryInstalledCapabilityPlan,
    capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    approved: Option<&WorthQueryApprovedElevation>,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    if installed.elevation().is_none() && approved.is_some() {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::ElevationNotApplicable,
            capability.contract().name(),
        ));
    }
    Ok(())
}

fn require_approved_transition<Schema, Capability, Operation, Input>(
    installed: &WorthQueryInstalledCapabilityPlan,
    capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    approved: Option<&WorthQueryApprovedElevation>,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    if installed.elevation().is_some() && approved.is_none() {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::ElevationTransitionRequired,
            capability.contract().name(),
        ));
    }
    Ok(())
}

fn policy_not_installed<Schema, Capability, Operation, Input>(
    capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::PolicyNotInstalled,
        capability.contract().name(),
    )
}
