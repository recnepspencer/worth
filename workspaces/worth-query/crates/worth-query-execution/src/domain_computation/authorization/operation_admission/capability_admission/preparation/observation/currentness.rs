//! Exact-snapshot currentness observation for one prepared capability admission.
mod approval_currentness;
use approval_currentness::{
    validate_approval_lifecycle, validate_approval_support, validate_approved_use,
};

use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentityBinding;
use worth_query_installation::facade::{ApplicationScalarValueBinding, ApplicationSchema};

use super::request_resolution::resolve_capability_request;
use super::{ObservedCapabilityAdmission, ObservedSeal, WorthQueryResolvedCapabilityRequest};
use crate::domain_computation::authorization::capability_observation::WorthQueryObservedCapabilityDecision;
use crate::domain_computation::authorization::delegation_admission::WorthQueryCapabilityObservationSource;
use crate::domain_computation::authorization::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryPrincipalCurrentnessDependency,
};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::{
    validate_freshness_at_snapshot, WorthQueryApprovedElevation,
    WorthQueryPrimaryPrincipalBindingLayout, WorthQueryPrincipalResolutionMode,
};

use super::super::PreparedCapabilityAdmission;

mod denial;
mod observation_context;
pub(in crate::domain_computation::authorization) use observation_context::WorthQueryCurrentCapabilityObservation;

struct CurrentCapabilityObservationAxes<'observation> {
    session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    relational: &'observation worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'observation worth_relational::facade::snapshots::SnapshotHandle,
    bridge: &'observation worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    installed: &'observation crate::domain_computation::authorization::capability_registry::WorthQueryInstalledCapabilityPlan,
    request: &'observation WorthQueryRetainedCapabilityRequest,
    sample: &'observation crate::domain_computation::authorization::WorthQueryRuntimeTimeSample,
}

struct CurrentPrincipalObservation {
    layout: WorthQueryPrimaryPrincipalBindingLayout,
    dependency: WorthQueryPrincipalCurrentnessDependency,
}

struct RelationalCapabilityObservation<Schema, Scope> {
    resolved: WorthQueryResolvedCapabilityRequest<Schema, Scope>,
    revalidation: WorthQueryRetainedCapabilityRequest,
    observed: WorthQueryObservedCapabilityDecision,
}

pub(super) fn observe_and_admit<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Capability,
    Operation,
    Input,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut prepared: PreparedCapabilityAdmission<
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
    validate_approval_lifecycle(&prepared)?;
    let principal = capture_current_principal(&prepared)?;
    let observation = observe_relational_truth(execution, &prepared, &principal)?;
    validate_admission(&prepared, &observation)?;
    prepared.record_admission_decisions();
    Ok(finish_admission(prepared, principal, observation))
}

fn capture_current_principal<Schema, Principal, Identity, Capability, Operation, Input>(
    prepared: &PreparedCapabilityAdmission<
        '_,
        Schema,
        Principal,
        Identity,
        Capability,
        Operation,
        Input,
    >,
) -> Result<CurrentPrincipalObservation, WorthQueryOperationAuthorizationDenial>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let graph = prepared.runtime().runtime.primary_graph().ok_or_else(|| {
        denial::authorization_denial(
            WorthQueryOperationAuthorizationDenialKind::ForeignRuntime,
            prepared.capability().contract().operation(),
        )
    })?;
    let layout = graph
        .layout()
        .principal_binding(prepared.principal().binding())
        .cloned()
        .ok_or_else(|| denial::stale_principal(prepared.principal().binding()))?;
    let dependency = WorthQueryPrincipalCurrentnessDependency::capture(
        prepared.graph_work().identity(),
        prepared.principal(),
        &layout,
    );
    Ok(CurrentPrincipalObservation { layout, dependency })
}

