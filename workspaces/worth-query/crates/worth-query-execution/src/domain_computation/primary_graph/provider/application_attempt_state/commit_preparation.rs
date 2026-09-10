//! Atomic extraction and validation of one prepared session's commit inputs.

use crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage;
use crate::domain_computation::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
};

use super::super::{
    mutation_work::WorthQueryPrimaryMutationWorkCounters, WorthQueryPrimaryGraphApplicationAttempt,
    WorthQueryPrimaryGraphProvider,
};

mod preimage_retention;
mod relational_commit;
pub(in crate::domain_computation::primary_graph) use preimage_retention::WorthQueryPreImageRetentionWork;
pub(crate) use preimage_retention::WorthQueryRetainedPreImageSeal;
pub(in crate::domain_computation::primary_graph) use relational_commit::{
    WorthQueryMutationWorkCommitSeal, WorthQueryPrimaryGraphCommittedApplication,
};

pub(super) struct WorthQueryPreparedApplicationCommit {
    attempt: WorthQueryPrimaryGraphApplicationAttempt,
    candidate: worth_relational::facade::mvcc::ValidatedRelationalProposal,
    work: WorthQueryPrimaryMutationWorkCounters,
    branch: worth_relational::facade::history::BranchId,
    retained_preimage: Option<WorthQueryRetainedPreImage>,
    preimage_retention_work: WorthQueryPreImageRetentionWork,
    _completion: super::commit_completion::WorthQueryApplicationAttemptCompletion,
}

impl WorthQueryPreparedApplicationCommit {
    fn validate_decision_work(&self) -> Result<(), WorthQueryProviderSessionFailure> {
        let complete = self.attempt.decision_fact_count() == self.attempt.facts().len()
            && self.attempt.affinity().graph_work_session().as_u64() != 0
            && self.work.decision_fact_count() == self.attempt.decision_fact_count()
            && self.work.proposed_fact_count() == self.attempt.expected_steps().len();
        if complete {
            Ok(())
        } else {
            Err(super::super::session_commit::provider_failure(
                WorthQueryProviderSessionProtocolStage::Commit,
                "application attempt lost its complete session decision facts",
            ))
        }
    }
}

pub(in crate::domain_computation::primary_graph::provider) fn commit_prepared_application(
    provider: &WorthQueryPrimaryGraphProvider,
    session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
) -> Result<
    crate::domain_computation::WorthQueryProviderTerminalDescription,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let prepared = take_prepared_session(provider, session)
        .map_err(crate::domain_computation::WorthQueryProviderSessionCommitStop::from)?;
    if provider.take_rejected_commit_before_transaction() {
        return Err(
            crate::domain_computation::WorthQueryProviderSessionCommitStop::Denied(
                super::super::session_commit::provider_failure(
                    WorthQueryProviderSessionProtocolStage::Commit,
                    "injected rejection before the atomic application transaction",
                ),
            ),
        );
    }
    prepared
        .validate_decision_work()
        .map_err(crate::domain_computation::WorthQueryProviderSessionCommitStop::from)?;
    relational_commit::commit_owner_validated(provider, prepared)
}

fn take_prepared_session(
    provider: &WorthQueryPrimaryGraphProvider,
    session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
) -> Result<WorthQueryPreparedApplicationCommit, WorthQueryProviderSessionFailure> {
    let attempts = std::sync::Arc::clone(&provider.attempts);
    let prepared = {
        attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take_commit_prepared(session, std::sync::Arc::clone(&attempts))
    }
    .ok_or_else(|| {
        commit_failure("primary graph session has no exact commit-prepared application attempt")
    })?;
    let (attempt, candidate, work, completion) = prepared.into_parts();
    let branch = attempt.affinity().branch().clone();
    let (retained_preimage, preimage_retention_work) =
        preimage_retention::retain_attempt_preimage(&attempt, &candidate)?.into_parts();
    Ok(WorthQueryPreparedApplicationCommit {
        attempt,
        candidate,
        work,
        branch,
        retained_preimage,
        preimage_retention_work,
        _completion: completion,
    })
}

fn commit_failure(detail: &'static str) -> WorthQueryProviderSessionFailure {
    super::super::session_commit::provider_failure(
        WorthQueryProviderSessionProtocolStage::Commit,
        detail,
    )
}
