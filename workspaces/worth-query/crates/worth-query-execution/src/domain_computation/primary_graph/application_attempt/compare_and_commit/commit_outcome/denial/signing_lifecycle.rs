//! Lifecycle refusals occur before either source or consequence is published.
use {super::WorthQueryApplicationCommitDenial, crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialStage, crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialKind};
impl WorthQueryApplicationCommitDenial {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn signing_lifecycle_attempt(
        cause: crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::SigningLifecycleObservation {
                kind: cause.kind(),
            },
            stage: WorthQueryApplicationCommitDenialStage::DecisionReadSet,
            detail: None,
            cause: Some(Box::new(
                super::denial_cause::DenialCause::SigningLifecycleAttempt(cause),
            )),
        }
    }
    pub fn signing_lifecycle_observation_failure(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial> {
        match self.cause.as_deref() {
            Some(super::denial_cause::DenialCause::SigningLifecycleAttempt(cause)) => Some(cause),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn signing_lifecycle(
        kind: WorthQueryApplicationCommitDenialKind,
    ) -> Self {
        Self {
            kind,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: None,
            cause: None,
        }
    }
}
