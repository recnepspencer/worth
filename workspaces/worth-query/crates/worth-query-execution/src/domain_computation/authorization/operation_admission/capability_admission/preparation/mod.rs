//! Static capability admission and graph-work preparation.

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityRequest, ApplicationCapabilityRequestProjection,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationCapability,
};

use crate::domain_computation::authorization::capability_registry::WorthQueryInstalledCapabilityPlan;
use crate::domain_computation::authorization::{
    WorthQueryOperationAdmissionIdentity, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryRuntimeTimeSample,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApprovedElevation, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession;

mod observation;
mod preflight;

pub use observation::WorthQueryAdmittedApplicationCapabilityAccess;
pub(in crate::domain_computation::authorization) use observation::{
    WorthQueryCapabilityContextKey, WorthQueryResolvedCapabilityRequest,
};
pub(in crate::domain_computation::authorization) use observation::{
    WorthQueryCurrentCapabilityObservation, WorthQueryDelegationResolvedRequest,
    WorthQueryExactCapabilityObservationContext,
};

pub(super) fn complete_capability_admission<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Capability,
    Operation,
    Input,
>(
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
    WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let observed = observation::observe_current_capability(prepared)?;
    Ok(observation::admit_observed_capability(observed))
}

pub(super) struct PreparedCapabilityAdmission<
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
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &'a WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    capability: &'a WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    approved: Option<&'a WorthQueryApprovedElevation>,
    input: Input,
    request_scope: WorthQueryRequestScope,
    installed: &'a WorthQueryInstalledCapabilityPlan,
    projection: ApplicationCapabilityRequestProjection<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Context,
    >,
    sample: WorthQueryRuntimeTimeSample,
    operation_admission_identity: WorthQueryOperationAdmissionIdentity,
    graph_work: WorthQueryManagedGraphWorkSession,
    _seal: PreparedSeal,
}

struct PreparedSeal;

impl<'a, Schema, Principal, PrincipalIdentity, Capability, Operation, Input>
    PreparedCapabilityAdmission<
        'a,
        Schema,
        Principal,
        PrincipalIdentity,
        Capability,
        Operation,
        Input,
    >
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    const fn runtime(&self) -> &'a WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime
    }

    const fn principal(
        &self,
    ) -> &'a WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity> {
        self.principal
    }

    const fn capability(
        &self,
    ) -> &'a WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input> {
        self.capability
    }

    const fn approved(&self) -> Option<&'a WorthQueryApprovedElevation> {
        self.approved
    }

    const fn installed(&self) -> &'a WorthQueryInstalledCapabilityPlan {
        self.installed
    }

    const fn projection(
        &self,
    ) -> &ApplicationCapabilityRequestProjection<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Context,
    > {
        &self.projection
    }

    const fn sample(&self) -> &WorthQueryRuntimeTimeSample {
        &self.sample
    }

    const fn graph_work(&self) -> &WorthQueryManagedGraphWorkSession {
        &self.graph_work
    }

    fn record_admission_decisions(&mut self) {
        self.graph_work.record_decision_facts(2);
    }
}

pub(super) fn prepare_capability_admission<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Capability,
    Operation,
    Input,
>(
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &crate::basis::WorthQueryProductBranchLease,
    principal: &'a WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    capability: &'a WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    input: Input,
    request: &WorthQueryRequestScope,
    approved: Option<&'a WorthQueryApprovedElevation>,
) -> Result<
    PreparedCapabilityAdmission<
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
    Operation: 'static,
    Input: ApplicationCapabilityRequest<Schema, Capability>
        + worth_query_declaration::facade::portable_identity::WorthQueryPortableType
        + 'static,
{
    preflight::validate_static_authority(runtime, principal, capability)?;
    let installed = preflight::admit_installed_plan(runtime, capability, approved)?;
    let projection = preflight::project_request(&input, installed, capability, approved)?;
    let sample = preflight::sample_trusted_time(runtime, capability, installed)?;
    let operation = preflight::resolve_installed_operation(runtime, capability)?;
    let operation_admission_identity = preflight::mint_operation_admission(capability)?;
    let graph_work = preflight::start_graph_work(
        runtime,
        product.retained_clone(),
        principal,
        capability,
        &operation,
    )?;
    Ok(PreparedCapabilityAdmission {
        runtime,
        principal,
        capability,
        approved,
        input,
        request_scope: request.clone(),
        installed,
        projection,
        sample,
        operation_admission_identity,
        graph_work,
        _seal: PreparedSeal,
    })
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
