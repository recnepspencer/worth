//! Complete provider read-set owner evidence at the application commit boundary.

use super::{
    denial_cause::DenialCause, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationCommitDenialKind as CommitKind,
    WorthQueryApplicationCommitDenialStage as Stage,
};
use crate::domain_computation::{
    WorthQueryDecisionReadSetDenialKind as Kind, WorthQueryDecisionReadSetFailure,
};

impl WorthQueryApplicationCommitDenial {
    /// Full read-set refusal, including any original physical allocation cause.
    pub fn decision_read_set_failure(&self) -> Option<&WorthQueryDecisionReadSetFailure> {
        match self.cause.as_deref() {
            Some(DenialCause::DecisionReadSet(failure)) => Some(failure),
            _ => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn decision_read_set_denied(
        failure: WorthQueryDecisionReadSetFailure,
    ) -> Self {
        Self::decision_read_set_denied_at(failure, Stage::DecisionReadSet)
    }

    pub(in crate::domain_computation::primary_graph) fn decision_read_set_denied_at(
        failure: WorthQueryDecisionReadSetFailure,
        stage: Stage,
    ) -> Self {
        let kind = match failure.kind() {
            Kind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => CommitKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            Kind::RetentionCapacityExhausted => CommitKind::RetentionCapacityExhausted,
            Kind::RetentionIdentityExhausted => CommitKind::RetentionIdentityExhausted,
            Kind::SnapshotIdentityExhausted => CommitKind::SnapshotIdentityExhausted,
            _ => CommitKind::ProviderRejected,
        };
        Self {
            kind,
            stage,
            detail: Some(failure.detail().into()),
            cause: Some(Box::new(DenialCause::DecisionReadSet(failure))),
        }
    }
}
