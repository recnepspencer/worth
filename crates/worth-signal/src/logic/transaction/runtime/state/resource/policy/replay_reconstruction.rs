use super::super::ResourceRuntimeState;
use super::replay::{
    ResourceReplayDenialDigestBasis, ResourceReplayDeniedCompletionEntryDigestBasis,
    ResourceReplayDescriptorDigestBasis, ResourceReplayDigestBasis,
    ResourceReplayHandleDigestBasis, ResourceReplayInFlightDigestBasis,
    ResourceReplayInFlightEntryDigestBasis, ResourceReplayLifecycleDigestBasis,
    ResourceReplayLifecycleDigestEntry, ResourceReplayOutputContinuityDigestBasis,
    ResourceReplayOutputContinuityDigestEntry, ResourceReplayRetryLineageDigestBasis,
    ResourceReplayUnavailableDeniedCompletionDigestBasis,
    RESOURCE_REPLAY_RECONSTRUCTION_SCHEMA_VERSION,
    RESOURCE_REPLAY_RECONSTRUCTION_WITH_OMISSION_SCHEMA_VERSION,
};
use crate::data::resource::*;
use crate::data::telemetry::ResourceTelemetry;
use crate::logic::transaction::runtime::state::merge::canonical_digest;

struct ReplayReconstructionBasis {
    descriptors: Vec<LoweredResourceDescriptor>,
    lifecycle_summaries: Vec<ResourceLifecycleSummary>,
    lifecycle_entries: Vec<ResourceReplayLifecycleDigestEntry>,
    output_entries: Vec<ResourceReplayOutputContinuityDigestEntry>,
    denied_completion_entries: Vec<ResourceReplayDeniedCompletionEntryDigestBasis>,
    unavailable_denied_completion_entries:
        Vec<ResourceReplayUnavailableDeniedCompletionDigestBasis>,
    retained_retry_lineages: Vec<RetainedResourceRetryLineage>,
    unavailable_retry_lineages: Vec<ResourceRetainedRetryLineageAvailability>,
    in_flight_requests: Vec<InFlightResourceRequest>,
    retained_history_availability: Vec<ResourceRetainedHistoryAvailability>,
    retained_history_unavailable_count: u32,
    expired_lifecycle_availability: super::super::super::retention_omission::RetentionOmission,
    expired_denied_availability: super::super::super::retention_omission::RetentionOmission,
    expired_retry_availability: super::super::super::retention_omission::RetentionOmission,
}

struct ReplayReconstructionWidths {
    descriptor: u32,
    lifecycle_summary: u32,
    denied_completion: u32,
    retained_retry_lineage: u32,
    in_flight: u32,
    denied_completion_unavailable: u32,
    retry_lineage_unavailable: u32,
}

struct ReplayReconstructionDigests {
    descriptor: String,
    lifecycle: String,
    output_continuity: String,
    denied_completion: String,
    retry_lineage: String,
    in_flight: String,
    replay: String,
}

impl ReplayReconstructionBasis {
    fn widths(&self) -> ReplayReconstructionWidths {
        ReplayReconstructionWidths {
            descriptor: self.descriptors.len() as u32,
            lifecycle_summary: self.lifecycle_summaries.len() as u32,
            denied_completion: self.denied_completion_entries.len() as u32,
            retained_retry_lineage: self.retained_retry_lineages.len() as u32,
            in_flight: self.in_flight_requests.len() as u32,
            denied_completion_unavailable: self
                .unavailable_denied_completion_entries
                .len()
                .saturating_add(
                    usize::try_from(self.expired_denied_availability.count()).unwrap_or(usize::MAX),
                )
                .min(u32::MAX as usize) as u32,
            retry_lineage_unavailable: self
                .unavailable_retry_lineages
                .len()
                .saturating_add(
                    usize::try_from(self.expired_retry_availability.count()).unwrap_or(usize::MAX),
                )
                .min(u32::MAX as usize) as u32,
        }
    }
}

