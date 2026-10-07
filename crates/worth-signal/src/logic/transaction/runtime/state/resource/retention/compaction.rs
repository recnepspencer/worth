use super::super::policy::replay::ResourceRetentionCompactionPolicyProvenanceDigestBasis;
use super::super::ResourceRuntimeState;
use crate::data::resource::*;
use crate::data::telemetry::ResourceTelemetry;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{self, Write};

#[derive(Default)]
struct LifecycleCompactionCounts {
    selected_terminal_count: u32,
    reclaimed_in_flight_count: u32,
    retained_history_write_count: u32,
    retained_history_pruned_count: u32,
    retained_history_unavailable_count: u32,
    retained_denied_completion_pruned_count: u32,
    retained_retry_lineage_pruned_count: u32,
    compacted_terminal_summary_count: u32,
    compacted_superseded_count: u32,
    compacted_cancelled_count: u32,
    compacted_timed_out_count: u32,
    expired_lifecycle_availability_count: u32,
    expired_denied_availability_count: u32,
    expired_retry_availability_count: u32,
}

impl ResourceRuntimeState {
    pub fn compact_lifecycle_history_optional(
        &mut self,
        max_reclaimed: u32,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> ResourceLifecycleRetentionCompactionReport {
        self.compact_lifecycle_history_with_budget_optional(
            max_reclaimed,
            ResourceRetentionCompactionBudget::unbounded(),
            telemetry,
        )
    }

    pub fn compact_lifecycle_history_with_retained_limit_optional(
        &mut self,
        max_reclaimed: u32,
        retained_history_limit: Option<u32>,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> ResourceLifecycleRetentionCompactionReport {
        let budget = retained_history_limit.map_or_else(
            ResourceRetentionCompactionBudget::unbounded,
            ResourceRetentionCompactionBudget::retained_history_limit_only,
        );
        self.compact_lifecycle_history_with_budget_optional(max_reclaimed, budget, telemetry)
    }

    pub fn compact_lifecycle_history_with_budget_optional(
        &mut self,
        max_reclaimed: u32,
        budget: ResourceRetentionCompactionBudget,
        mut telemetry: Option<&mut ResourceTelemetry>,
    ) -> ResourceLifecycleRetentionCompactionReport {
        let selected = self.select_compaction_candidates(max_reclaimed);
        let mut counts = LifecycleCompactionCounts {
            selected_terminal_count: selected.len() as u32,
            ..LifecycleCompactionCounts::default()
        };
        self.compact_selected_terminal_records(&selected, &mut counts);
        self.apply_retained_history_limit(budget.retained_lifecycle_history_limit(), &mut counts);
        self.apply_denied_completion_limit(budget.retained_denied_completion_limit(), &mut counts);
        self.apply_retry_lineage_limit(budget.retained_retry_lineage_limit(), &mut counts);
        let expired = self.expire_pruned_availability(budget);
        counts.expired_lifecycle_availability_count = expired[0];
        counts.expired_denied_availability_count = expired[1];
        counts.expired_retry_availability_count = expired[2];
        self.record_compaction_telemetry(&counts, telemetry.as_deref_mut());

        let retained_history_width = self.retained_in_flight_history_by_request.len() as u32;
        let retained_denied_completion_width = self.denied_completions.len() as u32;
        let retained_retry_lineage_width = self.retained_retry_lineage_by_ordinal.len() as u32;
        let hot_in_flight_width = self.in_flight_by_request.len() as u32;
        let policy_provenance_digest = self.compaction_policy_provenance_digest();
        let performance = Self::record_boundary_performance_optional(
            telemetry,
            ResourceBoundaryPerformanceEnvelope::lifecycle_retention_compaction(
                counts.selected_terminal_count,
                counts.reclaimed_in_flight_count,
                counts.retained_history_write_count,
            ),
        );
        ResourceLifecycleRetentionCompactionReport::new(
            counts.selected_terminal_count,
            counts.reclaimed_in_flight_count,
            counts.retained_history_write_count,
            counts.retained_history_pruned_count,
            counts.retained_history_unavailable_count,
            counts.retained_denied_completion_pruned_count,
            counts.retained_retry_lineage_pruned_count,
            retained_history_width,
            retained_denied_completion_width,
            retained_retry_lineage_width,
            hot_in_flight_width,
            counts.compacted_terminal_summary_count,
            counts.compacted_superseded_count,
            counts.compacted_cancelled_count,
            counts.compacted_timed_out_count,
            counts.expired_lifecycle_availability_count,
            counts.expired_denied_availability_count,
            counts.expired_retry_availability_count,
            self.expired_lifecycle_availability.count(),
            self.expired_denied_availability.count(),
            self.expired_retry_availability.count(),
            policy_provenance_digest,
            performance,
        )
    }

    fn select_compaction_candidates(&self, max_reclaimed: u32) -> Vec<ResourceRequestId> {
        self.terminal_in_flight_by_request
            .iter()
            .copied()
            .filter(|request_id| !self.pending_retry_by_request.contains_key(request_id))
            .filter(|request_id| {
                self.terminal_in_flight_record(*request_id)
                    .and_then(|request| request.managed_queue())
                    .is_none_or(ResourceManagedQueueState::is_empty)
            })
            .filter(|request_id| {
                let Some(in_flight) = self.terminal_in_flight_record(*request_id) else {
                    return false;
                };
                let Some(descriptor) = self.descriptor_for_node(in_flight.node()) else {
                    return false;
                };
                descriptor.retention_decision_plan().retains_rich_history()
                    || descriptor
                        .retention_decision_plan()
                        .permits_compaction_for_lifecycle(in_flight.lifecycle())
            })
            .take(max_reclaimed as usize)
            .collect()
    }

    fn compact_selected_terminal_records(
        &mut self,
        selected: &[ResourceRequestId],
        counts: &mut LifecycleCompactionCounts,
    ) {
        for request_id in selected.iter().copied() {
            self.terminal_in_flight_by_request.remove(&request_id);
            let Some(in_flight) = self
                .in_flight_by_request
                .remove(&request_id)
                .filter(|in_flight| in_flight.lifecycle().is_terminal())
            else {
                continue;
            };
            counts.reclaimed_in_flight_count = counts.reclaimed_in_flight_count.saturating_add(1);
            let Some(descriptor) = self.descriptor_for_node(in_flight.node()).cloned() else {
                continue;
            };
            if descriptor.retention_decision_plan().retains_rich_history() {
                self.retained_in_flight_history_by_request
                    .insert(request_id, in_flight);
                counts.retained_history_write_count =
                    counts.retained_history_write_count.saturating_add(1);
                continue;
            }
            let Some(class) = Self::retention_availability_class_for_lifecycle(
                &descriptor,
                in_flight.lifecycle(),
            ) else {
                continue;
            };
            self.pruned_in_flight_history_by_request.insert(
                request_id,
                Self::retention_availability_from_in_flight(&descriptor, in_flight, class),
            );
            counts.retained_history_unavailable_count =
                counts.retained_history_unavailable_count.saturating_add(1);
            match class {
                ResourceRetainedHistoryAvailabilityClass::TerminalSummaryOnly => {
                    counts.compacted_terminal_summary_count =
                        counts.compacted_terminal_summary_count.saturating_add(1)
                }
                ResourceRetainedHistoryAvailabilityClass::CompactSuperseded => {
                    counts.compacted_superseded_count =
                        counts.compacted_superseded_count.saturating_add(1)
                }
                ResourceRetainedHistoryAvailabilityClass::CompactCancelled => {
                    counts.compacted_cancelled_count =
                        counts.compacted_cancelled_count.saturating_add(1)
                }
                ResourceRetainedHistoryAvailabilityClass::CompactTimedOut => {
                    counts.compacted_timed_out_count =
                        counts.compacted_timed_out_count.saturating_add(1)
                }
                ResourceRetainedHistoryAvailabilityClass::PrunedByRetainedHistoryLimit => {}
            }
        }
    }

    fn apply_retained_history_limit(
        &mut self,
        retained_history_limit: Option<u32>,
        counts: &mut LifecycleCompactionCounts,
    ) {
        let Some(limit) = retained_history_limit else {
            return;
        };
        while self.retained_in_flight_history_by_request.len() > limit as usize {
            let Some(request_id) = self
                .retained_in_flight_history_by_request
                .keys()
                .next()
                .copied()
            else {
                break;
            };
            if let Some(pruned) = self
                .retained_in_flight_history_by_request
                .remove(&request_id)
            {
                let descriptor = self
                    .descriptor_for_node(pruned.node())
                    .cloned()
                    .expect("retained history pruning requires descriptor for node");
                self.pruned_in_flight_history_by_request.insert(
                    request_id,
                    Self::retention_availability_from_in_flight(
                        &descriptor,
                        pruned,
                        ResourceRetainedHistoryAvailabilityClass::PrunedByRetainedHistoryLimit,
                    ),
                );
                counts.retained_history_unavailable_count =
                    counts.retained_history_unavailable_count.saturating_add(1);
            }
            counts.retained_history_pruned_count =
                counts.retained_history_pruned_count.saturating_add(1);
        }
    }

    fn apply_denied_completion_limit(
        &mut self,
        retained_denied_completion_limit: Option<u32>,
        counts: &mut LifecycleCompactionCounts,
    ) {
        let Some(limit) = retained_denied_completion_limit else {
            return;
        };
        while self.denied_completions.len() > limit as usize {
            let Some(denial_id) = self.denied_completions.keys().next().copied() else {
                break;
            };
            if let Some(denied) = self.denied_completions.remove(&denial_id) {
                self.pruned_denied_completions_by_id.insert(
                    denial_id,
                    Self::pruned_denied_completion_availability(denied),
                );
                if let Some(node) = denied.node() {
                    self.rebuild_latest_denied_completion_for_node(node);
                }
            }
            counts.retained_denied_completion_pruned_count = counts
                .retained_denied_completion_pruned_count
                .saturating_add(1);
        }
    }

    fn apply_retry_lineage_limit(
        &mut self,
        retained_retry_lineage_limit: Option<u32>,
        counts: &mut LifecycleCompactionCounts,
    ) {
        let Some(limit) = retained_retry_lineage_limit else {
            return;
        };
        while self.retained_retry_lineage_by_ordinal.len() > limit as usize {
            let Some(retry_ordinal) = self
                .retained_retry_lineage_by_ordinal
                .keys()
                .next()
                .copied()
            else {
                break;
            };
            if let Some(retained) = self
                .retained_retry_lineage_by_ordinal
                .remove(&retry_ordinal)
            {
                self.pruned_retry_lineage_by_ordinal.insert(
                    retry_ordinal,
                    ResourceRetainedRetryLineageAvailability::from_retained(
                        retained,
                        ResourceRetainedRetryLineageAvailabilityClass::PrunedByRetainedRetryLineageLimit,
                    ),
                );
            }
            counts.retained_retry_lineage_pruned_count =
                counts.retained_retry_lineage_pruned_count.saturating_add(1);
        }
    }

    fn record_compaction_telemetry(
        &self,
        counts: &LifecycleCompactionCounts,
        telemetry: Option<&mut ResourceTelemetry>,
    ) {
        if let Some(telemetry) = telemetry {
            telemetry.resource_hot_in_flight_compaction_count += 1;
            telemetry.resource_in_flight_retired_record_count = telemetry
                .resource_in_flight_retired_record_count
                .saturating_add(counts.selected_terminal_count as u64);
            telemetry.resource_in_flight_reclaimed_record_count = telemetry
                .resource_in_flight_reclaimed_record_count
                .saturating_add(counts.reclaimed_in_flight_count as u64);
            telemetry.resource_retained_lifecycle_history_write_count = telemetry
                .resource_retained_lifecycle_history_write_count
                .saturating_add(counts.retained_history_write_count as u64);
            telemetry.resource_retained_lifecycle_history_pruned_count = telemetry
                .resource_retained_lifecycle_history_pruned_count
                .saturating_add(counts.retained_history_pruned_count as u64);
            telemetry.resource_retained_denied_completion_count =
                self.denied_completions.len() as u64;
            telemetry.resource_retained_retry_lineage_count =
                self.retained_retry_lineage_by_ordinal.len() as u64;
            telemetry.resource_retained_history_unavailable_count = telemetry
                .resource_retained_history_unavailable_count
                .saturating_add(counts.retained_history_unavailable_count as u64);
            telemetry.resource_in_flight_request_count = self.in_flight_by_request.len() as u64;
        }
    }

    fn compaction_policy_provenance_digest(&self) -> String {
        let retained_history_decision_digests = self
            .pruned_in_flight_history_by_request
            .values()
            .map(|availability| availability.retention_decision_digest().as_str())
            .collect::<Vec<_>>();
        let retry_lineage_decision_digests = self
            .pruned_retry_lineage_by_ordinal
            .values()
            .map(|availability| availability.policy_decision_digest().as_str())
            .collect::<Vec<_>>();
        streaming_policy_digest(&ResourceRetentionCompactionPolicyProvenanceDigestBasis {
            schema_version: if self.expired_lifecycle_availability.is_empty()
                && self.expired_retry_availability.is_empty()
            {
                "worth.resource.retention-compaction-policy-provenance.v1"
            } else {
                "worth.resource.retention-compaction-policy-provenance.v2"
            },
            retained_history_decision_digests: &retained_history_decision_digests,
            retry_lineage_decision_digests: &retry_lineage_decision_digests,
            expired_lifecycle: self.expired_lifecycle_availability,
            expired_retry: self.expired_retry_availability,
        })
    }
}

fn streaming_policy_digest<T: Serialize>(value: &T) -> String {
    let mut writer = PolicyDigestWriter(Sha256::new());
    serde_json::to_writer(&mut writer, value).expect("retention policy provenance serialization");
    writer
        .0
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct PolicyDigestWriter(Sha256);

impl Write for PolicyDigestWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "compaction/streaming_digest_tests.rs"]
mod streaming_digest_tests;
