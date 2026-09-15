use crate::data::telemetry::TransactionTelemetry;

use super::SignalRuntime;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::logic::transaction::runtime::state) fn merge_global_transaction_telemetry(
        current: TransactionTelemetry,
        restored: &mut TransactionTelemetry,
    ) {
        restored.transaction_begin_count = restored
            .transaction_begin_count
            .max(current.transaction_begin_count);
        restored.transaction_commit_count = restored
            .transaction_commit_count
            .max(current.transaction_commit_count);
        restored.transaction_rollback_count = restored
            .transaction_rollback_count
            .max(current.transaction_rollback_count);
        restored.transaction_poison_count = restored
            .transaction_poison_count
            .max(current.transaction_poison_count);
        restored.rollback_packet_breadth = restored
            .rollback_packet_breadth
            .max(current.rollback_packet_breadth);
        restored.rollback_packet_config_count = restored
            .rollback_packet_config_count
            .max(current.rollback_packet_config_count);
        restored.rollback_packet_diagnostics_count = restored
            .rollback_packet_diagnostics_count
            .max(current.rollback_packet_diagnostics_count);
        restored.rollback_packet_graph_patch_count = restored
            .rollback_packet_graph_patch_count
            .max(current.rollback_packet_graph_patch_count);
        restored.rollback_packet_created_node_count = restored
            .rollback_packet_created_node_count
            .max(current.rollback_packet_created_node_count);
        restored.rollback_packet_subscriber_repair_count = restored
            .rollback_packet_subscriber_repair_count
            .max(current.rollback_packet_subscriber_repair_count);
        restored.rollback_packet_resource_count = restored
            .rollback_packet_resource_count
            .max(current.rollback_packet_resource_count);
        restored.rollback_packet_temporal_count = restored
            .rollback_packet_temporal_count
            .max(current.rollback_packet_temporal_count);
        restored.move_transfer_count = restored
            .move_transfer_count
            .max(current.move_transfer_count);
        restored.explicit_fork_count = restored
            .explicit_fork_count
            .max(current.explicit_fork_count);
        restored.explicit_snapshot_fork_count = restored
            .explicit_snapshot_fork_count
            .max(current.explicit_snapshot_fork_count);
        restored.explicit_fork_denial_count = restored
            .explicit_fork_denial_count
            .max(current.explicit_fork_denial_count);
        restored.restore_transfer_count = restored
            .restore_transfer_count
            .max(current.restore_transfer_count);
        restored.heavy_capture_count = restored
            .heavy_capture_count
            .max(current.heavy_capture_count);
        restored.branch_basis_production_count = restored
            .branch_basis_production_count
            .max(current.branch_basis_production_count);
        restored.branch_basis_validation_count = restored
            .branch_basis_validation_count
            .max(current.branch_basis_validation_count);
        restored.branch_basis_denial_count = restored
            .branch_basis_denial_count
            .max(current.branch_basis_denial_count);
        restored.branch_basis_stale_count = restored
            .branch_basis_stale_count
            .max(current.branch_basis_stale_count);
        restored.branch_retirement_plan_count = restored
            .branch_retirement_plan_count
            .max(current.branch_retirement_plan_count);
        restored.branch_retirement_execution_count = restored
            .branch_retirement_execution_count
            .max(current.branch_retirement_execution_count);
        restored.branch_retirement_denial_count = restored
            .branch_retirement_denial_count
            .max(current.branch_retirement_denial_count);
        restored.branch_retirement_reclaimed_branch_state_count = restored
            .branch_retirement_reclaimed_branch_state_count
            .max(current.branch_retirement_reclaimed_branch_state_count);
        restored.branch_retirement_reclaimed_snapshot_state_count = restored
            .branch_retirement_reclaimed_snapshot_state_count
            .max(current.branch_retirement_reclaimed_snapshot_state_count);
        restored.branch_retirement_reclaimed_runtime_meta_count = restored
            .branch_retirement_reclaimed_runtime_meta_count
            .max(current.branch_retirement_reclaimed_runtime_meta_count);
        restored.branch_retirement_retained_proof_count = restored
            .branch_retirement_retained_proof_count
            .max(current.branch_retirement_retained_proof_count);
        restored.branch_targeted_transaction_plan_count = restored
            .branch_targeted_transaction_plan_count
            .max(current.branch_targeted_transaction_plan_count);
        restored.branch_targeted_transaction_execution_count = restored
            .branch_targeted_transaction_execution_count
            .max(current.branch_targeted_transaction_execution_count);
        restored.branch_targeted_transaction_denial_count = restored
            .branch_targeted_transaction_denial_count
            .max(current.branch_targeted_transaction_denial_count);
        restored.branch_targeted_transaction_stale_count = restored
            .branch_targeted_transaction_stale_count
            .max(current.branch_targeted_transaction_stale_count);
        restored.branch_targeted_transaction_active_switch_avoided_count = restored
            .branch_targeted_transaction_active_switch_avoided_count
            .max(current.branch_targeted_transaction_active_switch_avoided_count);
        restored.branch_targeted_transaction_touched_node_count = restored
            .branch_targeted_transaction_touched_node_count
            .max(current.branch_targeted_transaction_touched_node_count);
        restored.branch_local_suppressed_observation_count = restored
            .branch_local_suppressed_observation_count
            .max(current.branch_local_suppressed_observation_count);
        restored.decision_log_event_count = restored
            .decision_log_event_count
            .max(current.decision_log_event_count);
        restored.staged_node_patch_count = restored
            .staged_node_patch_count
            .max(current.staged_node_patch_count);
        restored.max_touched_nodes_in_txn = restored
            .max_touched_nodes_in_txn
            .max(current.max_touched_nodes_in_txn);
        restored.transaction_mark_dirty_candidate_visits = restored
            .transaction_mark_dirty_candidate_visits
            .max(current.transaction_mark_dirty_candidate_visits);
        restored.staged_observation_candidate_count = restored
            .staged_observation_candidate_count
            .max(current.staged_observation_candidate_count);
        restored.staged_observation_match_count = restored
            .staged_observation_match_count
            .max(current.staged_observation_match_count);
        restored.observed_demand_reach_visits = restored
            .observed_demand_reach_visits
            .max(current.observed_demand_reach_visits);
        restored.observed_demand_targets = restored
            .observed_demand_targets
            .max(current.observed_demand_targets);
        restored.observed_demand_passes = restored
            .observed_demand_passes
            .max(current.observed_demand_passes);
        restored.classified_observation_count = restored
            .classified_observation_count
            .max(current.classified_observation_count);
        restored.observation_classification_breadth = restored
            .observation_classification_breadth
            .max(current.observation_classification_breadth);
        restored.delivered_observation_count = restored
            .delivered_observation_count
            .max(current.delivered_observation_count);
        restored.rollback_suppressed_observation_count = restored
            .rollback_suppressed_observation_count
            .max(current.rollback_suppressed_observation_count);
    }
}

