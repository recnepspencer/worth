use serde::Serialize;

use super::super::super::retention_omission::RetentionOmission;
use super::super::ResourceRuntimeState;
use super::registry::resource_policy_resolution_signal_error;
use crate::data::resource::*;
use crate::logic::transaction::runtime::state::merge::canonical_digest;

pub(super) const RESOURCE_REPLAY_RECONSTRUCTION_SCHEMA_VERSION: &str =
    "worth.resource.replay-reconstruction.v2";
pub(super) const RESOURCE_REPLAY_RECONSTRUCTION_WITH_OMISSION_SCHEMA_VERSION: &str =
    "worth.resource.replay-reconstruction.v3";

#[derive(Debug, Serialize)]
pub(super) struct ResourceReplayLifecycleDigestBasis<'a> {
    pub(in crate::logic::transaction::runtime::state::resource) schema_version: &'static str,
    pub(in crate::logic::transaction::runtime::state::resource) lifecycle_entries:
        &'a [ResourceReplayLifecycleDigestEntry],
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(super) struct ResourceReplayLifecycleDigestEntry {
    pub(in crate::logic::transaction::runtime::state::resource) node: ResourceNodeId,
    pub(in crate::logic::transaction::runtime::state::resource) lifecycle: ResourceLifecycleClass,
    pub(in crate::logic::transaction::runtime::state::resource) lifecycle_ordinal:
        ResourceLifecycleOrdinal,
}

#[derive(Debug, Serialize)]
pub(super) struct ResourceReplayOutputContinuityDigestBasis<'a> {
    pub(in crate::logic::transaction::runtime::state::resource) schema_version: &'static str,
    pub(in crate::logic::transaction::runtime::state::resource) output_entries:
        &'a [ResourceReplayOutputContinuityDigestEntry],
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(super) struct ResourceReplayOutputContinuityDigestEntry {
    pub(in crate::logic::transaction::runtime::state::resource) node: ResourceNodeId,
    pub(in crate::logic::transaction::runtime::state::resource) output_continuity:
        ResourceOutputContinuity,
    pub(in crate::logic::transaction::runtime::state::resource) lifecycle_ordinal:
        ResourceLifecycleOrdinal,
}

#[derive(Debug, Serialize)]
pub(super) struct ResourceReplayDescriptorDigestBasis<'a> {
    pub(in crate::logic::transaction::runtime::state::resource) schema_version: &'static str,
    pub(in crate::logic::transaction::runtime::state::resource) descriptors:
        &'a [LoweredResourceDescriptor],
}

#[derive(Debug, Serialize)]
pub(super) struct ResourceReplayDenialDigestBasis<'a> {
    pub(in crate::logic::transaction::runtime::state::resource) schema_version: &'static str,
    pub(in crate::logic::transaction::runtime::state::resource) denied_completions:
        &'a [ResourceReplayDeniedCompletionEntryDigestBasis],
    pub(in crate::logic::transaction::runtime::state::resource) unavailable_denied_completions:
        &'a [ResourceReplayUnavailableDeniedCompletionDigestBasis],
    #[serde(skip_serializing_if = "RetentionOmission::is_empty")]
    pub(in crate::logic::transaction::runtime::state::resource) expired_denied_availability:
        RetentionOmission,
}

#[derive(Debug, Serialize)]
pub(super) struct ResourceReplayRetryLineageDigestBasis<'a> {
    pub(in crate::logic::transaction::runtime::state::resource) schema_version: &'static str,
    pub(in crate::logic::transaction::runtime::state::resource) retained_retry_lineages:
        &'a [RetainedResourceRetryLineage],
    pub(in crate::logic::transaction::runtime::state::resource) unavailable_retry_lineages:
        &'a [ResourceRetainedRetryLineageAvailability],
    #[serde(skip_serializing_if = "RetentionOmission::is_empty")]
    pub(in crate::logic::transaction::runtime::state::resource) expired_retry_availability:
        RetentionOmission,
}

#[derive(Debug, Serialize)]
pub(super) struct ResourceReplayInFlightDigestBasis<'a> {
    pub(in crate::logic::transaction::runtime::state::resource) schema_version: &'static str,
    pub(in crate::logic::transaction::runtime::state::resource) in_flight_requests:
        &'a [ResourceReplayInFlightEntryDigestBasis<'a>],
    pub(in crate::logic::transaction::runtime::state::resource) retained_history_availability:
        &'a [ResourceRetainedHistoryAvailability],
    #[serde(skip_serializing_if = "RetentionOmission::is_empty")]
    pub(in crate::logic::transaction::runtime::state::resource) expired_lifecycle_availability:
        RetentionOmission,
}

#[derive(Debug, Serialize)]
pub(super) struct ResourceReplayHandleDigestBasis {
    pub(in crate::logic::transaction::runtime::state::resource) request_id: ResourceRequestId,
    pub(in crate::logic::transaction::runtime::state::resource) generation: ResourceGeneration,
}