impl ResourceRuntimeState {
    pub fn reconstruct_replay_summary_optional(
        &self,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> ResourceReplayReconstructionReport {
        let basis = self.collect_replay_reconstruction_basis();
        let widths = basis.widths();
        let digests = Self::digest_replay_reconstruction(&basis);
        let performance = self.record_replay_reconstruction_telemetry_optional(
            &widths,
            basis.retained_history_unavailable_count,
            telemetry,
        );
        ResourceReplayReconstructionReport::new(
            widths.descriptor,
            widths.lifecycle_summary,
            widths.denied_completion,
            widths.retained_retry_lineage,
            widths.in_flight,
            basis.retained_history_unavailable_count,
            widths.denied_completion_unavailable,
            widths.retry_lineage_unavailable,
            digests.descriptor,
            digests.lifecycle,
            digests.output_continuity,
            digests.denied_completion,
            digests.retry_lineage,
            digests.in_flight,
            digests.replay,
            performance,
        )
    }

    fn collect_replay_reconstruction_basis(&self) -> ReplayReconstructionBasis {
        let descriptors = self.descriptors.values().cloned().collect::<Vec<_>>();
        let lifecycle_summaries = self.lifecycle_by_node.values().copied().collect::<Vec<_>>();
        let lifecycle_entries = lifecycle_summaries
            .iter()
            .copied()
            .map(Self::lifecycle_digest_entry)
            .collect::<Vec<_>>();
        let output_entries = lifecycle_summaries
            .iter()
            .copied()
            .map(Self::output_continuity_digest_entry)
            .collect::<Vec<_>>();
        let denied_completion_entries = self
            .denied_completions
            .values()
            .map(|denied| ResourceReplayDeniedCompletionEntryDigestBasis {
                class: denied.class(),
                node: denied.node(),
                request_id: denied.request_id(),
                generation: denied.generation(),
                restore_epoch: denied.branch_epoch().restore_epoch(),
                attempt: denied.attempt(),
                payload_byte_len: denied.payload_byte_len(),
            })
            .collect::<Vec<_>>();
        let unavailable_denied_completion_entries = self
            .pruned_denied_completions_by_id
            .values()
            .map(
                |availability| ResourceReplayUnavailableDeniedCompletionDigestBasis {
                    request_id: availability.request_id(),
                    node: availability.node(),
                    denial_class: availability.denial_class(),
                    class: availability.class(),
                },
            )
            .collect::<Vec<_>>();
        let retained_retry_lineages = self
            .retained_retry_lineage_by_ordinal
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let unavailable_retry_lineages = self
            .pruned_retry_lineage_by_ordinal
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let mut in_flight_requests = self
            .in_flight_by_request
            .values()
            .cloned()
            .collect::<Vec<_>>();
        in_flight_requests.extend(self.retained_in_flight_history_by_request.values().cloned());
        in_flight_requests.sort_by_key(|request| request.handle());
        let retained_history_availability = self
            .pruned_in_flight_history_by_request
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let retained_history_unavailable_count = lifecycle_summaries
            .iter()
            .filter(|summary| {
                summary.lifecycle() == ResourceLifecycleClass::RetainedHistoryUnavailable
            })
            .count()
            .saturating_add(retained_history_availability.len())
            .saturating_add(
                usize::try_from(self.expired_lifecycle_availability.count()).unwrap_or(usize::MAX),
            )
            .min(u32::MAX as usize) as u32;
        ReplayReconstructionBasis {
            descriptors,
            lifecycle_summaries,
            lifecycle_entries,
            output_entries,
            denied_completion_entries,
            unavailable_denied_completion_entries,
            retained_retry_lineages,
            unavailable_retry_lineages,
            in_flight_requests,
            retained_history_availability,
            retained_history_unavailable_count,
            expired_lifecycle_availability: self.expired_lifecycle_availability,
            expired_denied_availability: self.expired_denied_availability,
            expired_retry_availability: self.expired_retry_availability,
        }
    }

    fn digest_replay_reconstruction(
        basis: &ReplayReconstructionBasis,
    ) -> ReplayReconstructionDigests {
        let schema_version = if basis.expired_lifecycle_availability.is_empty()
            && basis.expired_denied_availability.is_empty()
            && basis.expired_retry_availability.is_empty()
        {
            RESOURCE_REPLAY_RECONSTRUCTION_SCHEMA_VERSION
        } else {
            RESOURCE_REPLAY_RECONSTRUCTION_WITH_OMISSION_SCHEMA_VERSION
        };
        let descriptor = canonical_digest(&ResourceReplayDescriptorDigestBasis {
            schema_version,
            descriptors: &basis.descriptors,
        });
        let lifecycle = canonical_digest(&ResourceReplayLifecycleDigestBasis {
            schema_version,
            lifecycle_entries: &basis.lifecycle_entries,
        });
        let output_continuity = canonical_digest(&ResourceReplayOutputContinuityDigestBasis {
            schema_version,
            output_entries: &basis.output_entries,
        });
        let denied_completion = canonical_digest(&ResourceReplayDenialDigestBasis {
            schema_version,
            denied_completions: &basis.denied_completion_entries,
            unavailable_denied_completions: &basis.unavailable_denied_completion_entries,
            expired_denied_availability: basis.expired_denied_availability,
        });
        let retry_lineage = canonical_digest(&ResourceReplayRetryLineageDigestBasis {
            schema_version,
            retained_retry_lineages: &basis.retained_retry_lineages,
            unavailable_retry_lineages: &basis.unavailable_retry_lineages,
            expired_retry_availability: basis.expired_retry_availability,
        });
        let in_flight_entries = basis
            .in_flight_requests
            .iter()
            .map(Self::in_flight_digest_entry)
            .collect::<Vec<_>>();
        let in_flight = canonical_digest(&ResourceReplayInFlightDigestBasis {
            schema_version,
            in_flight_requests: &in_flight_entries,
            retained_history_availability: &basis.retained_history_availability,
            expired_lifecycle_availability: basis.expired_lifecycle_availability,
        });
        let replay = canonical_digest(&ResourceReplayDigestBasis {
            schema_version,
            descriptor_digest: &descriptor,
            lifecycle_digest: &lifecycle,
            output_continuity_digest: &output_continuity,
            denied_completion_digest: &denied_completion,
            retry_lineage_digest: &retry_lineage,
            in_flight_digest: &in_flight,
            retained_history_unavailable_count: basis.retained_history_unavailable_count,
            denied_completion_unavailable_count: basis.widths().denied_completion_unavailable,
            retry_lineage_unavailable_count: basis.widths().retry_lineage_unavailable,
        });
        ReplayReconstructionDigests {
            descriptor,
            lifecycle,
            output_continuity,
            denied_completion,
            retry_lineage,
            in_flight,
            replay,
        }
    }

    fn in_flight_digest_entry(
        request: &InFlightResourceRequest,
    ) -> ResourceReplayInFlightEntryDigestBasis<'_> {
        let revalidation_freshness_decision = request.revalidation_freshness_decision();
        let managed_queue = request.managed_queue();
        ResourceReplayInFlightEntryDigestBasis {
            handle: ResourceReplayHandleDigestBasis {
                request_id: request.handle().request_id(),
                generation: request.handle().generation(),
            },
            node: request.node(),
            descriptor_id: request.descriptor_id(),
            attempt: request.attempt(),
            request_intent_digest: request.request_intent_digest().as_str(),
            generation_started_tick: request.generation_started_tick(),
            lifecycle: request.lifecycle(),
            lifecycle_ordinal: request.lifecycle_ordinal(),
            status: request.status(),
            has_timeout_wake: request.timeout_wake_id().is_some(),
            timeout_duration: request.timeout_duration(),
            timeout_due_tick: request.timeout_due_tick(),
            timeout_outcome_class: request.timeout_outcome_class(),
            timeout_deadline_authority: request.timeout_deadline_authority(),
            timeout_decision_digest: request.timeout_decision_digest(),
            revalidation_freshness_class: revalidation_freshness_decision
                .as_ref()
                .map(|decision| decision.class()),
            revalidation_freshness_digest: revalidation_freshness_decision
                .as_ref()
                .map(|decision| decision.freshness_digest().to_owned()),
            revalidation_policy_decision_digest: revalidation_freshness_decision
                .as_ref()
                .map(|decision| decision.policy_decision_digest().clone()),
            superseded_by: request
                .superseded_by()
                .map(|handle| ResourceReplayHandleDigestBasis {
                    request_id: handle.request_id(),
                    generation: handle.generation(),
                }),
            managed_queue_depth: managed_queue.map(ResourceManagedQueueState::queue_depth),
            managed_queue_capacity: managed_queue.map(ResourceManagedQueueState::queue_capacity),
        }
    }