#[cfg(test)]
mod tests {
    use crate::data::telemetry::TransactionTelemetry;

    use super::SignalRuntime;

    #[test]
    fn merge_global_transaction_telemetry_preserves_observation_counters() {
        let current = TransactionTelemetry {
            staged_observation_candidate_count: 11,
            staged_observation_match_count: 19,
            classified_observation_count: 7,
            observation_classification_breadth: 23,
            delivered_observation_count: 5,
            rollback_suppressed_observation_count: 3,
            observed_demand_reach_visits: 13,
            observed_demand_targets: 2,
            observed_demand_passes: 1,
            ..TransactionTelemetry::default()
        };
        let mut restored = TransactionTelemetry::default();

        SignalRuntime::<(), (), (), (), ()>::merge_global_transaction_telemetry(
            current,
            &mut restored,
        );

        assert_eq!(restored.staged_observation_candidate_count, 11);
        assert_eq!(restored.observed_demand_reach_visits, 13);
        assert_eq!(restored.observed_demand_targets, 2);
        assert_eq!(restored.observed_demand_passes, 1);
        assert_eq!(restored.staged_observation_match_count, 19);
        assert_eq!(restored.classified_observation_count, 7);
        assert_eq!(restored.observation_classification_breadth, 23);
        assert_eq!(restored.delivered_observation_count, 5);
        assert_eq!(restored.rollback_suppressed_observation_count, 3);
    }
}
