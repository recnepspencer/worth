use crate::diagnostics::history::{
    BridgeHistoricalEvaluationCounters, BridgeHistoricalEvaluationFailureClass,
    BridgeHistoricalEvaluationFailureIdentity, BridgeHistoricalEvaluationFailureRecord,
};
use crate::diagnostics::BridgeDiagnosticsFacade;
use crate::identity::{BridgeIdentity, BridgeIdentityEvidence, CheckpointTokenIdentityTag};
use crate::routing::{
    BridgeBulkPlanningCounters, BridgeBulkPlanningFailure, BridgeCanonicalBulkPlanRecord,
    BridgePreparationMode, BridgeWorkloadIdentity,
};
use crate::stream::{
    ConsumerCheckpointToken, StreamCheckpointFrontierKind, StreamProtocolCounters,
};

use super::super::digest_basis::{
    compose_retained_causal_mapping_evidence_identity,
    compose_retained_causal_mapping_evidence_identity_for_basis,
    retained_mapping_bridge_identity_part, retained_mapping_counter_part,
    retained_mapping_evidence_part, retained_mapping_shape_part,
    RetainedCausalMappingDigestArtifact, RetainedCausalMappingDigestBasis,
};

type StreamCheckpointTokenIdentity = BridgeIdentity<CheckpointTokenIdentityTag>;

pub(crate) fn bulk_planning_record_digest(
    facade: &BridgeDiagnosticsFacade,
    reference_identity: &BridgeIdentityEvidence,
) -> Option<BridgeIdentityEvidence> {
    facade
        .bulk_record_for_workload_identity(&BridgeWorkloadIdentity::from_reference_evidence(
            reference_identity.revalidate_bridge_retained_reference(),
        ))
        .map(|record| bulk_planning_digest(&record))
}

pub(crate) fn historical_evaluation_failure_record_digest(
    facade: &BridgeDiagnosticsFacade,
    reference_identity: &BridgeIdentityEvidence,
) -> Option<BridgeIdentityEvidence> {
    facade
        .historical_failure_for_identity(
            &BridgeHistoricalEvaluationFailureIdentity::from_reference_evidence(
                reference_identity.revalidate_bridge_retained_reference(),
            ),
        )
        .map(|record| historical_evaluation_failure_digest(&record))
}

pub(crate) fn stream_checkpoint_record_digest(
    facade: &BridgeDiagnosticsFacade,
    reference_identity: &BridgeIdentityEvidence,
) -> Option<BridgeIdentityEvidence> {
    facade
        .stream_checkpoint_for_identity(&StreamCheckpointTokenIdentity::from_reference_evidence(
            reference_identity.revalidate_bridge_retained_reference(),
        ))
        .map(|record| stream_checkpoint_digest(&record))
}

pub(crate) fn bulk_planning_digest(
    record: &BridgeCanonicalBulkPlanRecord,
) -> BridgeIdentityEvidence {
    let planning_failures_digest = bulk_planning_failures_digest(record.planning_failures());
    let counters_digest = bulk_planning_counters_digest(record.counters());
    compose_retained_causal_mapping_evidence_identity(
        RetainedCausalMappingDigestArtifact::BulkPlanningRecord,
        &[
            retained_mapping_bridge_identity_part(record.workload_identity()),
            retained_mapping_shape_part(record.schema_version()),
            retained_mapping_bridge_identity_part(record.canonical_planning_identity()),
            retained_mapping_bridge_identity_part(record.admission_profile_identity()),
            retained_mapping_shape_part(preparation_mode_label(record.selected_mode())),
            retained_mapping_evidence_part(counters_digest),
            retained_mapping_counter_part(record.planning_failure_count()),
            retained_mapping_evidence_part(planning_failures_digest),
        ],
    )
}

fn preparation_mode_label(value: BridgePreparationMode) -> &'static str {
    match value {
        BridgePreparationMode::Serial => "serial",
        BridgePreparationMode::ParallelPreparation => "parallel-preparation",
    }
}