    fn record_replay_reconstruction_telemetry_optional(
        &self,
        widths: &ReplayReconstructionWidths,
        retained_history_unavailable_count: u32,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> ResourceBoundaryPerformanceEnvelope {
        if let Some(telemetry) = telemetry {
            telemetry.resource_replay_reconstruction_count += 1;
            telemetry.resource_replay_reconstruction_lifecycle_width = telemetry
                .resource_replay_reconstruction_lifecycle_width
                .max(widths.lifecycle_summary as u64);
            telemetry.resource_replay_reconstruction_denial_width = telemetry
                .resource_replay_reconstruction_denial_width
                .max(widths.denied_completion as u64);
            telemetry.resource_replay_reconstruction_in_flight_width = telemetry
                .resource_replay_reconstruction_in_flight_width
                .max(widths.in_flight as u64);
            telemetry.resource_retained_history_unavailable_count = telemetry
                .resource_retained_history_unavailable_count
                .saturating_add(retained_history_unavailable_count as u64);
            Self::record_boundary_performance(
                telemetry,
                ResourceBoundaryPerformanceEnvelope::replay_reconstruction(
                    widths.descriptor,
                    widths.lifecycle_summary,
                    widths.denied_completion,
                    widths.in_flight,
                    retained_history_unavailable_count,
                ),
            )
        } else {
            ResourceBoundaryPerformanceEnvelope::replay_reconstruction(
                widths.descriptor,
                widths.lifecycle_summary,
                widths.denied_completion,
                widths.in_flight,
                retained_history_unavailable_count,
            )
        }
    }
}