#[derive(Debug, Serialize)]
pub(super) struct ResourceReplayDeniedCompletionEntryDigestBasis {
    pub(in crate::logic::transaction::runtime::state::resource) class: CompletionDenialClass,
    pub(in crate::logic::transaction::runtime::state::resource) node: Option<ResourceNodeId>,
    pub(in crate::logic::transaction::runtime::state::resource) request_id: ResourceRequestId,
    pub(in crate::logic::transaction::runtime::state::resource) generation: ResourceGeneration,
    pub(in crate::logic::transaction::runtime::state::resource) restore_epoch: u64,
    pub(in crate::logic::transaction::runtime::state::resource) attempt: ResourceAttemptId,
    pub(in crate::logic::transaction::runtime::state::resource) payload_byte_len: u64,
}

#[derive(Debug, Serialize)]
pub(super) struct ResourceReplayUnavailableDeniedCompletionDigestBasis {
    pub(in crate::logic::transaction::runtime::state::resource) request_id: ResourceRequestId,
    pub(in crate::logic::transaction::runtime::state::resource) node: Option<ResourceNodeId>,
    pub(in crate::logic::transaction::runtime::state::resource) denial_class: CompletionDenialClass,
    pub(in crate::logic::transaction::runtime::state::resource) class:
        ResourceRetainedDeniedCompletionAvailabilityClass,
}

#[derive(Debug, Serialize)]
pub(super) struct ResourceReplayInFlightEntryDigestBasis<'a> {
    pub(in crate::logic::transaction::runtime::state::resource) handle:
        ResourceReplayHandleDigestBasis,
    pub(in crate::logic::transaction::runtime::state::resource) node: ResourceNodeId,
    pub(in crate::logic::transaction::runtime::state::resource) descriptor_id: ResourceDescriptorId,
    pub(in crate::logic::transaction::runtime::state::resource) attempt: ResourceAttemptId,
    pub(in crate::logic::transaction::runtime::state::resource) request_intent_digest: &'a str,
    pub(in crate::logic::transaction::runtime::state::resource) generation_started_tick:
        crate::data::temporal::ClockTick,
    pub(in crate::logic::transaction::runtime::state::resource) lifecycle: ResourceLifecycleClass,
    pub(in crate::logic::transaction::runtime::state::resource) lifecycle_ordinal:
        ResourceLifecycleOrdinal,
    pub(in crate::logic::transaction::runtime::state::resource) status: ResourceInFlightStatus,
    pub(in crate::logic::transaction::runtime::state::resource) has_timeout_wake: bool,
    pub(in crate::logic::transaction::runtime::state::resource) timeout_duration:
        Option<crate::data::temporal::TemporalDuration>,
    pub(in crate::logic::transaction::runtime::state::resource) timeout_due_tick:
        Option<crate::data::temporal::ClockTick>,
    pub(in crate::logic::transaction::runtime::state::resource) timeout_outcome_class:
        ResourceTimeoutOutcomeClass,
    pub(in crate::logic::transaction::runtime::state::resource) timeout_deadline_authority:
        ResourceTimeoutDeadlineAuthority,
    pub(in crate::logic::transaction::runtime::state::resource) timeout_decision_digest:
        &'a ResourcePolicyDigest,
    pub(in crate::logic::transaction::runtime::state::resource) revalidation_freshness_class:
        Option<ResourceRevalidationFreshnessClass>,
    pub(in crate::logic::transaction::runtime::state::resource) revalidation_freshness_digest:
        Option<String>,
    pub(in crate::logic::transaction::runtime::state::resource) revalidation_policy_decision_digest:
        Option<ResourcePolicyDigest>,
    pub(in crate::logic::transaction::runtime::state::resource) superseded_by:
        Option<ResourceReplayHandleDigestBasis>,
    pub(in crate::logic::transaction::runtime::state::resource) managed_queue_depth: Option<u64>,
    pub(in crate::logic::transaction::runtime::state::resource) managed_queue_capacity: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(super) struct ResourceReplayDigestBasis<'a> {
    pub(in crate::logic::transaction::runtime::state::resource) schema_version: &'static str,
    pub(in crate::logic::transaction::runtime::state::resource) descriptor_digest: &'a str,
    pub(in crate::logic::transaction::runtime::state::resource) lifecycle_digest: &'a str,
    pub(in crate::logic::transaction::runtime::state::resource) output_continuity_digest: &'a str,
    pub(in crate::logic::transaction::runtime::state::resource) denied_completion_digest: &'a str,
    pub(in crate::logic::transaction::runtime::state::resource) retry_lineage_digest: &'a str,
    pub(in crate::logic::transaction::runtime::state::resource) in_flight_digest: &'a str,
    pub(in crate::logic::transaction::runtime::state::resource) retained_history_unavailable_count:
        u32,
    pub(in crate::logic::transaction::runtime::state::resource) denied_completion_unavailable_count:
        u32,
    pub(in crate::logic::transaction::runtime::state::resource) retry_lineage_unavailable_count:
        u32,
}

