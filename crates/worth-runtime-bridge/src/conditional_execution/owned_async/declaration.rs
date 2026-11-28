use std::sync::Arc;

use worth_signal::facade::{
    NodeId, ResourceCancellationPolicyDeclaration, ResourceDiagnosticsPolicyDeclaration,
    ResourceLifecyclePolicyDeclaration, ResourceNodeDeclaration, ResourceNodeId,
    ResourceObservationPolicyDeclaration, ResourceOutputContinuityPolicyDeclaration,
    ResourcePayloadContract, ResourcePayloadContractId, ResourceReplayPolicyDeclaration,
    ResourceRetentionPolicyDeclaration, ResourceRetryPolicyDeclaration,
    ResourceRevalidationPolicyDeclaration, ResourceStaleAfterPolicyDeclaration,
    ResourceSupersessionPolicyDeclaration, ResourceTimeoutPolicyDeclaration, TemporalDuration,
};

use crate::facade::{
    BridgeAsyncSourceDeclarationRejection, BridgeAsyncSourceDeclarationRejectionKind,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeOwnedAsyncRequestResponseDeclaration {
    pub(super) identity: Arc<str>,
    pub(super) payload_contract: u64,
    pub(super) max_payload_bytes: u64,
    pub(super) retry_max_attempts: u32,
    pub(super) retry_delay_ticks: u64,
    pub(super) timeout_ticks: u64,
    pub(super) managed_queue_capacity: Option<std::num::NonZeroU64>,
    pub(super) process_local_retention: bool,
    policies: Option<Box<BridgeOwnedAsyncResourcePolicies>>,
}

/// Declaration meaning only; it recreates no installed Signal source or port.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BridgeOwnedAsyncRetention {
    Mandatory,
    ProcessLocal,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BridgeOwnedAsyncDeclarationDescription {
    pub identity: String,
    pub payload_contract: u64,
    pub max_payload_bytes: u64,
    pub retry_max_attempts: u32,
    pub retry_delay_ticks: u64,
    pub timeout_ticks: u64,
    pub managed_queue_capacity: Option<std::num::NonZeroU64>,
    pub retention: BridgeOwnedAsyncRetention,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policies: Option<BridgeOwnedAsyncResourcePolicies>,
}
/// Explicit Signal resource policies for an owned request-response declaration.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct BridgeOwnedAsyncResourcePolicies {
    pub lifecycle: ResourceLifecyclePolicyDeclaration,
    pub retry: ResourceRetryPolicyDeclaration,
    pub timeout: ResourceTimeoutPolicyDeclaration,
    pub cancellation: ResourceCancellationPolicyDeclaration,
    pub stale_after: ResourceStaleAfterPolicyDeclaration,
    pub supersession: ResourceSupersessionPolicyDeclaration,
    pub revalidation: ResourceRevalidationPolicyDeclaration,
    pub observation: ResourceObservationPolicyDeclaration,
    pub output_continuity: ResourceOutputContinuityPolicyDeclaration,
    pub retention: ResourceRetentionPolicyDeclaration,
    pub diagnostics: ResourceDiagnosticsPolicyDeclaration,
    pub replay: ResourceReplayPolicyDeclaration,
}

