//! Attempt-bound capability access minted only by capability admission.

use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Instant;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityRequest, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityValidityTimeline,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaBindingIdentity, WorthQueryCanonicalWorkEvidence,
};
use worth_relational::facade::authorization::RelationalAuthorizationObservationCounters;

use super::request_resolution::WorthQueryResolvedCapabilityRequest;
use super::ObservedCapabilityAdmission;
use crate::domain_computation::authorization::{
    WorthQueryRetainedCapabilityAuthorization, WorthQueryRetainedCapabilitySupport,
};
mod exact_observation;
mod inspection;
mod operation_progression;

pub(in crate::domain_computation::authorization) use exact_observation::{
    WorthQueryDelegationResolvedRequest, WorthQueryExactCapabilityObservationContext,
};

/// Move-only Query authority proving that one exact capability request was
/// admitted from current graph truth.
///
/// This proof opens no execution surface. A separately installed operation
/// must consume it before an application attempt can begin.
///
/// Authentication is not admitted capability access:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAuthenticatedPrincipal,
/// };
///
/// fn cannot_substitute_authentication<Schema, Capability, Operation, Input, Principal, Identity>(
///     principal: WorthQueryAuthenticatedPrincipal<Schema, Principal, Identity>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _: WorthQueryAdmittedApplicationCapabilityAccess<
///         Schema, Capability, Operation, Input,
///     > = principal;
/// }
/// ```
///
/// A descriptive capability reference is not admitted capability access:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::{
///     ApplicationCapabilityRef, ApplicationCapabilityRequest,
/// };
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
///
/// fn cannot_substitute_description<Schema, Capability, Operation, Input>(
///     capability: ApplicationCapabilityRef<Schema, Capability>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _: WorthQueryAdmittedApplicationCapabilityAccess<
///         Schema, Capability, Operation, Input,
///     > = capability;
/// }
/// ```
///
/// The access proof is move-only:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
///
/// fn cannot_clone_access<Schema, Capability, Operation, Input>(
///     access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _copied = access.clone();
/// }
/// ```
///
/// Exact delegation lineage is private decision evidence, not a consumer
/// inspection or reconstruction surface:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
///
/// fn cannot_extract_lineage<Schema, Capability, Operation, Input>(
///     access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _lineage = access.delegation_lineage();
/// }
/// ```
///
/// A portable provenance marker cannot be promoted into access authority:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::{
///     ApplicationCapabilityProvenanceRef, ApplicationCapabilityRequest,
/// };
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
///
/// fn cannot_promote_provenance<Schema, Capability, Operation, Input, Provenance>(
///     provenance: ApplicationCapabilityProvenanceRef<Schema, Provenance>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _: WorthQueryAdmittedApplicationCapabilityAccess<
///         Schema, Capability, Operation, Input,
///     > = provenance;
/// }
/// ```
///
/// The operation marker is exact and cannot be repurposed:
///
/// ```compile_fail
/// use worth_query_declaration::facade::{
///     application_capability::ApplicationCapabilityRequest,
///     application_schema::TypedMutationPreconditions,
/// };
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryPrimaryGraphApplicationRuntime,
/// };
/// use worth_query_installation::facade::{
///     ApplicationSchema, WorthQueryInstalledApplicationOperation,
/// };
///
/// fn cannot_change_operation<Schema, Capability, Granted, Other, Input>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Granted, Input>,
///     other: &WorthQueryInstalledApplicationOperation<Schema, Other, Input>,
/// ) where
///     Schema: ApplicationSchema,
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     runtime.authorize_capability_operation(
///         access,
///         other,
///         TypedMutationPreconditions::new(),
///     );
/// }
/// ```
pub struct WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    binding_identity: ApplicationSchemaBindingIdentity,
    capability: Arc<str>,
    capability_type: Arc<str>,
    operation: Arc<str>,
    principal_entity_id: worth_relational::facade::identity::EntityId,
    input: Input,
    governed_input_identity: Option<[u8; 32]>,
    projection: ApplicationCapabilityRequestProjection<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Context,
    >,
    resolved: WorthQueryResolvedCapabilityRequest<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    >,
    authentication_valid_until: Instant,
    request_scope: WorthQueryRequestScope,
    canonical_work: WorthQueryCanonicalWorkEvidence,
    authorization: WorthQueryRetainedCapabilityAuthorization,
    operation_admission_identity: super::super::super::super::WorthQueryOperationAdmissionIdentity,
    graph_work: crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    _marker: PhantomData<fn() -> (Capability, Operation)>,
}

