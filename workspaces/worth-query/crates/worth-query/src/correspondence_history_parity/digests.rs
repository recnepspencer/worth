use crate::correspondence::CorrespondenceEvidenceResolved;
use crate::historical::{
    HistoricalCounterSnapshot, HistoricalEvaluationError, HistoricalPathCostPosture,
    PerformancePredictionDriftOutcome,
};
use crate::identity::{
    CorrespondenceCostPostureDigest, CorrespondenceOutcomeDigest, CounterSnapshotDigest,
    FailureDigest, HistoricalCostPostureDigest, HistoricalPathClassDigest,
};

pub(crate) fn digest_correspondence_outcome(
    correspondence: &CorrespondenceEvidenceResolved,
) -> CorrespondenceOutcomeDigest {
    let outcome = correspondence.outcome();
    let mut parts = vec![format!("family:{}", outcome.family_name())];

    if let Some(lineage) = outcome.as_lineage_continuity() {
        parts.push(format!("canonical_subject:{}", lineage.canonical_subject()));
        parts.push(format!(
            "authoritative_counterpart:{}",
            lineage.authoritative_counterpart()
        ));
    }

    if let Some(unique) = outcome.as_advisory_structural_unique() {
        parts.push(format!(
            "advisory_candidate:{}",
            unique.advisory_candidate()
        ));
        parts.push("uniqueness_witness:present".to_string());
    }

    if let Some(ambiguous) = outcome.as_advisory_structural_ambiguous() {
        parts.push(format!(
            "candidate_count:{}",
            ambiguous.candidate_set().len()
        ));
        parts.extend(
            ambiguous
                .candidate_set()
                .candidates()
                .iter()
                .map(|candidate| format!("candidate:{candidate}")),
        );
    }

    if let Some(disagreement) = outcome.as_lineage_structural_disagreement() {
        parts.push(format!(
            "lineage_counterpart:{}",
            disagreement.lineage_counterpart()
        ));
        parts.push(format!(
            "structural_counterpart:{}",
            disagreement.structural_counterpart()
        ));
    }

    if let Some(denied) = outcome.as_denied() {
        parts.push(format!("denied_reason:{}", denied.reason()));
        parts.push(format!(
            "denied_cost_posture:{}",
            denied.cost_posture().as_str()
        ));
    }

    CorrespondenceOutcomeDigest::from_parts(&parts)
}

pub(crate) fn digest_requested_path(value: &str) -> HistoricalPathClassDigest {
    HistoricalPathClassDigest::from_parts(&[format!("requested_path:{value}")])
}

pub(crate) fn digest_admitted_path(value: &str) -> HistoricalPathClassDigest {
    HistoricalPathClassDigest::from_parts(&[format!("admitted_path:{value}")])
}

pub(crate) fn digest_resolved_path(value: &str) -> HistoricalPathClassDigest {
    HistoricalPathClassDigest::from_parts(&[format!("resolved_path:{value}")])
}

pub(crate) fn digest_correspondence_cost_posture(value: &str) -> CorrespondenceCostPostureDigest {
    CorrespondenceCostPostureDigest::from_parts(&[format!("correspondence_cost_posture:{value}")])
}

pub(crate) fn digest_historical_cost_posture(
    value: &HistoricalPathCostPosture,
) -> HistoricalCostPostureDigest {
    HistoricalCostPostureDigest::from_parts(&[format!(
        "historical_cost_posture:{}",
        value.as_str()
    )])
}

