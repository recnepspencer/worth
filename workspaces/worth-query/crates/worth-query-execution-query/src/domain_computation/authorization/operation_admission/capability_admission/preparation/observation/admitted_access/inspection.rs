//! Read-only observations over an admitted capability access proof.
use super::*;

impl<Schema, Capability, Operation, Input>
    WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    pub(crate) fn capability_input(&self) -> &Input {
        self.input.as_ref()
    }

    pub(crate) fn retain_observed_support(
        &mut self,
        supporting: WorthQueryRetainedCapabilitySupport,
    ) -> Result<(), ()> {
        self.authorization.retain_supporting(supporting)?;
        self.graph_work_decision_facts = self.graph_work_decision_facts.saturating_add(1);
        Ok(())
    }

    pub(crate) fn resolved_context_entity(
        &self,
        slot: &worth_query_declaration::facade::application_capability::ApplicationCapabilityContextEntitySlotBinding,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        self.resolved.context_entity(slot)
    }

    pub(crate) fn resource_entity_kind(&self) -> worth_relational::facade::identity::KindId {
        self.resolved.resource_entity_kind()
    }

    pub(crate) fn resource_entity_name(&self) -> &str {
        self.resolved.resource_entity_name()
    }

    pub(crate) const fn graph_work_session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.graph_work_session
    }

    pub fn record_request_identity_work(
        &mut self,
        _: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        work: worth_query_declaration::facade::application_operation::ApplicationCanonicalWork,
    ) {
        self.record_request_identity_work_inner(work);
    }

    pub(crate) fn record_request_identity_work_inner(
        &mut self,
        work: worth_query_declaration::facade::application_operation::ApplicationCanonicalWork,
    ) {
        self.canonical_work = self
            .canonical_work
            .combine(WorthQueryCanonicalWorkEvidence::streamed_identities(work));
    }

    pub(crate) fn read_security_basis_identity(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity {
        &self.security_basis
    }

    pub(crate) fn read_authorization(&self) -> &WorthQueryRetainedCapabilityAuthorization {
        &self.authorization
    }

    pub(crate) fn graph_work_managed_run_identity(
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

    pub(crate) fn capability_name(&self) -> &str {
        &self.capability
    }

    pub(crate) fn capability_type(&self) -> &str {
        &self.capability_type
    }

    pub(crate) fn runtime_authority_for(
        &self,
        receiver: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        worth_proof::HomeIncarnationDenial,
    > {
        receiver.admit_incarnation(self.home_owner)?;
        Ok(self.runtime_authority)
    }

    pub(crate) const fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub(crate) const fn principal_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.principal_entity_id
    }

    pub(crate) fn resource_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.resolved.resource_entity_id()
    }

    pub(crate) fn disclosure_value(&self) -> Option<&worth_foundational::facade::AspectValue> {
        self.projection.field_value()
    }

    pub(crate) const fn authentication_valid_until(&self) -> std::time::Instant {
        self.authentication_valid_until
    }

    pub(crate) const fn request_scope(&self) -> &WorthQueryRequestScope {
        &self.request_scope
    }

    pub(crate) fn into_query_authorization(self) -> WorthQueryRetainedCapabilityAuthorization {
        self.authorization
    }

    pub(crate) const fn projected_request(
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

    pub(crate) fn installed_capability_identity(&self) -> [u8; 32] {
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

#[path = "inspection/court_doors_for_test.rs"]
pub mod court_doors_for_test;

impl<Schema, Capability, Operation, Input> crate::ports::AdmittedCapabilityAccessInspectionPort
    for WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Input: worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
        Schema,
        Capability,
    >,
{
    fn runtime_authority_for(
        &self,
        receiver: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        worth_proof::HomeIncarnationDenial,
    > {
        Self::runtime_authority_for(self, receiver)
    }

    fn principal_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        Self::principal_entity_id(self)
    }

    fn resource_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        Self::resource_entity_id(self)
    }
}
