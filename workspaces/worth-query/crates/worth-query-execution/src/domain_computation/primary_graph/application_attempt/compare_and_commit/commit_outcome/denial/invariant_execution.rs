//! Complete invariant-owner refusal evidence at the application commit boundary.

use super::{
    request_authority::DenialCause, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationCommitDenialKind as CommitKind, WorthQueryApplicationCommitDenialStage,
};
use crate::domain_computation::{
    WorthQueryInvariantExecutionDenialKind as Kind, WorthQueryInvariantExecutionFailure,
};

impl WorthQueryApplicationCommitDenial {
    /// The invariant owner's complete refusal, when invariant admission stopped
    /// this commit. Its kind and posture retain capacity bounds and distinguish
    /// exhaustion from an authority or semantic refusal.
    pub fn invariant_execution_failure(&self) -> Option<&WorthQueryInvariantExecutionFailure> {
        match &self.cause {
            Some(DenialCause::InvariantExecution(failure)) => Some(failure),
            _ => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn invariant_execution_denied(
        stage: WorthQueryApplicationCommitDenialStage,
        failure: WorthQueryInvariantExecutionFailure,
    ) -> Self {
        let kind = if failure.custom_invariant_denial().is_some() {
            CommitKind::CustomInvariantDenied
        } else {
            match failure.kind() {
                Kind::RetentionCapacityExhausted => CommitKind::RetentionCapacityExhausted,
                Kind::RetentionIdentityExhausted => CommitKind::RetentionIdentityExhausted,
                Kind::ProductBasisStale => CommitKind::ProductBasisStale,
                _ => CommitKind::ProviderRejected,
            }
        };
        Self {
            kind,
            stage,
            detail: Some(failure.detail().into()),
            cause: Some(DenialCause::InvariantExecution(failure)),
        }
    }
}
