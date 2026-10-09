use crate::data::resource::descriptor::ResourceDescriptorId;
use serde::{Deserialize, Serialize};

use super::super::performance::ResourceBoundaryPerformanceEnvelope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ResourceRuntimeSummary {
    descriptor_count: u64,
    declared_resource_node_count: u64,
    in_flight_request_count: u64,
    active_in_flight_node_count: u64,
    retained_lifecycle_history_count: u64,
    retained_history_unavailable_count: u64,
    retained_retry_lineage_count: u64,
    retained_retry_lineage_unavailable_count: u64,
    denied_completion_count: u64,
    retained_denied_completion_count: u64,
    retained_denied_completion_unavailable_count: u64,
    next_descriptor_id: ResourceDescriptorId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRuntimeSummaryReadReport {
    summary: ResourceRuntimeSummary,
    performance: ResourceBoundaryPerformanceEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLifecycleRetentionCompactionReport {
    selected_terminal_count: u32,
    reclaimed_in_flight_count: u32,
    retained_history_write_count: u32,
    retained_history_pruned_count: u32,
    retained_history_unavailable_count: u32,
    retained_denied_completion_pruned_count: u32,
    retained_retry_lineage_pruned_count: u32,
    retained_history_width: u32,
    retained_denied_completion_width: u32,
    retained_retry_lineage_width: u32,
    hot_in_flight_width: u32,
    compacted_terminal_summary_count: u32,
    compacted_superseded_count: u32,
    compacted_cancelled_count: u32,
    compacted_timed_out_count: u32,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    expired_lifecycle_availability_count: u32,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    expired_denied_availability_count: u32,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    expired_retry_availability_count: u32,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    total_expired_lifecycle_availability: u64,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    total_expired_denied_availability: u64,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    total_expired_retry_availability: u64,
    policy_provenance_digest: String,
    performance: ResourceBoundaryPerformanceEnvelope,
}

fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}

fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

impl ResourceLifecycleRetentionCompactionReport {
    pub(crate) fn new(
        selected_terminal_count: u32,
        reclaimed_in_flight_count: u32,
        retained_history_write_count: u32,
        retained_history_pruned_count: u32,
        retained_history_unavailable_count: u32,
        retained_denied_completion_pruned_count: u32,
        retained_retry_lineage_pruned_count: u32,
        retained_history_width: u32,
        retained_denied_completion_width: u32,
        retained_retry_lineage_width: u32,
        hot_in_flight_width: u32,
        compacted_terminal_summary_count: u32,
        compacted_superseded_count: u32,
        compacted_cancelled_count: u32,
        compacted_timed_out_count: u32,
        expired_lifecycle_availability_count: u32,
        expired_denied_availability_count: u32,
        expired_retry_availability_count: u32,
        total_expired_lifecycle_availability: u64,
        total_expired_denied_availability: u64,
        total_expired_retry_availability: u64,
        policy_provenance_digest: String,
        performance: ResourceBoundaryPerformanceEnvelope,
    ) -> Self {
        Self {
            selected_terminal_count,
            reclaimed_in_flight_count,
            retained_history_write_count,
            retained_history_pruned_count,
            retained_history_unavailable_count,
            retained_denied_completion_pruned_count,
            retained_retry_lineage_pruned_count,
            retained_history_width,
            retained_denied_completion_width,
            retained_retry_lineage_width,
            hot_in_flight_width,
            compacted_terminal_summary_count,
            compacted_superseded_count,
            compacted_cancelled_count,
            compacted_timed_out_count,
            expired_lifecycle_availability_count,
            expired_denied_availability_count,
            expired_retry_availability_count,
            total_expired_lifecycle_availability,
            total_expired_denied_availability,
            total_expired_retry_availability,
            policy_provenance_digest,
            performance,
        }
    }

    pub fn selected_terminal_count(&self) -> u32 {
        self.selected_terminal_count
    }

    pub fn reclaimed_in_flight_count(&self) -> u32 {
        self.reclaimed_in_flight_count
    }

    pub fn retained_history_write_count(&self) -> u32 {
        self.retained_history_write_count
    }

    pub fn retained_history_pruned_count(&self) -> u32 {
        self.retained_history_pruned_count
    }

    pub fn retained_history_unavailable_count(&self) -> u32 {
        self.retained_history_unavailable_count
    }

    pub fn retained_denied_completion_pruned_count(&self) -> u32 {
        self.retained_denied_completion_pruned_count
    }

    pub fn retained_retry_lineage_pruned_count(&self) -> u32 {
        self.retained_retry_lineage_pruned_count
    }

    pub fn retained_history_width(&self) -> u32 {
        self.retained_history_width
    }

    pub fn retained_denied_completion_width(&self) -> u32 {
        self.retained_denied_completion_width
    }

    pub fn retained_retry_lineage_width(&self) -> u32 {
        self.retained_retry_lineage_width
    }

    pub fn hot_in_flight_width(&self) -> u32 {
        self.hot_in_flight_width
    }

    pub fn compacted_terminal_summary_count(&self) -> u32 {
        self.compacted_terminal_summary_count
    }

    pub fn compacted_superseded_count(&self) -> u32 {
        self.compacted_superseded_count
    }

    pub fn compacted_cancelled_count(&self) -> u32 {
        self.compacted_cancelled_count
    }

    pub fn compacted_timed_out_count(&self) -> u32 {
        self.compacted_timed_out_count
    }

    pub fn expired_lifecycle_availability_count(&self) -> u32 {
        self.expired_lifecycle_availability_count
    }

    pub fn expired_denied_availability_count(&self) -> u32 {
        self.expired_denied_availability_count
    }

    pub fn expired_retry_availability_count(&self) -> u32 {
        self.expired_retry_availability_count
    }

    pub fn total_expired_lifecycle_availability(&self) -> u64 {
        self.total_expired_lifecycle_availability
    }

    pub fn total_expired_denied_availability(&self) -> u64 {
        self.total_expired_denied_availability
    }

    pub fn total_expired_retry_availability(&self) -> u64 {
        self.total_expired_retry_availability
    }

    pub fn policy_provenance_digest(&self) -> &str {
        &self.policy_provenance_digest
    }

    pub fn performance(&self) -> ResourceBoundaryPerformanceEnvelope {
        self.performance
    }
}

impl ResourceRuntimeSummaryReadReport {
    pub(crate) fn new(
        summary: ResourceRuntimeSummary,
        performance: ResourceBoundaryPerformanceEnvelope,
    ) -> Self {
        Self {
            summary,
            performance,
        }
    }

    pub fn summary(self) -> ResourceRuntimeSummary {
        self.summary
    }

    pub fn performance(self) -> ResourceBoundaryPerformanceEnvelope {
        self.performance
    }
}

impl ResourceRuntimeSummary {
    pub(crate) fn new(
        descriptor_count: usize,
        declared_resource_node_count: usize,
        in_flight_request_count: usize,
        active_in_flight_node_count: usize,
        retained_lifecycle_history_count: usize,
        retained_history_unavailable_count: usize,
        denied_completion_count: usize,
        retained_retry_lineage_count: usize,
        retained_denied_completion_count: usize,
        retained_denied_completion_unavailable_count: usize,
        retained_retry_lineage_unavailable_count: usize,
        next_descriptor_id: ResourceDescriptorId,
    ) -> Self {
        Self {
            descriptor_count: descriptor_count as u64,
            declared_resource_node_count: declared_resource_node_count as u64,
            in_flight_request_count: in_flight_request_count as u64,
            active_in_flight_node_count: active_in_flight_node_count as u64,
            retained_lifecycle_history_count: retained_lifecycle_history_count as u64,
            retained_history_unavailable_count: retained_history_unavailable_count as u64,
            denied_completion_count: denied_completion_count as u64,
            retained_retry_lineage_count: retained_retry_lineage_count as u64,
            retained_denied_completion_count: retained_denied_completion_count as u64,
            retained_denied_completion_unavailable_count:
                retained_denied_completion_unavailable_count as u64,
            retained_retry_lineage_unavailable_count: retained_retry_lineage_unavailable_count
                as u64,
            next_descriptor_id,
        }
    }

    pub fn descriptor_count(self) -> u64 {
        self.descriptor_count
    }

    pub fn declared_resource_node_count(self) -> u64 {
        self.declared_resource_node_count
    }

    pub fn in_flight_request_count(self) -> u64 {
        self.in_flight_request_count
    }

    pub fn active_in_flight_node_count(self) -> u64 {
        self.active_in_flight_node_count
    }

    pub fn retained_lifecycle_history_count(self) -> u64 {
        self.retained_lifecycle_history_count
    }

    pub fn retained_history_unavailable_count(self) -> u64 {
        self.retained_history_unavailable_count
    }

    pub fn retained_retry_lineage_count(self) -> u64 {
        self.retained_retry_lineage_count
    }

    pub fn retained_retry_lineage_unavailable_count(self) -> u64 {
        self.retained_retry_lineage_unavailable_count
    }

    pub fn denied_completion_count(self) -> u64 {
        self.denied_completion_count
    }

    pub fn retained_denied_completion_count(self) -> u64 {
        self.retained_denied_completion_count
    }

    pub fn retained_denied_completion_unavailable_count(self) -> u64 {
        self.retained_denied_completion_unavailable_count
    }

    pub fn next_descriptor_id(self) -> ResourceDescriptorId {
        self.next_descriptor_id
    }
}
