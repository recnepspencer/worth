use serde::{Deserialize, Serialize};

use super::declaration::ResourcePayloadContractId;
use super::policy::{
    ResourceCancellationDecisionPlan, ResourceCancellationPolicyDeclaration,
    ResourceDiagnosticsDecisionPlan, ResourceDiagnosticsPolicyDeclaration,
    ResourceLifecyclePolicyDeclaration, ResourceObservationDecisionPlan,
    ResourceObservationPolicyDeclaration, ResourceOutputContinuityDecisionPlan,
    ResourceOutputContinuityPolicyDeclaration, ResourceReplayDecisionPlan,
    ResourceReplayPolicyDeclaration, ResourceRetentionDecisionPlan,
    ResourceRetentionPolicyDeclaration, ResourceRetryDecisionPlan, ResourceRetryPolicyDeclaration,
    ResourceRevalidationDecisionPlan, ResourceRevalidationPolicyDeclaration,
    ResourceStaleAfterDecisionPlan, ResourceStaleAfterPolicyDeclaration,
    ResourceSupersessionDecisionPlan, ResourceSupersessionPolicyDeclaration,
    ResourceTimeoutDecisionPlan, ResourceTimeoutPolicyDeclaration,
};
use super::policy_registry::{
    LoweredResourcePolicyBundle, ResourcePolicyResolutionError, ValidatedResourcePolicyDeclaration,
};
use super::request::ResourceNodeId;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResourceDescriptorId(u64);

impl ResourceDescriptorId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn get(self) -> u64 {
        self.0
    }

    pub(crate) fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResourceDescriptorVersion(u64);

