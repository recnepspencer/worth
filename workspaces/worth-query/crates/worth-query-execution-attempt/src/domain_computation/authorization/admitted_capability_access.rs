//! Pure admitted application capability access state.
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Instant;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityRequest, ApplicationCapabilityRequestProjection,
};
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryCanonicalWorkEvidence,
};

use crate::domain_computation::authorization::operation_progression::request_resolution::WorthQueryResolvedCapabilityRequest;
use crate::domain_computation::authorization::{WorthQueryRetainedCapabilityAuthorization, WorthQueryRetainedCapabilitySupport};

pub struct WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    pub authentication: Arc<dyn std::any::Any + Send + Sync>,
    pub runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    pub home_owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    pub binding_identity: ApplicationSchemaBindingIdentity,
    pub capability: Arc<str>,
    pub capability_type: Arc<str>,
    pub operation: Arc<str>,
    pub principal_entity_id: worth_relational::facade::identity::EntityId,
    pub input: worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
    pub governed_input_identity: Option<[u8; 32]>,
    pub projection: ApplicationCapabilityRequestProjection<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Context,
    >,
    pub resolved: WorthQueryResolvedCapabilityRequest<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    >,
    pub authentication_valid_until: Instant,
    pub request_scope: WorthQueryRequestScope,
    pub canonical_work: WorthQueryCanonicalWorkEvidence,
    pub authorization: WorthQueryRetainedCapabilityAuthorization,
    pub operation_admission_identity: crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
    pub graph_work_session:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    pub graph_work_managed_run:
        crate::domain_computation::provider_session::WorthQueryGraphWorkManagedRunIdentity,
    pub graph_work_branch: worth_relational::facade::history::BranchId,
    pub graph_work_capability: Option<[u8; 32]>,
    pub graph_work_decision_facts: usize,
    pub product: crate::basis::WorthQueryProductBranch,
    pub security_basis: worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
    pub _marker: PhantomData<fn() -> (Capability, Operation)>,
}

impl<Schema, Capability, Operation, Input>
    WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    pub fn capability_input(&self) -> &Input {
        self.input.as_ref()
    }

    pub fn retain_observed_support(
        &mut self,
        supporting: WorthQueryRetainedCapabilitySupport,
    ) -> Result<(), ()> {
        self.authorization.retain_supporting(supporting)?;
        self.graph_work_decision_facts = self.graph_work_decision_facts.saturating_add(1);
        Ok(())
    }

    pub fn resolved_context_entity(
        &self,
        slot: &worth_query_declaration::facade::application_capability::ApplicationCapabilityContextEntitySlotBinding,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        self.resolved.context_entity(slot)
    }

    pub fn resource_entity_kind(&self) -> worth_relational::facade::identity::KindId {
        self.resolved.resource_entity_kind()
    }

    pub fn resource_entity_name(&self) -> &str {
        self.resolved.resource_entity_name()
    }
    pub const fn graph_work_session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.graph_work_session
    }


    pub fn record_request_identity_work(
        &mut self,
        _: &dyn worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccessPort,
        work: worth_query_declaration::facade::application_operation::ApplicationCanonicalWork,
    ) {
        self.record_request_identity_work_inner(work);
    }

    pub fn record_request_identity_work_inner(
        &mut self,
        work: worth_query_declaration::facade::application_operation::ApplicationCanonicalWork,
    ) {
        self.canonical_work = self
            .canonical_work
            .combine(WorthQueryCanonicalWorkEvidence::streamed_identities(work));
    }

    pub fn read_security_basis_identity(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity {
        &self.security_basis
    }

    pub fn read_authorization(&self) -> &WorthQueryRetainedCapabilityAuthorization {
        &self.authorization
    }

    pub fn graph_work_managed_run_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkManagedRunIdentity {
        self.graph_work_managed_run
    }

    pub fn graph_work_branch(&self) -> &worth_relational::facade::history::BranchId {
        &self.graph_work_branch
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn capability_name(&self) -> &str {
        &self.capability
    }

    pub fn capability_type(&self) -> &str {
        &self.capability_type
    }

    pub fn runtime_authority_for(
        &self,
        receiver: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        worth_proof::HomeIncarnationDenial,
    > {
        receiver.admit_incarnation(self.home_owner)?;
        Ok(self.runtime_authority)
    }

    pub const fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub const fn principal_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.principal_entity_id
    }

    pub fn resource_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.resolved.resource_entity_id()
    }

    pub fn disclosure_value(&self) -> Option<&worth_foundational::facade::AspectValue> {
        self.projection.field_value()
    }

    pub const fn authentication_valid_until(&self) -> std::time::Instant {
        self.authentication_valid_until
    }

    pub const fn request_scope(&self) -> &WorthQueryRequestScope {
        &self.request_scope
    }

    pub fn into_query_authorization(self) -> WorthQueryRetainedCapabilityAuthorization {
        self.authorization
    }

    pub const fn projected_request(
        &self,
    ) -> &ApplicationCapabilityRequestProjection<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Context,
    > {
        &self.projection
    }

    pub fn installed_capability_authority_identity(&self) -> &str {
        self.authorization.capability_authority_identity()
    }

    pub fn installed_capability_identity(&self) -> [u8; 32] {
        self.authorization.installed_capability_identity()
    }

    pub const fn capability_time_timeline(
        &self,
    ) -> worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline
    {
        self.authorization.timeline()
    }

    pub const fn capability_time_sample(&self) -> &worth_foundational::facade::AspectValue {
        self.authorization.sampled_value()
    }

    pub fn authorization_decision_fact_count(&self) -> usize {
        self.authorization.exact_fact_count()
    }

    pub fn relational_counters(
        &self,
    ) -> worth_relational::facade::authorization::RelationalAuthorizationObservationCounters {
        self.authorization.relational_counters()
    }

    pub fn signal_dependency_count(&self) -> usize {
        self.authorization.signal_dependency_count()
    }

    pub const fn admission_canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }
}

#[path = "admitted_capability_access_court_doors_for_test.rs"]
pub mod court_doors_for_test;