pub(crate) fn historical_evaluation_failure_digest(
    record: &BridgeHistoricalEvaluationFailureRecord,
) -> BridgeIdentityEvidence {
    let commit_identity = record
        .commit_identity()
        .map(retained_mapping_bridge_identity_part)
        .unwrap_or_else(|| retained_mapping_shape_part("none"));
    let snapshot_identity = record
        .snapshot_identity()
        .map(retained_mapping_bridge_identity_part)
        .unwrap_or_else(|| retained_mapping_shape_part("none"));
    let counters_digest = historical_evaluation_counters_digest(record.counters());
    compose_retained_causal_mapping_evidence_identity(
        RetainedCausalMappingDigestArtifact::HistoricalEvaluationFailureRecord,
        &[
            retained_mapping_bridge_identity_part(record.failure_identity()),
            retained_mapping_bridge_identity_part(record.declaration_identity()),
            retained_mapping_bridge_identity_part(record.selector_identity()),
            retained_mapping_bridge_identity_part(record.branch_identity()),
            commit_identity,
            snapshot_identity,
            retained_mapping_shape_part(historical_failure_class_label(record.failure_class())),
            retained_mapping_shape_part(record.detail()),
            retained_mapping_evidence_part(counters_digest),
        ],
    )
}

fn historical_failure_class_label(value: BridgeHistoricalEvaluationFailureClass) -> &'static str {
    match value {
        BridgeHistoricalEvaluationFailureClass::ExecutionDenied(denial) => denial.label(),
        BridgeHistoricalEvaluationFailureClass::UnsupportedTruthViewSelector => {
            "unsupported-truth-view-selector"
        }
        BridgeHistoricalEvaluationFailureClass::TruthViewUnavailable => "truth-view-unavailable",
        BridgeHistoricalEvaluationFailureClass::RejectedBranchMismatch => {
            "rejected-branch-mismatch"
        }
        BridgeHistoricalEvaluationFailureClass::RejectedSnapshotMismatch => {
            "rejected-snapshot-mismatch"
        }
        BridgeHistoricalEvaluationFailureClass::RejectedHistoricalResolutionFailure => {
            "rejected-historical-resolution-failure"
        }
        BridgeHistoricalEvaluationFailureClass::HistoricalReplayMismatch => {
            "historical-replay-mismatch"
        }
        BridgeHistoricalEvaluationFailureClass::UnresolvedTruthViewPolicyConflict => {
            "unresolved-truth-view-policy-conflict"
        }
    }
}

pub(crate) fn stream_checkpoint_digest(record: &ConsumerCheckpointToken) -> BridgeIdentityEvidence {
    let counters_digest = stream_protocol_counters_digest(record.counters());
    compose_retained_causal_mapping_evidence_identity(
        RetainedCausalMappingDigestArtifact::StreamCheckpointRecord,
        &[
            retained_mapping_bridge_identity_part(record.checkpoint_token_identity()),
            retained_mapping_bridge_identity_part(record.consumer_contract_identity()),
            retained_mapping_bridge_identity_part(record.stream_protocol_identity()),
            retained_mapping_shape_part(checkpoint_frontier_kind_label(
                record.checkpoint_frontier_kind(),
            )),
            retained_mapping_shape_part(record.contiguous_acknowledged_through_position()),
            retained_mapping_counter_part(record.checkpoint_member_count()),
            retained_mapping_shape_part(record.protocol_semantics_version()),
            retained_mapping_evidence_part(counters_digest),
        ],
    )
}