impl ResourceDescriptorVersion {
    pub const INITIAL: Self = Self(0);

    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResourcePayloadContractDigest(String);

impl ResourcePayloadContractDigest {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub(crate) fn from_contract(
        id: ResourcePayloadContractId,
        max_payload_bytes: Option<u64>,
    ) -> Self {
        Self(format!(
            "payload-contract:{}:{}",
            id.get(),
            max_payload_bytes
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unbounded".to_owned())
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoweredResourceDescriptor {
    descriptor_id: ResourceDescriptorId,
    descriptor_version: ResourceDescriptorVersion,
    node: ResourceNodeId,
    lifecycle_policy: ResourceLifecyclePolicyDeclaration,
    retry_policy: ResourceRetryPolicyDeclaration,
    timeout_policy: ResourceTimeoutPolicyDeclaration,
    cancellation_policy: ResourceCancellationPolicyDeclaration,
    stale_after_policy: ResourceStaleAfterPolicyDeclaration,
    supersession_policy: ResourceSupersessionPolicyDeclaration,
    revalidation_policy: ResourceRevalidationPolicyDeclaration,
    observation_policy: ResourceObservationPolicyDeclaration,
    output_continuity_policy: ResourceOutputContinuityPolicyDeclaration,
    retention_policy: ResourceRetentionPolicyDeclaration,
    diagnostics_policy: ResourceDiagnosticsPolicyDeclaration,
    replay_policy: ResourceReplayPolicyDeclaration,
    lowered_policy_bundle: LoweredResourcePolicyBundle,
    retry_decision_plan: ResourceRetryDecisionPlan,
    timeout_decision_plan: ResourceTimeoutDecisionPlan,
    cancellation_decision_plan: ResourceCancellationDecisionPlan,
    supersession_decision_plan: ResourceSupersessionDecisionPlan,
    revalidation_decision_plan: ResourceRevalidationDecisionPlan,
    observation_decision_plan: ResourceObservationDecisionPlan,
    stale_after_decision_plan: ResourceStaleAfterDecisionPlan,
    output_continuity_decision_plan: ResourceOutputContinuityDecisionPlan,
    retention_decision_plan: ResourceRetentionDecisionPlan,
    diagnostics_decision_plan: ResourceDiagnosticsDecisionPlan,
    replay_decision_plan: ResourceReplayDecisionPlan,
    payload_contract_digest: ResourcePayloadContractDigest,
    max_payload_bytes: Option<u64>,
}

impl LoweredResourceDescriptor {
    pub(crate) fn default_timeout_decision_plan() -> ResourceTimeoutDecisionPlan {
        ResourceTimeoutDecisionPlan::disabled_builtin_default()
    }

    pub(crate) fn from_validated_policy_declaration(
        descriptor_id: ResourceDescriptorId,
        descriptor_version: ResourceDescriptorVersion,
        validated: &ValidatedResourcePolicyDeclaration,
        lowered_policy_bundle: LoweredResourcePolicyBundle,
    ) -> Result<Self, ResourcePolicyResolutionError> {
        let declaration = validated.declaration();
        let retry_decision_plan =
            ResourceRetryDecisionPlan::lower(declaration, lowered_policy_bundle.retry())?;
        let timeout_decision_plan = ResourceTimeoutDecisionPlan::lower(
            declaration.timeout_policy(),
            lowered_policy_bundle.timeout(),
        )?;
        let cancellation_decision_plan = ResourceCancellationDecisionPlan::lower(
            declaration.node(),
            declaration.cancellation_policy(),
            lowered_policy_bundle.cancellation(),
            declaration.cancellation_grace_period(),
            declaration.declared_dependent_cancellation_nodes(),
        )?;
        let supersession_decision_plan = ResourceSupersessionDecisionPlan::lower(
            declaration.supersession_policy(),
            lowered_policy_bundle.supersession(),
        )?;
        let revalidation_decision_plan = ResourceRevalidationDecisionPlan::lower(
            declaration.revalidation_policy(),
            lowered_policy_bundle.revalidation(),
        )?;
        let observation_decision_plan = ResourceObservationDecisionPlan::lower(
            declaration.observation_policy(),
            lowered_policy_bundle.observation(),
        )?;
        let stale_after_decision_plan = ResourceStaleAfterDecisionPlan::lower(
            declaration.stale_after_policy(),
            lowered_policy_bundle.stale_after(),
        )?;
        let output_continuity_decision_plan = ResourceOutputContinuityDecisionPlan::lower(
            declaration.output_continuity_policy(),
            lowered_policy_bundle.output_continuity(),
        )?;
        let retention_decision_plan = ResourceRetentionDecisionPlan::lower(
            declaration.retention_policy(),
            lowered_policy_bundle.retention(),
        )?;
        let diagnostics_decision_plan = ResourceDiagnosticsDecisionPlan::lower(
            declaration.diagnostics_policy(),
            lowered_policy_bundle.diagnostics(),
        )?;
        let replay_decision_plan = ResourceReplayDecisionPlan::lower(
            declaration.replay_policy(),
            lowered_policy_bundle.replay(),
        )?;
        Ok(Self {
            descriptor_id,
            descriptor_version,
            node: declaration.node(),
            lifecycle_policy: declaration.lifecycle_policy(),
            retry_policy: declaration.retry_policy().clone(),
            timeout_policy: declaration.timeout_policy().clone(),
            cancellation_policy: declaration.cancellation_policy().clone(),
            stale_after_policy: declaration.stale_after_policy().clone(),
            supersession_policy: declaration.supersession_policy().clone(),
            revalidation_policy: declaration.revalidation_policy().clone(),
            observation_policy: declaration.observation_policy().clone(),
            output_continuity_policy: declaration.output_continuity_policy().clone(),
            retention_policy: declaration.retention_policy().clone(),
            diagnostics_policy: declaration.diagnostics_policy().clone(),
            replay_policy: declaration.replay_policy().clone(),
            lowered_policy_bundle,
            retry_decision_plan,
            timeout_decision_plan,
            cancellation_decision_plan,
            supersession_decision_plan,
            revalidation_decision_plan,
            observation_decision_plan,
            stale_after_decision_plan,
            output_continuity_decision_plan,
            retention_decision_plan,
            diagnostics_decision_plan,
            replay_decision_plan,
            payload_contract_digest: ResourcePayloadContractDigest::from_contract(
                declaration.payload_contract().id(),
                declaration.payload_contract().max_payload_bytes(),
            ),
            max_payload_bytes: declaration.payload_contract().max_payload_bytes(),
        })
    }

    pub fn descriptor_id(&self) -> ResourceDescriptorId {
        self.descriptor_id
    }

    pub fn descriptor_version(&self) -> ResourceDescriptorVersion {
        self.descriptor_version
    }

    pub fn node(&self) -> ResourceNodeId {
        self.node
    }

    pub fn payload_contract_digest(&self) -> &ResourcePayloadContractDigest {
        &self.payload_contract_digest
    }

    pub fn lowered_policy_bundle(&self) -> &LoweredResourcePolicyBundle {
        &self.lowered_policy_bundle
    }

    pub fn resolved_policy_bundle(&self) -> &LoweredResourcePolicyBundle {
        &self.lowered_policy_bundle
    }

    pub fn max_payload_bytes(&self) -> Option<u64> {
        self.max_payload_bytes
    }

    pub fn timeout_policy(&self) -> &ResourceTimeoutPolicyDeclaration {
        &self.timeout_policy
    }

    pub fn retry_policy(&self) -> &ResourceRetryPolicyDeclaration {
        &self.retry_policy
    }

    pub fn retry_decision_plan(&self) -> &ResourceRetryDecisionPlan {
        &self.retry_decision_plan
    }

    pub fn timeout_decision_plan(&self) -> &ResourceTimeoutDecisionPlan {
        &self.timeout_decision_plan
    }

    pub fn cancellation_decision_plan(&self) -> &ResourceCancellationDecisionPlan {
        &self.cancellation_decision_plan
    }

    pub fn supersession_decision_plan(&self) -> &ResourceSupersessionDecisionPlan {
        &self.supersession_decision_plan
    }

    pub fn revalidation_decision_plan(&self) -> &ResourceRevalidationDecisionPlan {
        &self.revalidation_decision_plan
    }

    pub fn observation_decision_plan(&self) -> &ResourceObservationDecisionPlan {
        &self.observation_decision_plan
    }

    pub fn stale_after_decision_plan(&self) -> &ResourceStaleAfterDecisionPlan {
        &self.stale_after_decision_plan
    }

    pub fn output_continuity_decision_plan(&self) -> &ResourceOutputContinuityDecisionPlan {
        &self.output_continuity_decision_plan
    }

    pub fn retention_decision_plan(&self) -> &ResourceRetentionDecisionPlan {
        &self.retention_decision_plan
    }

    pub(crate) fn retention_policy(&self) -> &ResourceRetentionPolicyDeclaration {
        &self.retention_policy
    }

    pub fn diagnostics_decision_plan(&self) -> &ResourceDiagnosticsDecisionPlan {
        &self.diagnostics_decision_plan
    }

    pub fn replay_decision_plan(&self) -> &ResourceReplayDecisionPlan {
        &self.replay_decision_plan
    }
}