impl BridgeOwnedAsyncRequestResponseDeclaration {
    pub(super) fn signal_declaration(
        &self,
        node: NodeId,
    ) -> Result<ResourceNodeDeclaration, BridgeAsyncSourceDeclarationRejection> {
        if self.process_local_retention
            && self.policies.as_ref().is_some_and(|policies| {
                policies.retention != ResourceRetentionPolicyDeclaration::TerminalSummariesOnly
            })
        {
            return Err(BridgeAsyncSourceDeclarationRejection::new(
                BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected,
                "process-local retention requires a terminal-only resource policy",
            ));
        }
        let mut resource = ResourceNodeDeclaration::new(
            ResourceNodeId::from_node(node),
            ResourcePayloadContract::new(ResourcePayloadContractId::new(self.payload_contract))
                .with_max_payload_bytes(self.max_payload_bytes),
        );
        if let Some(policies) = &self.policies {
            resource = resource
                .with_lifecycle_policy(policies.lifecycle)
                .with_retry_policy(policies.retry.clone())
                .with_timeout_policy(policies.timeout.clone())
                .with_cancellation_policy(policies.cancellation.clone())
                .with_stale_after_policy(policies.stale_after.clone())
                .with_supersession_policy(policies.supersession.clone())
                .with_revalidation_policy(policies.revalidation.clone())
                .with_observation_policy(policies.observation.clone())
                .with_output_continuity_policy(policies.output_continuity.clone())
                .with_retention_policy(policies.retention.clone())
                .with_diagnostics_policy(policies.diagnostics.clone())
                .with_replay_policy(policies.replay.clone());
        } else {
            resource = resource
                .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleOnly)
                .with_retry_policy(ResourceRetryPolicyDeclaration::FixedDelay {
                    delay: duration(self.retry_delay_ticks)?,
                })
                .with_timeout_policy(ResourceTimeoutPolicyDeclaration::PerAttemptTimeout {
                    timeout: duration(self.timeout_ticks)?,
                });
        }
        if !matches!(
            resource.retry_policy(),
            ResourceRetryPolicyDeclaration::Disabled
        ) {
            resource = resource.with_retry_max_attempts(self.retry_max_attempts);
        }
        if self.process_local_retention {
            resource = resource
                .with_retention_policy(ResourceRetentionPolicyDeclaration::TerminalSummariesOnly);
        }
        Ok(resource)
    }

    /// Reconstruct declaration meaning. Installation still issues fresh Native owners.
    pub fn from_description(description: BridgeOwnedAsyncDeclarationDescription) -> Self {
        Self {
            identity: Arc::from(description.identity),
            policies: description.policies.map(Box::new),
            payload_contract: description.payload_contract,
            max_payload_bytes: description.max_payload_bytes,
            retry_max_attempts: description.retry_max_attempts,
            retry_delay_ticks: description.retry_delay_ticks,
            timeout_ticks: description.timeout_ticks,
            managed_queue_capacity: description.managed_queue_capacity,
            process_local_retention: description.retention
                == BridgeOwnedAsyncRetention::ProcessLocal,
        }
    }
    pub fn description(&self) -> BridgeOwnedAsyncDeclarationDescription {
        BridgeOwnedAsyncDeclarationDescription {
            identity: self.identity.to_string(),
            policies: self.policies.as_deref().cloned(),
            payload_contract: self.payload_contract,
            max_payload_bytes: self.max_payload_bytes,
            retry_max_attempts: self.retry_max_attempts,
            retry_delay_ticks: self.retry_delay_ticks,
            timeout_ticks: self.timeout_ticks,
            managed_queue_capacity: self.managed_queue_capacity,
            retention: if self.process_local_retention {
                BridgeOwnedAsyncRetention::ProcessLocal
            } else {
                BridgeOwnedAsyncRetention::Mandatory
            },
        }
    }
    pub fn new(
        identity: impl Into<Arc<str>>,
        payload_contract: u64,
        max_payload_bytes: u64,
        retry_max_attempts: u32,
        retry_delay_ticks: u64,
        timeout_ticks: u64,
    ) -> Self {
        Self {
            identity: identity.into(),
            payload_contract,
            max_payload_bytes,
            retry_max_attempts,
            retry_delay_ticks,
            timeout_ticks,
            managed_queue_capacity: None,
            process_local_retention: false,
            policies: None,
        }
    }

    /// Selects all twelve Signal resource policy families. Process-local declarations
    /// require terminal-only Signal retention.
    pub fn with_resource_policies(mut self, policies: BridgeOwnedAsyncResourcePolicies) -> Self {
        self.policies = Some(Box::new(policies));
        self
    }

    pub fn with_managed_queue_capacity(mut self, capacity: std::num::NonZeroU64) -> Self {
        self.managed_queue_capacity = Some(capacity);
        self
    }

    /// Opts this queued resource into exact process-local terminal release.
    /// Installation refuses this posture without a managed queue capacity.
    pub fn with_process_local_retention(mut self) -> Self {
        self.process_local_retention = true;
        self
    }
}
fn duration(ticks: u64) -> Result<TemporalDuration, BridgeAsyncSourceDeclarationRejection> {
    TemporalDuration::temporal_duration(ticks).map_err(|error| {
        BridgeAsyncSourceDeclarationRejection::new(
            BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected,
            error.to_string(),
        )
    })
}

#[cfg(test)]
#[path = "declaration_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

#[cfg(test)]
#[path = "declaration_tests.rs"]
mod tests;
