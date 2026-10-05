//! Fail-closed evidence for an indeterminate provider commit result.

pub(super) fn unknown_commit_recovery_evidence(
    detail: &'static str,
) -> super::super::WorthQueryApplicationUnresolvedCommitEvidence {
    let failure = crate::domain_computation::provider_session::WorthQueryProviderSessionFailure::new(
        crate::domain_computation::provider_session::WorthQueryProviderSessionDenialKind::ProviderRejected,
        crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage::Commit,
        detail,
        crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolCounters::default(),
    );
    super::super::WorthQueryApplicationUnresolvedCommitEvidence::from_provider_session_failure(
        super::super::WorthQueryApplicationCommitRecoveryKind::CommitRecoveryRequired,
        &failure,
    )
}