pub(in super::super) fn admit_observed_capability<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Capability,
    Operation,
    Input,
>(
    observed: ObservedCapabilityAdmission<
        'a,
        Schema,
        Principal,
        PrincipalIdentity,
        Capability,
        Operation,
        Input,
    >,
) -> WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let ObservedCapabilityAdmission {
        prepared,
        resolved,
        authorization,
        ..
    } = observed;
    let input = prepared.input;
    let governed_input_identity = input.governed_input_identity();
    let canonical_work = prepared
        .capability
        .lookup_evidence()
        .canonical_work()
        .combine(
            governed_input_identity
                .and_then(|binding| binding.canonical_work())
                .map(WorthQueryCanonicalWorkEvidence::one_digest)
                .unwrap_or_else(WorthQueryCanonicalWorkEvidence::zero),
        );
    WorthQueryAdmittedApplicationCapabilityAccess {
        runtime_authority: prepared.runtime.runtime.authority_identity(),
        binding_identity: prepared.capability.binding_identity().clone(),
        capability: prepared.capability.contract().name().into(),
        capability_type: prepared.capability.contract().capability_type().into(),
        operation: prepared.capability.contract().operation().into(),
        principal_entity_id: prepared.principal.principal_entity_id(),
        input,
        governed_input_identity: governed_input_identity.map(|binding| binding.identity()),
        projection: prepared.projection,
        resolved,
        authentication_valid_until: prepared.principal.valid_until(),
        request_scope: prepared.request_scope,
        canonical_work,
        authorization,
        operation_admission_identity: prepared.operation_admission_identity,
        graph_work: prepared.graph_work,
        _marker: PhantomData,
    }
}

impl<Schema, Capability, Operation, Input>
    WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    pub(in crate::domain_computation::authorization) fn validate_operation_authority(
        &self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        operation: &worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
            Schema,
            Operation,
            Input,
        >,
        progression: crate::domain_computation::authorization::operation_progression::WorthQueryCapabilityOperationProgression,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    where
        Schema: ApplicationSchema,
        Operation: ApplicationOperationMarkerIdentity<Schema>,
    {
        operation_progression::validate_capability_operation_authority(
            runtime,
            self,
            operation,
            progression,
        )
    }

    pub(in crate::domain_computation::authorization) fn bind_operation_preconditions(
        &self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        operation: &worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
            Schema,
            Operation,
            Input,
        >,
        preconditions: worth_query_declaration::facade::application_schema::TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: ApplicationSchema,
    {
        operation_progression::bind_capability_operation_preconditions(
            runtime,
            self,
            operation,
            preconditions,
        )
    }

    pub(in crate::domain_computation::authorization) fn transition_operation(
        self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        operation: &worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
            Schema,
            Operation,
            Input,
        >,
        preconditions: crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: ApplicationSchema,
    {
        operation_progression::transition_capability_operation(
            runtime,
            self,
            operation,
            preconditions,
        )
    }

    pub(in crate::domain_computation::authorization) const fn capability_input(&self) -> &Input {
        &self.input
    }

    pub(in crate::domain_computation::authorization) fn with_exact_observation<Output>(
        &self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        observe: impl FnOnce(
            &exact_observation::WorthQueryExactCapabilityObservation<'_, Schema>,
        ) -> Output,
    ) -> Option<Output>
    where
        Schema: ApplicationSchema,
    {
        exact_observation::with_exact_observation(self, runtime, observe)
    }

    pub(in crate::domain_computation::authorization) fn retain_observed_support(
        &mut self,
        supporting: WorthQueryRetainedCapabilitySupport,
    ) -> Result<(), ()> {
        self.authorization.retain_supporting(supporting)?;
        self.graph_work.record_decision_facts(1);
        Ok(())
    }

    pub(in crate::domain_computation::authorization) fn resolved_context_entity(
        &self,
        slot: &worth_query_declaration::facade::application_capability::ApplicationCapabilityContextEntitySlotBinding,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        self.resolved.context_entity(slot)
    }

    pub(super) fn resource_entity_kind(&self) -> worth_relational::facade::identity::KindId {
        self.resolved.resource_entity_kind()
    }

    pub(super) fn resource_entity_name(&self) -> &str {
        self.resolved.resource_entity_name()
    }
}
