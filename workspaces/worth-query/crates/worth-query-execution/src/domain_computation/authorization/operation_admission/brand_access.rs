//! Inline brand forwarders for an admission's accessors and revalidation.

use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryCanonicalWorkPhases,
    WorthQueryCompiledApplicationOperationContracts,
};
use worth_relational::facade::authorization::RelationalAuthorizationObservationCounters;

use super::access::WorthQueryWorkflowApprovalAuthority;
use {super::WorthQueryAdmittedApplicationOperation, crate::domain_computation::vocabulary::authorization::operation_admission::WorthQueryOperationAdmissionIdentity};
use crate::domain_computation::authorization::{
    WorkflowApprovalRequestEncodingDenial, WorthQueryAdmissionLapse,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationScopeBinding,
    WorthQueryRetainedAuthorizationDecisionFacts,
};
use crate::domain_computation::vocabulary::primary_graph::encoding::FundedActorText;
use crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions;

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[inline]
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        self.core.binding_identity()
    }

    #[inline]
    pub(in crate::domain_computation) const fn runtime_authority(
        &self,
    ) -> crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity {
        self.core.runtime_authority()
    }

    #[inline]
    pub const fn admission_identity(&self) -> WorthQueryOperationAdmissionIdentity {
        self.core.admission_identity()
    }

    #[inline]
    pub const fn scope_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.core.scope_entity_id()
    }

    #[inline]
    pub(in crate::domain_computation) const fn scope_entity_kind(
        &self,
    ) -> worth_relational::facade::identity::KindId {
        self.core.scope_entity_kind()
    }

    #[inline]
    pub(in crate::domain_computation) fn scope_entity_name(&self) -> &str {
        self.core.scope_entity_name()
    }

    #[inline]
    pub fn operation(&self) -> &str {
        self.core.operation()
    }

    #[inline]
    pub fn allowed_graph_contract(&self) -> &WorthQueryCompiledApplicationOperationContracts {
        self.core.allowed_graph_contract()
    }

    #[inline]
    pub fn retain_graph_contract(&self) -> Arc<WorthQueryCompiledApplicationOperationContracts> {
        self.core.retain_graph_contract()
    }

    #[inline]
    pub const fn mutation_preconditions(&self) -> &WorthQueryBoundMutationPreconditions {
        self.core.mutation_preconditions()
    }

    #[inline]
    pub const fn canonical_work(&self) -> WorthQueryCanonicalWorkPhases {
        self.core.canonical_work()
    }

    #[inline]
    pub fn authorization_requirement_count(&self) -> usize {
        self.core.authorization_requirement_count()
    }

    #[inline]
    pub fn authorization_decision_fact_count(&self) -> usize {
        self.core.authorization_decision_fact_count()
    }

    #[inline]
    pub(in crate::domain_computation) const fn has_elevation_lifecycle_binding(&self) -> bool {
        self.core.has_elevation_lifecycle_binding()
    }

    #[inline]
    pub fn installed_capability_authority_identity(&self) -> Option<&str> {
        self.core.installed_capability_authority_identity()
    }

    #[inline]
    pub(in crate::domain_computation) fn workflow_subject(
        &self,
        selector: &worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        self.core.workflow_subject(selector)
    }

    #[inline]
    pub(in crate::domain_computation) fn installed_capability_identity(&self) -> Option<[u8; 32]> {
        self.core.installed_capability_identity()
    }

    #[inline]
    pub fn capability_time_timeline(
        &self,
    ) -> Option<
        worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline,
    >{
        self.core.capability_time_timeline()
    }

    #[inline]
    pub fn capability_time_sample(&self) -> Option<&worth_foundational::facade::AspectValue> {
        self.core.capability_time_sample()
    }

    #[inline]
    pub fn workflow_approval_authority(&self) -> Option<WorthQueryWorkflowApprovalAuthority> {
        self.core.workflow_approval_authority()
    }

    #[inline]
    pub fn relational_counters(&self) -> RelationalAuthorizationObservationCounters {
        self.core.relational_counters()
    }

    #[inline]
    pub fn signal_dependency_count(&self) -> usize {
        self.core.signal_dependency_count()
    }

    #[inline]
    pub(in crate::domain_computation) fn authorization_mut(
        &mut self,
    ) -> Option<&mut WorthQueryRetainedAuthorizationDecisionFacts> {
        self.core.authorization_mut()
    }

    #[inline]
    pub(in crate::domain_computation) fn authorization(
        &self,
    ) -> Option<&WorthQueryRetainedAuthorizationDecisionFacts> {
        self.core.authorization()
    }

    #[inline]
    pub(in crate::domain_computation) fn take_authorization_dependencies(
        &mut self,
        bridge: &worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryProviderCommitAuthorization,
        WorthQueryOperationAuthorizationDenial,
    > {
        self.core.take_authorization_dependencies(bridge)
    }

    /// Descriptive fingerprint of the exact installed operation authority
    /// retained by this proof. The string does not itself grant authority.
    #[inline]
    pub fn installed_operation_fingerprint(&self) -> &str {
        self.core.installed_operation_fingerprint()
    }

    /// Exact installed-operation identity bytes retained by this admission.
    ///
    /// Carried onto commit receipts (R8.62 / C1). Descriptive fingerprints are
    /// not a substitute for this identity.
    #[inline]
    pub const fn installed_operation_identity_bytes(&self) -> &[u8; 32] {
        self.core.installed_operation_identity_bytes()
    }

    #[inline]
    pub(in crate::domain_computation) fn retain_installed_operation_fingerprint(&self) -> Arc<str> {
        self.core.retain_installed_operation_fingerprint()
    }

    #[inline]
    pub(in crate::domain_computation) fn retain_resource_binding_identity(&self) -> Arc<str> {
        self.core.retain_resource_binding_identity()
    }

    /// Revalidates the time and cancellation authority carried from the exact
    /// authentication request that minted this operation admission.
    ///
    /// Later governed phases must call this rather than accepting a detached
    /// timestamp or caller assertion.
    #[inline]
    pub fn validate_current_authority(&self) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        self.core.validate_current_authority()
    }

    #[inline]
    pub(in crate::domain_computation) fn current_authority_lapse(
        &self,
    ) -> Option<WorthQueryAdmissionLapse> {
        self.core.current_authority_lapse()
    }

    #[inline]
    pub(in crate::domain_computation) fn retain_admitted_request(
        &self,
    ) -> crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness
    {
        self.core.retain_admitted_request()
    }

    #[inline]
    pub(in crate::domain_computation) fn publication_request(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        self.core.publication_request()
    }

    #[inline]
    pub(in crate::domain_computation) fn bind_source_facts(
        &mut self,
        facts: Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
        context: Option<crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext>,
    ) {
        self.core.bind_source_facts(facts, context)
    }

    #[inline]
    pub(in crate::domain_computation) fn query_context(&self) -> Option<&crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext>{
        self.core.query_context()
    }

    #[inline]
    pub(in crate::domain_computation) fn bind_source_partition(&mut self, identity: [u8; 32]) {
        self.core.bind_source_partition(identity)
    }

    #[inline]
    pub(in crate::domain_computation) fn bind_source_expectation_kind(
        &mut self,
        kind: crate::domain_computation::vocabulary::primary_graph::observed_source::WorthQuerySourceExpectationKind,
    ) {
        self.core.bind_source_expectation_kind(kind)
    }

    #[inline]
    pub(in crate::domain_computation) const fn source_expectation_kind(
        &self,
    ) -> Option<crate::domain_computation::vocabulary::primary_graph::observed_source::WorthQuerySourceExpectationKind> {
        self.core.source_expectation_kind()
    }

    #[inline]
    pub(in crate::domain_computation) const fn source_partition_identity(
        &self,
    ) -> Option<[u8; 32]> {
        self.core.source_partition_identity()
    }

    #[inline]
    pub(in crate::domain_computation) fn source_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact] {
        self.core.source_facts()
    }

    #[inline]
    pub(in crate::domain_computation) fn take_source_facts(
        &mut self,
    ) -> Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact> {
        self.core.take_source_facts()
    }

    /// Stable identity of the authenticated runtime, installed operation,
    /// principal, and typed scope. It intentionally excludes snapshot identity
    /// so an equivalent authorized retry can retain one idempotency intent.
    #[inline]
    pub const fn operation_scope_binding(&self) -> &WorthQueryOperationScopeBinding {
        self.core.operation_scope_binding()
    }

    #[inline]
    pub(in crate::domain_computation) fn signed_creation_principal(
        &self,
    ) -> Option<
        &std::sync::Arc<
            worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity,
        >,
    > {
        self.core.signed_creation_principal()
    }

    #[inline]
    pub(in crate::domain_computation) fn workflow_approval_signing_principal(
        &self,
    ) -> Option<
        &std::sync::Arc<
            worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity,
        >,
    > {
        self.core.workflow_approval_signing_principal()
    }

    #[inline]
    pub(in crate::domain_computation) fn workflow_approval_request_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        self.core.workflow_approval_request_snapshot(execution)
    }

    #[inline]
    pub(in crate::domain_computation) fn workflow_approval_principal_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        self.core.workflow_approval_principal_snapshot(execution)
    }

    #[inline]
    pub(in crate::domain_computation) fn workflow_approval_lineage_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        self.core.workflow_approval_lineage_snapshot(execution)
    }

    #[inline]
    pub(in crate::domain_computation) fn workflow_approval_support_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        self.core.workflow_approval_support_snapshot(execution)
    }

    #[inline]
    pub(in crate::domain_computation) fn workflow_approval_dependencies_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        self.core.workflow_approval_dependencies_snapshot(execution)
    }
}

// Mounted beside its path-mounted module, whose directory it shares with the parent.
#[path = "brand_access_court_doors_for_test.rs"]
mod court_doors_for_test;