#[derive(Debug, Serialize)]
pub(in crate::logic::transaction::runtime::state::resource) struct ResourceRetentionCompactionPolicyProvenanceDigestBasis<
    'a,
> {
    pub(in crate::logic::transaction::runtime::state::resource) schema_version: &'static str,
    pub(in crate::logic::transaction::runtime::state::resource) retained_history_decision_digests:
        &'a [&'a str],
    pub(in crate::logic::transaction::runtime::state::resource) retry_lineage_decision_digests:
        &'a [&'a str],
    #[serde(skip_serializing_if = "RetentionOmission::is_empty")]
    pub(in crate::logic::transaction::runtime::state::resource) expired_lifecycle:
        RetentionOmission,
    #[serde(skip_serializing_if = "RetentionOmission::is_empty")]
    pub(in crate::logic::transaction::runtime::state::resource) expired_retry: RetentionOmission,
}

#[derive(Debug, Serialize)]
pub(super) struct ObserverDemandResourceRevalidationDigestBasis<'a> {
    pub(in crate::logic::transaction::runtime::state::resource) schema_version: &'static str,
    pub(in crate::logic::transaction::runtime::state::resource) observer_id: u64,
    pub(in crate::logic::transaction::runtime::state::resource) handle_id: u64,
    pub(in crate::logic::transaction::runtime::state::resource) policy: &'a str,
    pub(in crate::logic::transaction::runtime::state::resource) matched_nodes: Vec<String>,
    pub(in crate::logic::transaction::runtime::state::resource) touched: bool,
    pub(in crate::logic::transaction::runtime::state::resource) recomputed: bool,
    pub(in crate::logic::transaction::runtime::state::resource) meaningful_change: bool,
    pub(in crate::logic::transaction::runtime::state::resource) trigger_matched: bool,
    pub(in crate::logic::transaction::runtime::state::resource) delivered: bool,
}

impl ResourceRuntimeState {
    pub(crate) fn observer_demand_observation_digest(
        observer_id: u64,
        handle_id: u64,
        policy: &str,
        matched_nodes: &[String],
        touched: bool,
        recomputed: bool,
        meaningful_change: bool,
        trigger_matched: bool,
        delivered: bool,
    ) -> String {
        canonical_digest(&ObserverDemandResourceRevalidationDigestBasis {
            schema_version: "worth.resource.observer-demand-revalidation.v1",
            observer_id,
            handle_id,
            policy,
            matched_nodes: matched_nodes.to_vec(),
            touched,
            recomputed,
            meaningful_change,
            trigger_matched,
            delivered,
        })
    }

    pub(super) fn lifecycle_digest_entry(
        summary: ResourceLifecycleSummary,
    ) -> ResourceReplayLifecycleDigestEntry {
        ResourceReplayLifecycleDigestEntry {
            node: summary.node(),
            lifecycle: summary.lifecycle(),
            lifecycle_ordinal: summary.lifecycle_ordinal(),
        }
    }

    pub(super) fn output_continuity_digest_entry(
        summary: ResourceLifecycleSummary,
    ) -> ResourceReplayOutputContinuityDigestEntry {
        ResourceReplayOutputContinuityDigestEntry {
            node: summary.node(),
            output_continuity: summary.output_continuity(),
            lifecycle_ordinal: summary.lifecycle_ordinal(),
        }
    }

    pub(super) fn replay_decision_plan_from_validated(
        &self,
        validated: &ValidatedResourcePolicyDeclaration,
    ) -> Result<ResourceReplayDecisionPlan, crate::data::error::SignalError> {
        let frozen = FrozenResourcePolicyDescriptorSet::from_validated_declaration(
            validated,
            &self.policy_registry,
        )
        .map_err(resource_policy_resolution_signal_error)?;
        ResourceReplayDecisionPlan::lower(validated.declaration().replay_policy(), frozen.replay())
            .map_err(resource_policy_resolution_signal_error)
    }

    pub fn replay_reconstruction_width(&self) -> u32 {
        let width = self
            .descriptors
            .len()
            .saturating_add(self.lifecycle_by_node.len())
            .saturating_add(self.denied_completions.len())
            .saturating_add(self.pruned_denied_completions_by_id.len())
            .saturating_add(self.retained_retry_lineage_by_ordinal.len())
            .saturating_add(self.pruned_retry_lineage_by_ordinal.len())
            .saturating_add(self.in_flight_by_request.len())
            .saturating_add(self.retained_in_flight_history_by_request.len())
            .saturating_add(self.pruned_in_flight_history_by_request.len());
        width.min(u32::MAX as usize) as u32
    }
}
