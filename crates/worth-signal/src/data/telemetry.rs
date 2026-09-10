use serde::{Deserialize, Serialize};

use crate::data::resource::{ResourceBoundaryPerformanceEnvelope, ResourceDensityStrategy};

mod execution;
mod invalidation_performed;
mod transaction;

pub use super::telemetry_mutation::RuntimeTelemetryMutation;

pub use execution::{
    EvaluationTelemetry, ExecutionTelemetry, InvalidationTelemetry, PlannerTelemetry,
};
pub use invalidation_performed::{
    InvalidationPerformedCounter, SignalInvalidationRealizedCounters,
};
pub use transaction::TransactionTelemetry;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageTelemetry {
    pub gc_epoch_count: u64,
    pub gc_epoch_nanos: u128,
    pub graph_storage_compaction_count: u64,
    pub graph_storage_dependency_segments_rewritten: u64,
    pub graph_storage_subscriber_segments_rewritten: u64,
    pub graph_storage_snapshot_rewrites: u64,
    pub shared_snapshot_replacement_count: u64,
    pub version_only_snapshot_update_count: u64,
    pub stable_shape_snapshot_proof_count: u64,
    pub stable_shape_snapshot_proof_failure_count: u64,
    pub stable_shape_batch_commit_count: u64,
    pub structural_replace_batch_commit_count: u64,
    pub snapshot_shape_reuse_count: u64,
    pub snapshot_between_fallback_count: u64,
    pub snapshot_batch_size: u64,
    pub snapshot_batch_commit_nanos: u128,
    pub rolled_back_created_node_count: u64,
    pub subscriber_index_rebuild_count: u64,
    pub scratch_reentry_error_count: u64,
    pub hot_path_artifact_retention_count: u64,
    pub hot_write_runtime_artifact_count: u64,
    pub hot_write_cold_record_materialization_count: u64,
    pub hot_write_cold_bypass_count: u64,
    pub eager_cold_artifact_materialization_count: u64,
    pub deferred_cold_artifact_bypass_count: u64,
    pub hot_node_inline_size_bytes: u64,
    pub warm_node_inline_size_bytes: u64,
    pub definition_node_inline_size_bytes: u64,
    pub hot_runtime_artifact_inline_size_bytes: u64,
    pub warm_runtime_artifact_inline_size_bytes: u64,
    pub cold_artifact_record_inline_size_bytes: u64,
    pub hot_path_artifact_reconstruction_count: u64,
    pub explicit_cold_materialization_request_count: u64,
    pub retained_forensic_read_count: u64,
    pub cold_explanation_reconstruction_count: u64,
    pub cold_provenance_reconstruction_count: u64,
    pub retained_artifact_read_count: u64,
    pub reconstructed_artifact_read_count: u64,
    pub denied_reconstruction_by_budget_count: u64,
    pub denied_reconstruction_by_tier_count: u64,
    pub denied_reconstruction_explanation_api_count: u64,
    pub denied_reconstruction_provenance_api_count: u64,
    pub structural_delta_size: u64,
    pub patch_application_breadth: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointTelemetry {
    pub event_flushes: u64,
    pub event_flush_nanos: u128,
    pub checkpoint_flushes: u64,
    pub checkpoint_flush_nanos: u128,
    pub rollback_count: u64,
    pub snapshot_restore_count: u64,
    pub snapshot_restore_apply_active_policy_count: u64,
    pub snapshot_restore_shared_delta_node_count: u64,
    pub snapshot_restore_coarse_reason_count: u64,
    pub checkpoint_size: u64,
    pub journal_replay_span: u64,
    pub restore_authority_breadth: u64,
    pub restore_required_derived_breadth: u64,
    pub restore_diagnostic_richness_breadth: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalTelemetry {
    pub temporal_wake_count: u64,
    pub deferred_by_time_count: u64,
    pub scheduled_frontier_width: u64,
    pub ready_queue_width: u64,
    pub retired_wake_count: u64,
    pub rescheduled_wake_count: u64,
    pub interval_wake_regeneration_count: u64,
    pub missed_interval_count: u64,
    pub temporal_eligibility_lowering_count: u64,
    pub previous_value_reference_count: u64,
    pub branch_local_temporal_restore_count: u64,
    pub temporal_replay_parity_check_count: u64,
    pub temporal_broad_scan_denial_count: u64,
    pub wake_allocation_count: u64,
    pub wake_reuse_count: u64,
    pub branch_restore_temporal_rebuild_denial_count: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceTelemetry {
    pub async_node_capability_validation_count: u64,
    pub async_node_capability_freeze_count: u64,
    pub async_node_capability_bundle_lowering_count: u64,
    pub async_node_capability_attachment_count: u64,
    pub async_node_capability_alias_lowering_count: u64,
    pub async_node_family_binding_count: u64,
    pub async_node_condition_governed_admission_count: u64,
    pub async_node_previous_value_governed_admission_count: u64,
    pub async_node_condition_blocked_admission_count: u64,
    pub async_node_revalidation_eligibility_count: u64,
    pub async_node_aspect_local_refresh_count: u64,
    pub async_node_partition_local_refresh_count: u64,
    pub async_node_interior_gate_admission_count: u64,
    pub async_node_hierarchical_propagation_count: u64,
    pub async_node_historical_parity_count: u64,
    pub async_node_capability_equivalence_count: u64,
    pub async_keyed_node_historical_parity_count: u64,
    pub async_keyed_node_capability_equivalence_count: u64,
    pub async_node_hierarchy_historical_parity_count: u64,
    pub async_node_capability_broad_scan_denial_count: u64,
    pub resource_declaration_lowering_count: u64,
    pub resource_policy_resolution_count: u64,
    pub resource_policy_resolution_denial_count: u64,
    pub resource_policy_compatibility_count: u64,
    pub resource_policy_descriptor_comparison_count: u64,
    pub resource_policy_descriptor_incompatibility_count: u64,
    pub resource_replay_compatibility_decision_count: u64,
    pub resource_replay_compatible_count: u64,
    pub resource_replay_incompatible_count: u64,
    pub resource_replay_missing_policy_count: u64,
    pub resource_replay_availability_decision_count: u64,
    pub resource_replay_availability_retained_count: u64,
    pub resource_replay_availability_reconstructed_count: u64,
    pub resource_replay_availability_omitted_count: u64,
    pub resource_replay_availability_unavailable_count: u64,
    pub resource_replay_availability_denied_count: u64,
    pub resource_replay_budget_history_unavailable_count: u64,
    pub resource_retry_policy_decision_count: u64,
    pub resource_retry_jitter_decision_count: u64,
    pub resource_timeout_policy_decision_count: u64,
    pub resource_cancellation_policy_decision_count: u64,
    pub resource_supersession_policy_decision_count: u64,
    pub resource_deadline_inherited_count: u64,
    pub resource_progress_heartbeat_extension_count: u64,
    pub resource_runtime_hard_cancellation_count: u64,
    pub resource_host_cancellation_advisory_count: u64,
    pub resource_cancellation_grace_period_count: u64,
    pub resource_dependent_cancellation_propagation_count: u64,
    pub resource_overlapping_generation_admission_count: u64,
    pub resource_intent_equivalence_coalescing_count: u64,
    pub resource_old_host_work_retained_count: u64,
    pub resource_old_host_work_advisory_cancelled_count: u64,
    pub resource_request_admission_count: u64,
    pub resource_cancellation_count: u64,
    pub resource_rejection_admission_count: u64,
    pub resource_timeout_admission_count: u64,
    pub resource_retry_schedule_count: u64,
    pub resource_retry_admission_count: u64,
    pub resource_revalidation_admission_count: u64,
    pub resource_completion_validation_count: u64,
    pub resource_completion_admission_count: u64,
    pub resource_completion_batch_admission_count: u64,
    pub resource_completion_staging_count: u64,
    pub resource_completion_denial_staging_count: u64,
    pub resource_completion_commit_count: u64,
    pub resource_completion_rollback_count: u64,
    pub resource_descriptor_count: u64,
    pub resource_in_flight_request_count: u64,
    pub resource_in_flight_frontier_width: u64,
    pub resource_superseded_in_flight_count: u64,
    pub resource_supersession_record_count: u64,
    pub resource_supersession_lineage_width: u64,
    pub resource_branch_restore_count: u64,
    pub resource_branch_restore_in_flight_width: u64,
    pub resource_branch_restore_retained_summary_width: u64,
    pub resource_branch_restore_broad_rebuild_denial_count: u64,
    pub resource_replay_reconstruction_count: u64,
    pub resource_replay_reconstruction_lifecycle_width: u64,
    pub resource_replay_reconstruction_denial_width: u64,
    pub resource_replay_reconstruction_in_flight_width: u64,
    pub resource_retained_history_unavailable_count: u64,
    pub resource_hot_in_flight_compaction_count: u64,
    pub resource_in_flight_retired_record_count: u64,
    pub resource_in_flight_reclaimed_record_count: u64,
    pub resource_retained_lifecycle_history_write_count: u64,
    pub resource_retained_lifecycle_history_pruned_count: u64,
    pub resource_retained_denied_completion_count: u64,
    pub resource_retained_retry_lineage_count: u64,
    pub resource_retained_summary_read_count: u64,
    pub resource_diagnostics_policy_decision_count: u64,
    pub resource_diagnostics_expansion_count: u64,
    pub resource_diagnostics_expansion_input_width: u64,
    pub resource_diagnostics_cold_reconstruction_count: u64,
    pub resource_duplicate_declaration_denial_count: u64,
    pub resource_non_live_owner_denial_count: u64,
    pub resource_undeclared_owner_denial_count: u64,
    pub resource_cancellation_denial_count: u64,
    pub resource_stale_cancellation_denial_count: u64,
    pub resource_non_active_cancellation_denial_count: u64,
    pub resource_rejection_denial_count: u64,
    pub resource_stale_rejection_denial_count: u64,
    pub resource_non_active_rejection_denial_count: u64,
    pub resource_timeout_denial_count: u64,
    pub resource_timeout_heartbeat_extension_denial_count: u64,
    pub resource_timeout_heartbeat_policy_denial_count: u64,
    pub resource_stale_timeout_denial_count: u64,
    pub resource_non_active_timeout_denial_count: u64,
    pub resource_missing_timeout_wake_denial_count: u64,
    pub resource_timeout_wake_mismatch_denial_count: u64,
    pub resource_retry_denial_count: u64,
    pub resource_stale_retry_denial_count: u64,
    pub resource_retry_policy_disabled_denial_count: u64,
    pub resource_retry_attempt_limit_denial_count: u64,
    pub resource_retry_budget_exhaustion_denial_count: u64,
    pub resource_retry_timeout_window_exhaustion_denial_count: u64,
    pub resource_non_retryable_denial_count: u64,
    pub resource_retry_already_scheduled_denial_count: u64,
    pub resource_retry_wake_mismatch_denial_count: u64,
    pub resource_retry_superseded_denial_count: u64,
    pub resource_revalidation_denial_count: u64,
    pub resource_revalidation_expected_mismatch_denial_count: u64,
    pub resource_revalidation_active_requires_expected_denial_count: u64,
    pub resource_revalidation_policy_decision_count: u64,
    pub resource_forced_revalidation_count: u64,
    pub resource_stale_after_revalidation_count: u64,
    pub resource_dependency_change_revalidation_count: u64,
    pub resource_observer_demand_revalidation_count: u64,
    pub resource_terminal_state_revalidation_count: u64,
    pub resource_fulfilled_lifecycle_revalidation_count: u64,
    pub resource_revalidation_coalesced_count: u64,
    pub resource_observation_policy_decision_count: u64,
    pub resource_observation_candidate_width: u64,
    pub resource_observation_coalesced_width: u64,
    pub resource_observation_delivered_width: u64,
    pub resource_denied_completion_observation_count: u64,
    pub resource_retry_schedule_observation_count: u64,
    pub resource_output_continuity_decision_count: u64,
    pub resource_output_continuity_classification_width: u64,
    pub resource_previous_output_preserved_count: u64,
    pub resource_previous_output_hidden_count: u64,
    pub resource_host_failure_rejection_count: u64,
    pub resource_semantic_rejection_count: u64,
    pub resource_forced_revalidation_policy_denial_count: u64,
    pub resource_revalidation_dependency_change_policy_denial_count: u64,
    pub resource_revalidation_dependency_change_proof_mismatch_denial_count: u64,
    pub resource_revalidation_observer_demand_policy_denial_count: u64,
    pub resource_revalidation_observer_demand_proof_mismatch_denial_count: u64,
    pub resource_revalidation_terminal_state_policy_denial_count: u64,
    pub resource_revalidation_terminal_state_proof_mismatch_denial_count: u64,
    pub resource_revalidation_fulfilled_lifecycle_policy_denial_count: u64,
    pub resource_revalidation_fulfilled_lifecycle_proof_mismatch_denial_count: u64,
    pub resource_revalidation_stale_after_policy_denial_count: u64,
    pub resource_revalidation_stale_after_wake_mismatch_denial_count: u64,
    pub resource_revalidation_stale_after_fulfilled_only_denial_count: u64,
    pub resource_revalidation_active_handle_proof_mismatch_denial_count: u64,
    pub resource_revalidation_active_handle_proof_check_count: u64,
    pub resource_revalidation_dependency_change_proof_check_count: u64,
    pub resource_revalidation_observer_demand_proof_check_count: u64,
    pub resource_revalidation_terminal_state_proof_check_count: u64,
    pub resource_revalidation_fulfilled_lifecycle_proof_check_count: u64,
    pub resource_completion_denial_count: u64,
    pub resource_stale_completion_denial_count: u64,
    pub resource_superseded_completion_denial_count: u64,
    pub resource_malformed_completion_denial_count: u64,
    pub resource_partial_completion_denial_count: u64,
    pub resource_contradictory_completion_denial_count: u64,
    pub resource_duplicate_completion_denial_count: u64,
    pub resource_unknown_request_completion_denial_count: u64,
    pub resource_retained_history_unavailable_completion_denial_count: u64,
    pub resource_cancelled_completion_denial_count: u64,
    pub resource_rejected_completion_denial_count: u64,
    pub resource_timed_out_completion_denial_count: u64,
    pub resource_timeout_temporal_wake_footprint: u64,
    pub resource_retry_temporal_wake_footprint: u64,
    pub resource_boundary_performance_envelope_count: u64,
    pub resource_retry_budget_scope_touch_count: u64,
    pub resource_broad_scan_denial_count: u64,
    pub resource_hot_in_flight_lookup_count: u64,
    pub resource_operational_allocation_count: u64,
    pub resource_retained_history_allocation_count: u64,
    pub resource_diagnostics_allocation_count: u64,
    pub resource_facade_report_allocation_count: u64,
    pub resource_density_strategy_selection_count: u64,
    pub resource_sparse_density_strategy_count: u64,
    pub resource_bursty_density_strategy_count: u64,
    pub resource_dense_density_strategy_count: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostComputedTelemetry {
    pub descriptor_registration_count: u64,
    pub evaluation_request_admission_count: u64,
    pub evaluation_request_denial_count: u64,
    pub read_set_admission_count: u64,
    pub dependency_patch_count: u64,
    pub committed_artifact_count: u64,
    pub denied_outcome_count: u64,
    pub failed_outcome_count: u64,
    pub self_read_denial_count: u64,
    pub dependency_patch_added_count: u64,
    pub dependency_patch_removed_count: u64,
    pub dependency_patch_retained_count: u64,
    pub dependency_patch_touched_subscriber_index_count: u64,
}

impl ResourceTelemetry {
    pub fn record_boundary_performance_envelope(
        &mut self,
        envelope: ResourceBoundaryPerformanceEnvelope,
    ) {
        self.record_boundary_allocation_posture(envelope);
        self.record_density_strategy(envelope.density_strategy());
        self.resource_boundary_performance_envelope_count += 1;
        self.resource_retry_budget_scope_touch_count = self
            .resource_retry_budget_scope_touch_count
            .saturating_add(envelope.retry_budget_scope_touch_count() as u64);
        self.resource_output_continuity_classification_width = self
            .resource_output_continuity_classification_width
            .saturating_add(envelope.output_continuity_classification_width() as u64);
    }

    fn record_density_strategy(&mut self, density_strategy: ResourceDensityStrategy) {
        match density_strategy {
            ResourceDensityStrategy::NotApplicable => {}
            ResourceDensityStrategy::SparseIndexedLookup => {
                self.resource_density_strategy_selection_count += 1;
                self.resource_sparse_density_strategy_count += 1;
            }
            ResourceDensityStrategy::BurstySortedDeduplicated => {
                self.resource_density_strategy_selection_count += 1;
                self.resource_bursty_density_strategy_count += 1;
            }
            ResourceDensityStrategy::DenseSortedDeduplicated => {
                self.resource_density_strategy_selection_count += 1;
                self.resource_dense_density_strategy_count += 1;
            }
        }
    }

    pub fn record_boundary_allocation_posture(
        &mut self,
        envelope: ResourceBoundaryPerformanceEnvelope,
    ) {
        self.resource_operational_allocation_count = self
            .resource_operational_allocation_count
            .saturating_add(envelope.operational_allocation_count() as u64);
        self.resource_retained_history_allocation_count = self
            .resource_retained_history_allocation_count
            .saturating_add(envelope.retained_history_allocation_count() as u64);
        self.resource_diagnostics_allocation_count = self
            .resource_diagnostics_allocation_count
            .saturating_add(envelope.diagnostics_allocation_count() as u64);
        self.resource_facade_report_allocation_count = self
            .resource_facade_report_allocation_count
            .saturating_add(envelope.facade_report_allocation_count() as u64);
    }
}

/// Lightweight runtime telemetry for signal orchestration internals.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTelemetry {
    pub evaluation: EvaluationTelemetry,
    pub invalidation: InvalidationTelemetry,
    pub transaction: TransactionTelemetry,
    pub planner: PlannerTelemetry,
    pub execution: ExecutionTelemetry,
    pub storage: StorageTelemetry,
    pub checkpoint: CheckpointTelemetry,
    pub temporal: TemporalTelemetry,
    pub resource: ResourceTelemetry,
    pub host_computed: HostComputedTelemetry,
}
