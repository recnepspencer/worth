//! Accessors and current-authority revalidation for admitted operations.

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryCanonicalWorkPhases,
    WorthQueryCompiledApplicationOperationContracts,
};
use worth_relational::facade::authorization::RelationalAuthorizationObservationCounters;

use super::WorthQueryAdmittedOperation;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryOperationScopeBinding, WorthQueryRetainedAuthorizationDecisionFacts,
    WorthQueryRetainedCapabilityAuthorization,
};
use crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions;

use super::WorthQueryOperationAdmissionIdentity;

#[path = "access/currentness.rs"]
mod currentness;
#[path = "access/workflow_approval_snapshot.rs"]
mod workflow_approval_snapshot;

pub struct WorthQueryWorkflowApprovalAuthority {
    pub principal: worth_relational::facade::identity::EntityId,
    pub grant: worth_relational::facade::identity::EntityId,
    pub decision_identity: [u8; 32],
    pub capability_authority_identity: String,
    pub action: worth_foundational::facade::AspectValue,
    pub purpose: worth_foundational::facade::AspectValue,
    pub timeline:
        worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline,
    pub sampled_value: worth_foundational::facade::AspectValue,
    pub expiry: worth_foundational::facade::AspectValue,
}

impl WorthQueryAdmittedOperation {
    pub(in crate::domain_computation) fn binding_identity(
        &self,
    ) -> &ApplicationSchemaBindingIdentity {
        &self.data.binding_identity
    }

    pub(in crate::domain_computation) fn belongs_to(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.data.runtime_authority == runtime_authority
            && &self.data.binding_identity == binding_identity
    }

    pub(in crate::domain_computation) const fn runtime_authority(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.data.runtime_authority
    }

    pub(in crate::domain_computation) const fn admission_identity(
        &self,
    ) -> WorthQueryOperationAdmissionIdentity {
        self.data.admission_identity
    }

    pub(in crate::domain_computation) const fn scope_entity_id(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.data.scope_entity_id
    }

    pub(in crate::domain_computation) const fn scope_entity_kind(
        &self,
    ) -> worth_relational::facade::identity::KindId {
        self.data.scope_entity_kind
    }

    pub(in crate::domain_computation) fn scope_entity_name(&self) -> &str {
        &self.data.scope_entity_name
    }

    pub(in crate::domain_computation) fn operation(&self) -> &str {
        &self.data.operation
    }

    pub(in crate::domain_computation) fn allowed_graph_contract(
        &self,
    ) -> &WorthQueryCompiledApplicationOperationContracts {
        &self.data.contracts
    }

    /// Retain the actual installed immutable descriptor for checked read
    /// layout reuse. Identity grants no source, session or currentness custody.
    pub(in crate::domain_computation) fn retain_graph_contract(
        &self,
    ) -> Arc<WorthQueryCompiledApplicationOperationContracts> {
        Arc::clone(&self.data.contracts)
    }

    pub(in crate::domain_computation) const fn mutation_preconditions(
        &self,
    ) -> &WorthQueryBoundMutationPreconditions {
        &self.data.mutation_preconditions
    }

    pub(in crate::domain_computation) const fn canonical_work(
        &self,
    ) -> WorthQueryCanonicalWorkPhases {
        self.data.canonical_work
    }

    pub(in crate::domain_computation) fn authorization_requirement_count(&self) -> usize {
        self.data.authorization.as_ref().map_or(
            0,
            WorthQueryRetainedAuthorizationDecisionFacts::policy_count,
        )
    }

    pub(in crate::domain_computation) fn authorization_decision_fact_count(&self) -> usize {
        self.data.authorization.as_ref().map_or(
            0,
            WorthQueryRetainedAuthorizationDecisionFacts::exact_fact_count,
        )
    }

    pub(in crate::domain_computation) const fn has_elevation_lifecycle_binding(&self) -> bool {
        self.data.authorization_basis.is_elevation_lifecycle()
    }

    pub(in crate::domain_computation) fn installed_capability_authority_identity(
        &self,
    ) -> Option<&str> {
        self.data
            .authorization
            .as_ref()?
            .capability_authorization()
            .map(WorthQueryRetainedCapabilityAuthorization::capability_authority_identity)
    }

    pub(in crate::domain_computation) fn workflow_subject(
        &self,
        selector: &worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        self.data
            .authorization
            .as_ref()?
            .capability_authorization()?
            .request()
            .workflow_subject(selector)
    }

    pub(in crate::domain_computation) fn installed_capability_identity(&self) -> Option<[u8; 32]> {
        self.data
            .authorization
            .as_ref()?
            .capability_authorization()
            .map(WorthQueryRetainedCapabilityAuthorization::installed_capability_identity)
    }

    pub(in crate::domain_computation) fn capability_time_timeline(
        &self,
    ) -> Option<
        worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline,
    >{
        self.data
            .authorization
            .as_ref()?
            .capability_authorization()
            .map(WorthQueryRetainedCapabilityAuthorization::timeline)
    }

    pub(in crate::domain_computation) fn capability_time_sample(
        &self,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        self.data
            .authorization
            .as_ref()?
            .capability_authorization()
            .map(WorthQueryRetainedCapabilityAuthorization::sampled_value)
    }