fn observe_relational_truth<Schema, Principal, Identity, Capability, Operation, Input>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    prepared: &PreparedCapabilityAdmission<
        '_,
        Schema,
        Principal,
        Identity,
        Capability,
        Operation,
        Input,
    >,
    principal: &CurrentPrincipalObservation,
) -> Result<
    RelationalCapabilityObservation<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    >,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let snapshot = prepared.graph_work().mutation_snapshot().unwrap().clone();
    let handle = prepared.graph_work().mutation_handle().unwrap().clone();
    let resolution = prepared
        .runtime()
        .runtime
        .primary_graph()
        .unwrap()
        .retain_entity_resolution_context();
    handle.with_runtime_mut(execution, |relational| {
        validate_principal_freshness(execution, prepared, principal, relational, &snapshot)?;
        let truth = resolution
            .at_snapshot(
                execution,
                relational,
                &snapshot,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(|_| denial::projection_rejected(prepared.capability().contract().name()))?;
        let resolved = resolve_capability_request(
            execution,
            &truth,
            &prepared.runtime().installed_schema,
            prepared.projection(),
        )?;
        let revalidation = WorthQueryRetainedCapabilityRequest::capture(
            *prepared.capability().identity().bytes(),
            prepared.principal().principal_entity_id(),
            prepared.projection(),
            &resolved,
        );
        validate_approval_support(execution, prepared, relational, &snapshot)?;
        let observation =
            WorthQueryCurrentCapabilityObservation::from_axes(CurrentCapabilityObservationAxes {
                session: prepared.graph_work().identity(),
                relational,
                snapshot: &snapshot,
                bridge: prepared.runtime().authorization.bridge(),
                installed: prepared.installed(),
                request: &revalidation,
                sample: prepared.sample(),
            });
        let observed = observation.observe_active_capability(
            execution,
            None,
            prepared
                .approved()
                .map(WorthQueryApprovedElevation::support_decision),
        )?;
        Ok(RelationalCapabilityObservation {
            resolved,
            revalidation,
            observed,
        })
    })
}

fn validate_principal_freshness<Schema, Principal, Identity, Capability, Operation, Input>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    prepared: &PreparedCapabilityAdmission<
        '_,
        Schema,
        Principal,
        Identity,
        Capability,
        Operation,
        Input,
    >,
    principal: &CurrentPrincipalObservation,
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let expected = WorthQueryExternalPrincipalIdentityBinding::encode(
        prepared.principal().external_identity(),
    )
    .map_err(|_| denial::stale_principal(prepared.principal().binding()))?;
    validate_freshness_at_snapshot(
        execution,
        relational,
        snapshot,
        prepared.principal(),
        &principal.layout,
        &expected,
    )
    .map_err(|_| denial::stale_principal(prepared.principal().binding()))
}

fn validate_admission<Schema, Principal, Identity, Capability, Operation, Input>(
    prepared: &PreparedCapabilityAdmission<
        '_,
        Schema,
        Principal,
        Identity,
        Capability,
        Operation,
        Input,
    >,
    observed: &RelationalCapabilityObservation<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    >,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    validate_approved_use(prepared, observed)?;
    crate::domain_computation::authorization::admission::admit_request(
        &prepared.request_scope,
        prepared.capability().contract().operation(),
    )?;
    if prepared.principal().is_expired() {
        return Err(denial::authorization_denial(
            WorthQueryOperationAuthorizationDenialKind::ExpiredAuthentication,
            prepared.principal().binding(),
        ));
    }
    Ok(())
}

fn finish_admission<'a, Schema, Principal, Identity, Capability, Operation, Input>(
    prepared: PreparedCapabilityAdmission<
        'a,
        Schema,
        Principal,
        Identity,
        Capability,
        Operation,
        Input,
    >,
    principal: CurrentPrincipalObservation,
    observed: RelationalCapabilityObservation<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    >,
) -> ObservedCapabilityAdmission<'a, Schema, Principal, Identity, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let authorization = observed
        .observed
        .into_retained_authorization(principal.dependency);
    ObservedCapabilityAdmission {
        prepared,
        resolved: observed.resolved,
        authorization,
        _seal: ObservedSeal,
    }
}
