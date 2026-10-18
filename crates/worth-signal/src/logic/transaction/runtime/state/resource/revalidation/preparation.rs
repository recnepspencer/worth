use super::super::ResourceRuntimeState;
use crate::data::resource::*;
use crate::data::telemetry::ResourceTelemetry;
use crate::data::temporal::ReadyTemporalWake;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::logic::transaction::runtime::state::resource) enum ResourceRevalidationAdmissionPreview
{
    Proceed {
        descriptor_id: ResourceDescriptorId,
    },
    Coalesce {
        descriptor_id: ResourceDescriptorId,
        active_request_id: ResourceRequestId,
    },
    Deny(ResourceRevalidationDenialClass),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::logic::transaction::runtime::state::resource) enum PreparedResourceRevalidationDisposition
{
    Proceed {
        descriptor_id: ResourceDescriptorId,
    },
    Coalesce {
        descriptor_id: ResourceDescriptorId,
        active_request_id: ResourceRequestId,
    },
}

#[derive(Debug)]
pub(in crate::logic::transaction::runtime::state) struct PreparedResourceRevalidation {
    pub(in crate::logic::transaction::runtime::state::resource) intent: ResourceRevalidationIntent,
    pub(in crate::logic::transaction::runtime::state::resource) revalidation_decision_digest:
        ResourcePolicyDigest,
    pub(in crate::logic::transaction::runtime::state::resource) freshness_decision:
        ResourceRevalidationFreshnessDecision,
    pub(in crate::logic::transaction::runtime::state::resource) evidence:
        ResourceRevalidationEvidence,
    pub(in crate::logic::transaction::runtime::state::resource) disposition:
        PreparedResourceRevalidationDisposition,
}

impl ResourceRuntimeState {
    fn prepare_resource_revalidation(
        &mut self,
        intent: ResourceRevalidationIntent,
        count_policy_decision: bool,
        revalidation_decision_digest: ResourcePolicyDigest,
        freshness_decision: ResourceRevalidationFreshnessDecision,
        evidence: ResourceRevalidationEvidence,
        mut telemetry: Option<&mut ResourceTelemetry>,
    ) -> Result<PreparedResourceRevalidation, ResourceRevalidationReport> {
        if count_policy_decision && let Some(telemetry) = telemetry.as_deref_mut() {
            telemetry.resource_revalidation_policy_decision_count += 1;
        }
        let disposition = match self.preview_revalidation_admission(intent, &freshness_decision) {
            ResourceRevalidationAdmissionPreview::Proceed { descriptor_id } => {
                PreparedResourceRevalidationDisposition::Proceed { descriptor_id }
            }
            ResourceRevalidationAdmissionPreview::Coalesce {
                descriptor_id,
                active_request_id,
            } => PreparedResourceRevalidationDisposition::Coalesce {
                descriptor_id,
                active_request_id,
            },
            ResourceRevalidationAdmissionPreview::Deny(class) => {
                return Err(self.deny_revalidation(intent, class, telemetry));
            }
        };
        Ok(PreparedResourceRevalidation {
            intent,
            revalidation_decision_digest,
            freshness_decision,
            evidence,
            disposition,
        })
    }
    pub(in crate::logic::transaction::runtime::state) fn prepare_explicit_resource_revalidation(
        &mut self,
        intent: ResourceRevalidationIntent,
        revalidation_decision_digest: ResourcePolicyDigest,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> Result<PreparedResourceRevalidation, ResourceRevalidationReport> {
        self.prepare_resource_revalidation(
            intent,
            true,
            revalidation_decision_digest.clone(),
            ResourceRevalidationFreshnessDecision::explicit_intent(revalidation_decision_digest),
            ResourceRevalidationEvidence::ExplicitIntent {
                expected_active: intent.expected_active(),
            },
            telemetry,
        )
    }
    pub(in crate::logic::transaction::runtime::state) fn prepare_forced_resource_revalidation(
        &mut self,
        proof: ActiveResourceRevalidationProof,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> Result<PreparedResourceRevalidation, ResourceRevalidationReport> {
        let intent = ResourceRevalidationIntent::with_expected_active(proof.node(), proof.handle());
        self.prepare_resource_revalidation(
            intent,
            false,
            proof.decision_digest().clone(),
            ResourceRevalidationFreshnessDecision::forced_active_handle(
                proof.handle(),
                proof.decision_digest().clone(),
            ),
            ResourceRevalidationEvidence::ForcedActive(proof),
            telemetry,
        )
    }
    pub(in crate::logic::transaction::runtime::state) fn prepare_dependency_change_resource_revalidation(
        &mut self,
        proof: DependencyChangeResourceRevalidationProof,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> Result<PreparedResourceRevalidation, ResourceRevalidationReport> {
        let intent = ResourceRevalidationIntent::new(proof.node());
        self.prepare_resource_revalidation(
            intent,
            false,
            proof.decision_digest().clone(),
            ResourceRevalidationFreshnessDecision::dependency_change(&proof),
            ResourceRevalidationEvidence::DependencyChange(proof),
            telemetry,
        )
    }
    pub(in crate::logic::transaction::runtime::state) fn prepare_observer_demand_resource_revalidation(
        &mut self,
        proof: ObserverDemandResourceRevalidationProof,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> Result<PreparedResourceRevalidation, ResourceRevalidationReport> {
        let intent = ResourceRevalidationIntent::new(proof.node());
        self.prepare_resource_revalidation(
            intent,
            false,
            proof.decision_digest().clone(),
            ResourceRevalidationFreshnessDecision::observer_demand(&proof),
            ResourceRevalidationEvidence::ObserverDemand(proof),
            telemetry,
        )
    }
    pub(in crate::logic::transaction::runtime::state) fn prepare_terminal_state_resource_revalidation(
        &mut self,
        proof: TerminalStateResourceRevalidationProof,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> Result<PreparedResourceRevalidation, ResourceRevalidationReport> {
        let intent = ResourceRevalidationIntent::new(proof.node());
        self.prepare_resource_revalidation(
            intent,
            false,
            proof.decision_digest().clone(),
            ResourceRevalidationFreshnessDecision::terminal_state(&proof),
            ResourceRevalidationEvidence::TerminalState(proof),
            telemetry,
        )
    }
    pub(in crate::logic::transaction::runtime::state) fn prepare_fulfilled_lifecycle_resource_revalidation(
        &mut self,
        proof: FulfilledLifecycleResourceRevalidationProof,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> Result<PreparedResourceRevalidation, ResourceRevalidationReport> {
        let intent = ResourceRevalidationIntent::new(proof.node());
        self.prepare_resource_revalidation(
            intent,
            false,
            proof.decision_digest().clone(),
            ResourceRevalidationFreshnessDecision::fulfilled_lifecycle(&proof),
            ResourceRevalidationEvidence::FulfilledLifecycle(proof),
            telemetry,
        )
    }
    pub(in crate::logic::transaction::runtime::state) fn prepare_stale_after_resource_revalidation(
        &mut self,
        node: ResourceNodeId,
        ready_wake: ReadyTemporalWake,
        revalidation_decision_digest: ResourcePolicyDigest,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> Result<PreparedResourceRevalidation, ResourceRevalidationReport> {
        let intent = ResourceRevalidationIntent::new(node);
        self.prepare_resource_revalidation(
            intent,
            false,
            revalidation_decision_digest.clone(),
            ResourceRevalidationFreshnessDecision::stale_after(
                node,
                ready_wake.id(),
                revalidation_decision_digest,
            ),
            ResourceRevalidationEvidence::StaleAfter(ready_wake),
            telemetry,
        )
    }
}
