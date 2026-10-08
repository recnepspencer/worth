//! Complete provider read-set owner evidence at the application commit boundary.

use super::{
    request_authority::DenialCause, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationCommitDenialKind as CommitKind,
    WorthQueryApplicationCommitDenialStage as Stage,
};
use crate::domain_computation::{
    WorthQueryDecisionReadSetDenialKind as Kind, WorthQueryDecisionReadSetFailure,
};

impl WorthQueryApplicationCommitDenial {
    /// Full read-set refusal, including any original physical allocation cause.
    pub fn decision_read_set_failure(&self) -> Option<&WorthQueryDecisionReadSetFailure> {
        match &self.cause {
            Some(DenialCause::DecisionReadSet(failure)) => Some(failure),
            _ => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn decision_read_set_denied(
        failure: WorthQueryDecisionReadSetFailure,
    ) -> Self {
        let kind = match failure.kind() {
            Kind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => CommitKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            Kind::RetentionCapacityExhausted => CommitKind::RetentionCapacityExhausted,
            _ => CommitKind::ProviderRejected,
        };
        Self {
            kind,
            stage: Stage::DecisionReadSet,
            detail: Some(failure.detail().into()),
            cause: Some(DenialCause::DecisionReadSet(failure)),
        }
    }
}
