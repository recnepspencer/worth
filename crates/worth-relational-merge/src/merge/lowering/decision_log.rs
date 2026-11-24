use std::sync::Arc;

use crate::merge::data::{
    LoweredMergePlanRecord, MergeExecutionReadiness, MergePlanningDecisionKind,
    MergePlanningDecisionLog, MergePlanningDecisionLogDigestBasis,
};

pub(super) fn build_decision_log(
    lowered_records: &[LoweredMergePlanRecord],
) -> MergePlanningDecisionLog {
    let mut decisions = lowered_records
        .iter()
        .map(|record| crate::merge::data::MergePlanningDecisionRecord {
            record: record.record.clone(),
            target_record: record.target_record.clone(),
            decision: match record.readiness {
                MergeExecutionReadiness::Admitted => MergePlanningDecisionKind::Admitted,
                MergeExecutionReadiness::Blocked => MergePlanningDecisionKind::Blocked,
                MergeExecutionReadiness::Rejected => MergePlanningDecisionKind::Rejected,
            },
            classification: record.classification,
            causal_disposition: record.causal_disposition,
            policy_proof_boundary: record.policy_proof_boundary,
        })
        .collect::<Vec<_>>();
    decisions.sort_by(|left, right| {
        left.decision
            .cmp(&right.decision)
            .then(left.record.cmp(&right.record))
            .then(left.target_record.cmp(&right.target_record))
    });
    MergePlanningDecisionLog {
        decisions: Arc::from(decisions),
    }
}

pub(super) fn build_decision_log_digest_basis(
    decision_log: &MergePlanningDecisionLog,
) -> MergePlanningDecisionLogDigestBasis {
    MergePlanningDecisionLogDigestBasis {
        canonical_decisions: Arc::from(
            decision_log
                .decisions
                .iter()
                .map(|decision| decision.decision)
                .collect::<Vec<_>>(),
        ),
        canonical_records: Arc::from(
            decision_log
                .decisions
                .iter()
                .map(|decision| decision.record.clone())
                .collect::<Vec<_>>(),
        ),
        canonical_target_records: Arc::from(
            decision_log
                .decisions
                .iter()
                .map(|decision| decision.target_record.clone())
                .collect::<Vec<_>>(),
        ),
        canonical_classifications: Arc::from(
            decision_log
                .decisions
                .iter()
                .map(|decision| decision.classification)
                .collect::<Vec<_>>(),
        ),
        canonical_causal_dispositions: Arc::from(
            decision_log
                .decisions
                .iter()
                .map(|decision| decision.causal_disposition)
                .collect::<Vec<_>>(),
        ),
        canonical_policy_proof_boundaries: Arc::from(
            decision_log
                .decisions
                .iter()
                .map(|decision| decision.policy_proof_boundary)
                .collect::<Vec<_>>(),
        ),
    }
}