pub(crate) fn digest_counter_snapshot(
    correspondence: &CorrespondenceEvidenceResolved,
    historical: Option<&HistoricalCounterSnapshot>,
) -> CounterSnapshotDigest {
    let counters = correspondence.counters();
    let mut parts = vec![
        format!(
            "correspondence.predicted_structural_candidate_count:{}",
            counters.predicted_structural_candidate_count()
        ),
        format!(
            "correspondence.structural_candidate_count:{}",
            counters.structural_candidate_count()
        ),
        format!(
            "correspondence.structural_candidate_rejection_count:{}",
            counters.structural_candidate_rejection_count()
        ),
        format!(
            "correspondence.structural_ambiguity_count:{}",
            counters.structural_ambiguity_count()
        ),
        format!(
            "correspondence.structural_unique_witness_count:{}",
            counters.structural_unique_witness_count()
        ),
        format!(
            "correspondence.lineage_structural_disagreement_count:{}",
            counters.lineage_structural_disagreement_count()
        ),
        format!(
            "correspondence.structural_authority_promotion_denial_count:{}",
            counters.structural_authority_promotion_denial_count()
        ),
        format!(
            "correspondence.predicted_correspondence_resolution_width:{}",
            counters.predicted_correspondence_resolution_width()
        ),
        format!(
            "correspondence.structural_candidate_prediction_drift_count:{}",
            counters.structural_candidate_prediction_drift_count()
        ),
        format!(
            "correspondence.executor_rediscovery_count:{}",
            counters.correspondence_executor_rediscovery_count()
        ),
    ];

    if let Some(historical) = historical {
        parts.extend([
            format!(
                "historical.requested_path_count:{}",
                historical.historical_requested_path_count()
            ),
            format!(
                "historical.admitted_path_count:{}",
                historical.historical_admitted_path_count()
            ),
            format!(
                "historical.resolved_path_count:{}",
                historical.historical_resolved_path_count()
            ),
            format!(
                "historical.compatibility_check_count:{}",
                historical.historical_compatibility_check_count()
            ),
            format!(
                "historical.predicted_replay_span:{}",
                historical.predicted_historical_replay_span()
            ),
            format!(
                "historical.predicted_reconstruction_scope:{}",
                historical.predicted_historical_reconstruction_scope()
            ),
            format!(
                "historical.retained_snapshot_admission_count:{}",
                historical.historical_retained_snapshot_admission_count()
            ),
            format!(
                "historical.delta_replay_admission_count:{}",
                historical.historical_delta_replay_admission_count()
            ),
            format!(
                "historical.full_reconstruction_admission_count:{}",
                historical.historical_full_reconstruction_admission_count()
            ),
            format!(
                "historical.path_denial_count:{}",
                historical.historical_path_denial_count()
            ),
            format!(
                "historical.hidden_path_substitution_denial_count:{}",
                historical.historical_hidden_path_substitution_denial_count()
            ),
            format!(
                "historical.result_path_metadata_count:{}",
                historical.historical_result_path_metadata_count()
            ),
            format!(
                "historical.replay_span_drift_count:{}",
                historical.historical_replay_span_drift_count()
            ),
            format!(
                "historical.reconstruction_scope_drift_count:{}",
                historical.historical_reconstruction_scope_drift_count()
            ),
            format!(
                "historical.work_avoided_by_retained_path_count:{}",
                historical.history_work_avoided_by_retained_path_count()
            ),
            format!(
                "historical.executor_rediscovery_count:{}",
                historical.historical_executor_rediscovery_count()
            ),
        ]);
    } else {
        parts.push("historical:absent".to_string());
    }

    CounterSnapshotDigest::from_parts(&parts)
}

pub(crate) fn infer_prediction_drift(
    correspondence: &CorrespondenceEvidenceResolved,
    historical: Option<&HistoricalCounterSnapshot>,
) -> PerformancePredictionDriftOutcome {
    if correspondence
        .counters()
        .structural_candidate_prediction_drift_count()
        > 0
    {
        return PerformancePredictionDriftOutcome::StructuralCandidatePredictionDrift;
    }

    if let Some(historical) = historical {
        if historical.historical_replay_span_drift_count() > 0 {
            return PerformancePredictionDriftOutcome::HistoricalReplaySpanDrift;
        }

        if historical.historical_reconstruction_scope_drift_count() > 0 {
            return PerformancePredictionDriftOutcome::HistoricalReconstructionScopeDrift;
        }
    }

    PerformancePredictionDriftOutcome::WithinBudget
}

pub(crate) fn digest_historical_failure(error: &HistoricalEvaluationError) -> FailureDigest {
    let mut parts = vec![
        format!("failure_class:{:?}", error.failure_class()),
        format!(
            "requested_path:{}",
            error
                .requested_path_class()
                .map(|path| path.as_str())
                .unwrap_or("not-admitted")
        ),
        format!("reason:{}", error.reason()),
    ];

    if let Some(admitted) = error.admitted_path_class() {
        parts.push(format!("admitted_path:{}", admitted.as_str()));
    }

    if let Some(resolved) = error.attempted_resolved_path_class() {
        parts.push(format!("attempted_resolved_path:{}", resolved.as_str()));
    }

    FailureDigest::from_parts(&parts)
}