fn historical_evaluation_counters_digest(
    counters: &BridgeHistoricalEvaluationCounters,
) -> BridgeIdentityEvidence {
    let counter_basis = RetainedCausalMappingDigestBasis::from_counter_usizes([
        counters.truth_view_selector_count(),
        counters.historical_truth_view_count(),
        counters.branch_truth_view_count(),
        counters.planned_truth_view_packet_count(),
        counters.resolved_truth_view_policy_count(),
        counters.materialized_truth_view_count(),
        counters.truth_view_unavailable_count(),
        counters.truth_view_branch_mismatch_count(),
        counters.truth_view_snapshot_mismatch_count(),
        counters.historical_replay_mismatch_count(),
        counters.branch_local_evaluation_count(),
        counters.truth_view_decision_log_count(),
        counters.selector_width(),
        counters.branch_width(),
        counters.direct_snapshot_materialization_count(),
        counters.commit_envelope_materialization_count(),
        counters.branch_head_materialization_count(),
    ]);
    compose_retained_causal_mapping_evidence_identity_for_basis(
        RetainedCausalMappingDigestArtifact::HistoricalEvaluationCounters,
        &counter_basis,
    )
}

fn bulk_planning_counters_digest(counters: &BridgeBulkPlanningCounters) -> BridgeIdentityEvidence {
    let counter_basis = RetainedCausalMappingDigestBasis::from_counter_usizes([
        counters.bulk_workload_count(),
        counters.bulk_routed_item_count(),
        counters.bulk_normalized_workload_width(),
        counters.bulk_packet_count(),
        counters.bulk_packet_entry_count(),
        counters.bulk_reduction_input_count(),
        counters.bulk_reduction_output_count(),
        counters.bulk_widening_count(),
        counters.bulk_packet_queue_depth_peak(),
        counters.bulk_reducer_input_buffer_peak(),
        counters.bulk_replay_mismatch_count(),
        counters.bulk_unsupported_path_count(),
        counters.bulk_serial_required_count(),
        counters.bulk_parallel_legal_count(),
        counters.bulk_parallel_profitable_count(),
        counters.bulk_parallel_preparation_admitted_count(),
        counters.bulk_parallel_preparation_rejected_count(),
        counters.bulk_parallel_serial_reduction_count(),
    ]);
    compose_retained_causal_mapping_evidence_identity_for_basis(
        RetainedCausalMappingDigestArtifact::BulkPlanningCounters,
        &counter_basis,
    )
}

fn bulk_planning_failures_digest(failures: &[BridgeBulkPlanningFailure]) -> BridgeIdentityEvidence {
    let failure_basis =
        RetainedCausalMappingDigestBasis::from_bulk_planning_failure_records(failures);
    compose_retained_causal_mapping_evidence_identity_for_basis(
        RetainedCausalMappingDigestArtifact::BulkPlanningFailures,
        &failure_basis,
    )
}

fn stream_protocol_counters_digest(counters: &StreamProtocolCounters) -> BridgeIdentityEvidence {
    let counter_basis = RetainedCausalMappingDigestBasis::from_counter_usizes([
        counters.stream_member_count(),
        counters.stream_window_count(),
        counters.stream_window_member_count(),
        counters.stream_consumer_contract_count(),
        counters.stream_checkpoint_count(),
        counters.stream_checkpoint_member_count(),
        counters.stream_resume_attempt_count(),
        counters.stream_resume_rejection_count(),
        counters.stream_replay_count(),
        counters.stream_replay_mismatch_count(),
        counters.stream_coalesced_member_count(),
        counters.stream_coalesced_window_count(),
        counters.stream_duplicate_member_observation_count(),
        counters.stream_backpressure_signal_count(),
        counters.stream_consumer_saturated_count(),
        counters.stream_checkpoint_lag_count(),
        counters.stream_protocol_mismatch_count(),
    ]);
    compose_retained_causal_mapping_evidence_identity_for_basis(
        RetainedCausalMappingDigestArtifact::StreamProtocolCounters,
        &counter_basis,
    )
}

fn checkpoint_frontier_kind_label(value: StreamCheckpointFrontierKind) -> &'static str {
    match value {
        StreamCheckpointFrontierKind::ContiguousFrontier => "contiguous-frontier",
        StreamCheckpointFrontierKind::ContiguousFrontierWithObservedDuplicates => {
            "contiguous-frontier-with-observed-duplicates"
        }
    }
}
