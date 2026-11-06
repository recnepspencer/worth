//! Complete invariant-owner refusal evidence at the application commit boundary.

use super::denial_cause::DenialCause;
use super::WorthQueryApplicationCommitDenial;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialKind as CommitKind;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialStage;
use crate::domain_computation::vocabulary::provider_session::invariant_execution_denial::WorthQueryInvariantExecutionDenialKind as Kind;
use crate::domain_computation::vocabulary::provider_session::invariant_execution_denial::WorthQueryInvariantExecutionFailure;

impl WorthQueryApplicationCommitDenial {
    /// The invariant owner's complete refusal, when invariant admission stopped
    /// this commit. Its kind and posture retain capacity bounds and distinguish
    /// exhaustion from an authority or semantic refusal.
    pub fn invariant_execution_failure(&self) -> Option<&WorthQueryInvariantExecutionFailure> {
        match self.cause.as_deref() {
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
                Kind::ExecutionDenied(kind) => super::execution::execution_kind(Ok(kind)),
                _ => CommitKind::ProviderRejected,
            }
        };
        Self {
            kind,
            stage,
            detail: Some(failure.detail().into()),
            cause: Some(Box::new(DenialCause::InvariantExecution(failure))),
        }
    }
}

mod court_doors_for_test;
