//! Accessors and current-authority revalidation for admitted operations.

use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryCanonicalWorkPhases,
    WorthQueryCompiledApplicationOperationContracts,
};
use worth_relational::facade::authorization::RelationalAuthorizationObservationCounters;

use super::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryOperationScopeBinding, WorthQueryRetainedAuthorizationDecisionFacts,
    WorthQueryRetainedCapabilityAuthorization,
};
use crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions;

use super::WorthQueryOperationAdmissionIdentity;
use std::time::Instant;

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub(in crate::domain_computation) fn belongs_to(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.runtime_authority == runtime_authority && &self.binding_identity == binding_identity
    }

    pub(in crate::domain_computation) fn validate_projection_authority(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if self.runtime_authority != runtime_authority {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::ForeignRuntime,
                &self.operation,
            ));
        }
        if &self.binding_identity != binding_identity {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::StaleInstalledSchema,
                &self.operation,
            ));
        }
        self.validate_current_authority()
    }

    pub(in crate::domain_computation) const fn runtime_authority(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    pub(in crate::domain_computation) const fn admission_identity(
        &self,
    ) -> WorthQueryOperationAdmissionIdentity {
        self.admission_identity
    }

    pub(in crate::domain_computation) const fn scope_entity_id(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.scope_entity_id
    }

    pub(in crate::domain_computation) const fn scope_entity_kind(
        &self,
    ) -> worth_relational::facade::identity::KindId {
        self.scope_entity_kind
    }

    pub(in crate::domain_computation) fn scope_entity_name(&self) -> &str {
        &self.scope_entity_name
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn allowed_graph_contract(&self) -> &WorthQueryCompiledApplicationOperationContracts {
        &self.contracts
    }

    pub(in crate::domain_computation) const fn mutation_preconditions(
        &self,
    ) -> &WorthQueryBoundMutationPreconditions {
        &self.mutation_preconditions
    }

    pub const fn canonical_work(&self) -> WorthQueryCanonicalWorkPhases {
        self.canonical_work
    }

    pub fn authorization_requirement_count(&self) -> usize {
        self.authorization.as_ref().map_or(
            0,
            WorthQueryRetainedAuthorizationDecisionFacts::policy_count,
        )
    }

    pub fn authorization_decision_fact_count(&self) -> usize {
        self.authorization.as_ref().map_or(
            0,
            WorthQueryRetainedAuthorizationDecisionFacts::exact_fact_count,
        )
    }

    pub const fn capability_input(&self) -> Option<&Input> {
        self.authorization_basis.capability_input()
    }

    pub(in crate::domain_computation) const fn has_elevation_lifecycle_binding(&self) -> bool {
        self.authorization_basis.is_elevation_lifecycle()
    }

    pub fn installed_capability_authority_identity(&self) -> Option<&str> {
        self.authorization
            .as_ref()?
            .capability_authorization()
            .map(WorthQueryRetainedCapabilityAuthorization::capability_authority_identity)
    }

    pub fn capability_time_timeline(
        &self,
    ) -> Option<
        worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline,
    >{
        self.authorization
            .as_ref()?
            .capability_authorization()
            .map(WorthQueryRetainedCapabilityAuthorization::timeline)
    }

    pub fn capability_time_sample(&self) -> Option<&worth_foundational::facade::AspectValue> {
        self.authorization
            .as_ref()?
            .capability_authorization()
            .map(WorthQueryRetainedCapabilityAuthorization::sampled_value)
    }

    pub fn relational_counters(&self) -> RelationalAuthorizationObservationCounters {
        self.authorization
            .as_ref()
            .map(WorthQueryRetainedAuthorizationDecisionFacts::relational_counters)
            .unwrap_or_default()
    }

    pub fn signal_dependency_count(&self) -> usize {
        self.authorization
            .as_ref()
            .map(WorthQueryRetainedAuthorizationDecisionFacts::signal_dependency_count)
            .unwrap_or(0)
    }

    pub(in crate::domain_computation) fn authorization_mut(
        &mut self,
    ) -> Option<&mut WorthQueryRetainedAuthorizationDecisionFacts> {
        self.authorization.as_mut()
    }

    pub(in crate::domain_computation) fn authorization(
        &self,
    ) -> Option<&WorthQueryRetainedAuthorizationDecisionFacts> {
        self.authorization.as_ref()
    }

    pub(in crate::domain_computation) fn take_authorization_dependencies(
        &mut self,
        bridge: &worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryProviderCommitAuthorization,
        WorthQueryOperationAuthorizationDenial,
    > {
        let authorization = self.authorization.take().ok_or_else(|| {
            WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                &self.operation,
            )
        })?;
        if !authorization.bridge_is_retained(bridge) {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                &self.operation,
            ));
        }
        Ok(authorization.into_provider_commit_authorization(self.admission_identity))
    }

    /// Descriptive fingerprint of the exact installed operation authority
    /// retained by this proof. The string does not itself grant authority.
    pub fn installed_operation_fingerprint(&self) -> &str {
        &self.operation_authority_identity
    }

    /// Exact installed-operation identity bytes retained by this admission.
    ///
    /// Carried onto commit receipts (R8.62 / C1). Descriptive fingerprints are
    /// not a substitute for this identity.
    pub const fn installed_operation_identity_bytes(&self) -> &[u8; 32] {
        &self.operation_authority_identity_bytes
    }

    pub(in crate::domain_computation) fn retain_installed_operation_fingerprint(&self) -> Arc<str> {
        Arc::clone(&self.operation_authority_identity)
    }

    pub(in crate::domain_computation) fn retain_resource_binding_identity(&self) -> Arc<str> {
        Arc::clone(&self.resource_binding_identity)
    }

    /// Revalidates the time and cancellation authority carried from the exact
    /// authentication request that minted this operation admission.
    ///
    /// Later governed phases must call this rather than accepting a detached
    /// timestamp or caller assertion.
    pub fn validate_current_authority(&self) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if let Some(interruption) = self.request_scope.interruption() {
            let kind = match interruption {
                WorthQueryRequestInterruption::Cancelled => {
                    WorthQueryOperationAuthorizationDenialKind::Cancelled
                }
                WorthQueryRequestInterruption::DeadlineExceeded => {
                    WorthQueryOperationAuthorizationDenialKind::DeadlineExceeded
                }
            };
            return Err(WorthQueryOperationAuthorizationDenial::new(
                kind,
                &self.operation,
            ));
        }
        if Instant::now() >= self.authentication_valid_until {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::ExpiredAuthentication,
                &self.operation,
            ));
        }
        Ok(())
    }

    pub(in crate::domain_computation) fn publication_request(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        &self.request_scope
    }

    /// Stable identity of the authenticated runtime, installed operation,
    /// principal, and typed scope. It intentionally excludes snapshot identity
    /// so an equivalent authorized retry can retain one idempotency intent.
    pub const fn operation_scope_binding(&self) -> &WorthQueryOperationScopeBinding {
        &self.operation_scope_binding
    }
}