    pub(in crate::domain_computation) fn workflow_approval_authority(
        &self,
    ) -> Option<WorthQueryWorkflowApprovalAuthority> {
        let authorization = self
            .data
            .authorization
            .as_ref()?
            .capability_authorization()?;
        Some(WorthQueryWorkflowApprovalAuthority {
            principal: authorization.request().principal(),
            grant: authorization.grant(),
            decision_identity: authorization.decision_identity(),
            capability_authority_identity: authorization.capability_authority_identity().to_owned(),
            action: authorization.request().action().clone(),
            purpose: authorization.request().purpose().clone(),
            timeline: authorization.timeline(),
            sampled_value: authorization.sampled_value().clone(),
            expiry: authorization.expiry().clone(),
        })
    }

    pub(in crate::domain_computation) fn relational_counters(
        &self,
    ) -> RelationalAuthorizationObservationCounters {
        self.data
            .authorization
            .as_ref()
            .map(WorthQueryRetainedAuthorizationDecisionFacts::relational_counters)
            .unwrap_or_default()
    }

    pub(in crate::domain_computation) fn signal_dependency_count(&self) -> usize {
        self.data
            .authorization
            .as_ref()
            .map(WorthQueryRetainedAuthorizationDecisionFacts::signal_dependency_count)
            .unwrap_or(0)
    }

    pub(in crate::domain_computation) fn authorization_mut(
        &mut self,
    ) -> Option<&mut WorthQueryRetainedAuthorizationDecisionFacts> {
        self.data.authorization.as_mut()
    }

    pub(in crate::domain_computation) fn authorization(
        &self,
    ) -> Option<&WorthQueryRetainedAuthorizationDecisionFacts> {
        self.data.authorization.as_ref()
    }

    pub(in crate::domain_computation) fn take_authorization_dependencies(
        &mut self,
        bridge: &worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryProviderCommitAuthorization,
        WorthQueryOperationAuthorizationDenial,
    > {
        let authorization = self.data.authorization.take().ok_or_else(|| {
            WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::InconsistentDecision,
                ),
                &self.data.operation,
            )
        })?;
        if !authorization.bridge_is_retained(bridge) {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::InconsistentDecision,
                ),
                &self.data.operation,
            ));
        }
        Ok(authorization.into_provider_commit_authorization(self.data.admission_identity))
    }

    /// Descriptive fingerprint of the exact installed operation authority
    /// retained by this proof. The string does not itself grant authority.
    pub(in crate::domain_computation) fn installed_operation_fingerprint(&self) -> &str {
        &self.data.operation_authority_identity
    }

    /// Exact installed-operation identity bytes retained by this admission.
    ///
    /// Carried onto commit receipts (R8.62 / C1). Descriptive fingerprints are
    /// not a substitute for this identity.
    pub(in crate::domain_computation) const fn installed_operation_identity_bytes(
        &self,
    ) -> &[u8; 32] {
        &self.data.operation_authority_identity_bytes
    }

    pub(in crate::domain_computation) fn retain_installed_operation_fingerprint(&self) -> Arc<str> {
        Arc::clone(&self.data.operation_authority_identity)
    }

    pub(in crate::domain_computation) fn retain_resource_binding_identity(&self) -> Arc<str> {
        Arc::clone(&self.data.resource_binding_identity)
    }

    pub(in crate::domain_computation) fn bind_source_facts(
        &mut self,
        facts: Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
        context: Option<crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext>,
    ) {
        self.data.source_facts = facts;
        self.data.query_context = context;
    }

    pub(in crate::domain_computation) fn query_context(&self) -> Option<&crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext>{
        self.data.query_context.as_ref()
    }

    pub(in crate::domain_computation) fn bind_source_partition(&mut self, identity: [u8; 32]) {
        self.data.source_partition_identity = Some(identity);
    }
    pub(in crate::domain_computation) fn bind_source_expectation_kind(
        &mut self,
        kind: crate::domain_computation::primary_graph::WorthQuerySourceExpectationKind,
    ) {
        self.data.source_expectation_kind = Some(kind);
    }
    pub(in crate::domain_computation) const fn source_expectation_kind(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQuerySourceExpectationKind> {
        self.data.source_expectation_kind
    }

    pub(in crate::domain_computation) const fn source_partition_identity(
        &self,
    ) -> Option<[u8; 32]> {
        self.data.source_partition_identity
    }

    pub(in crate::domain_computation) fn source_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact] {
        &self.data.source_facts
    }

    pub(in crate::domain_computation) fn take_source_facts(
        &mut self,
    ) -> Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact> {
        std::mem::take(&mut self.data.source_facts)
    }

    /// Stable identity of the authenticated runtime, installed operation,
    /// principal, and typed scope. It intentionally excludes snapshot identity
    /// so an equivalent authorized retry can retain one idempotency intent.
    pub(in crate::domain_computation) const fn operation_scope_binding(
        &self,
    ) -> &WorthQueryOperationScopeBinding {
        &self.data.operation_scope_binding
    }
}
