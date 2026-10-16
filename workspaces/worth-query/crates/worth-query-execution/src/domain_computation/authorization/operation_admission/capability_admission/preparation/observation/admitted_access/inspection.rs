//! Read-only observations of one admitted capability access.

use super::{
    ApplicationCapabilityRequest, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityValidityTimeline, ApplicationSchemaBindingIdentity,
    RelationalAuthorizationObservationCounters, WorthQueryAdmittedApplicationCapabilityAccess,
    WorthQueryCanonicalWorkEvidence, WorthQueryRequestScope,
    WorthQueryRetainedCapabilityAuthorization,
};

impl<Schema, Capability, Operation, Input>
    WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    pub(in crate::domain_computation) fn read_product(
        &self,
    ) -> Option<&crate::basis::WorthQueryProductBranchLease> {
        self.graph_work.mutation_product()
    }
    pub(in crate::domain_computation) fn read_security_basis(
        &self,
    ) -> Option<&worth_relational::facade::branch::AdmittedRelationalBranchBasis> {
        self.read_product()
            .map(|product| product.relational_basis())
    }
    pub(in crate::domain_computation::authorization) fn read_authorization(
        &self,
    ) -> &WorthQueryRetainedCapabilityAuthorization {
        &self.authorization
    }
    pub(in crate::domain_computation) fn validate_workflow_read_authority(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        runtime.validate_capability_read_access(execution, self)
    }

    pub(in crate::domain_computation) fn graph_work_session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.graph_work.identity()
    }

    #[cfg(test)]
    pub(in crate::domain_computation) fn graph_work_managed_run_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkManagedRunIdentity {
        self.graph_work.managed_run_identity()
    }

    pub(in crate::domain_computation) fn graph_work_branch(
        &self,
    ) -> &worth_relational::facade::history::BranchId {
        self.graph_work.branch().relational()
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub(in crate::domain_computation) fn capability_name(&self) -> &str {
        &self.capability
    }

    pub(in crate::domain_computation) fn capability_type(&self) -> &str {
        &self.capability_type
    }

    pub(in crate::domain_computation) const fn runtime_authority(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    pub(in crate::domain_computation) const fn binding_identity(
        &self,
    ) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub(in crate::domain_computation) const fn principal_entity_id(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.principal_entity_id
    }

    pub(in crate::domain_computation) fn resource_entity_id(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.resolved.resource_entity_id()
    }

    pub(in crate::domain_computation) fn disclosure_value(
        &self,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        self.projection.field_value()
    }

    pub(in crate::domain_computation) const fn authentication_valid_until(
        &self,
    ) -> std::time::Instant {
        self.authentication_valid_until
    }

    pub(in crate::domain_computation) const fn request_scope(&self) -> &WorthQueryRequestScope {
        &self.request_scope
    }

    pub(in crate::domain_computation) fn into_query_authorization(
        self,
    ) -> WorthQueryRetainedCapabilityAuthorization {
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

    pub(in crate::domain_computation::authorization) fn installed_capability_identity(
        &self,
    ) -> [u8; 32] {
        self.authorization.installed_capability_identity()
    }

    pub const fn capability_time_timeline(&self) -> ApplicationCapabilityValidityTimeline {
        self.authorization.timeline()
    }

    pub const fn capability_time_sample(&self) -> &worth_foundational::facade::AspectValue {
        self.authorization.sampled_value()
    }

    pub fn authorization_decision_fact_count(&self) -> usize {
        self.authorization.exact_fact_count()
    }

    pub fn relational_counters(&self) -> RelationalAuthorizationObservationCounters {
        self.authorization.relational_counters()
    }

    pub fn signal_dependency_count(&self) -> usize {
        self.authorization.signal_dependency_count()
    }

    pub const fn admission_canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }
}
