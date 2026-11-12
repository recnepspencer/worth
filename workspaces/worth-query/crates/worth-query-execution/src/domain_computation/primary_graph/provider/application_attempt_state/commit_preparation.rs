//! Atomic extraction and validation of one prepared session's commit inputs.

use crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage;
use crate::domain_computation::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
};

use super::super::{
    mutation_work::WorthQueryPrimaryMutationWorkCounters, WorthQueryPrimaryGraphApplicationAttempt,
    WorthQueryPrimaryGraphProvider,
};

mod coupled;
pub(super) use coupled::CoupledCommitPreparation;
pub(crate) mod preimage_retention;
pub(crate) mod relational_commit;
pub(in crate::domain_computation::primary_graph) use preimage_retention::WorthQueryPreImageRetentionWork;
pub(crate) use preimage_retention::{
    WorthQueryRetainedPreImageSeal, WorthQueryRetainedRelationPreImageSeal,
};
pub(in crate::domain_computation::primary_graph::provider) use relational_commit::publish_recovered;
pub(in crate::domain_computation::primary_graph::provider) use relational_commit::ManagedUnpublishedAttempt;
pub(in crate::domain_computation::primary_graph) use relational_commit::RebaseVerificationFailure;
pub use relational_commit::{FactlessCurrentness, OwnEffectOnReads, RebaseVerificationReason};
pub(in crate::domain_computation::primary_graph) use relational_commit::{
    WorthQueryMutationWorkCommitSeal, WorthQueryPrimaryGraphCommittedApplication,
};

pub(super) struct WorthQueryPreparedApplicationCommit {
    attempt: WorthQueryPrimaryGraphApplicationAttempt,
    members: Vec<super::CoupledApplicationMember>,
    candidate: worth_relational::facade::mvcc::ValidatedRelationalProposal,
    predicate: Option<crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner>,
    work: WorthQueryPrimaryMutationWorkCounters,
    retained_preimage: Option<WorthQueryRetainedPreImage>,
    preimage_retention_work: WorthQueryPreImageRetentionWork,
    source_fact_rebase: relational_commit::PreparedSourceFactRebase,
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
    history: Option<crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerCutover<'_>>,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    crate::domain_computation::WorthQueryProviderCommitCompletion,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let prepared =
        take_prepared_session(provider, session, allocation_policy).map_err(preparation_stop)?;
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
    relational_commit::commit_owner_validated(
        session.execution_request(),
        provider,
        prepared,
        history,
        allocation_policy,
    )
}

fn take_prepared_session(
    provider: &WorthQueryPrimaryGraphProvider,
    session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
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
    let (mut attempt, approved, completion) = prepared.into_parts();
    let super::WorthQueryApprovedApplicationCandidate {
        mut members,
        native: candidate,
        work,
        predicate,
    } = approved;
    coupled::prepare(
        &mut members,
        &candidate,
        &session.execution_request(),
        &provider.graph.layout,
        allocation_policy,
    )?;
    let source_fact_rebase = attempt.take_source_fact_rebase();
    let (retained_preimage, preimage_retention_work) = preimage_retention::retain_attempt_preimage(
        &attempt,
        &candidate,
        &session.execution_request(),
        &provider.graph.layout,
        allocation_policy,
    )?
    .into_parts();
    Ok(WorthQueryPreparedApplicationCommit {
        attempt,
        members,
        candidate,
        predicate,
        work,
        retained_preimage,
        preimage_retention_work,
        source_fact_rebase,
        _completion: completion,
    })
}

fn commit_failure(detail: &'static str) -> WorthQueryProviderSessionFailure {
    super::super::session_commit::provider_failure(
        WorthQueryProviderSessionProtocolStage::Commit,
        detail,
    )
}

pub(in crate::domain_computation::primary_graph) use relational_commit::{
    PreparedRebaseDenial, PreparedSourceFactRebase,
};

// Every stop here precedes the Native transaction. A read control stop keeps
// its control posture; every other typed refusal (a content or read rejection,
// a relation preimage, any preimage retention denial, candidate support) is a
// pre-effect refusal carrying its cause (public #122). The rest go through the
// existing abort-and-cleanup owner.
pub(in crate::domain_computation) use relational_commit::QueryPreparingApplicationCommit;
pub(in crate::domain_computation::primary_graph::provider) use relational_commit::RetainedQueryDurablePreparation;
// Typed content/read stops precede the Native transaction. Preserve semantic
// protocol failures through the existing abort-and-cleanup owner.
fn preparation_stop(
    failure: WorthQueryProviderSessionFailure,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::{
        WorthQueryProviderSessionCommitControlStopped as Control,
        WorthQueryProviderSessionCommitStop as Stop,
    };
    match failure.checked_read_stop() {
        Some(Err(kind)) => {
            Stop::ControlStopped(Control::execution_preparation(kind, failure.detail()))
        }
        Some(Ok(_)) => Stop::PreEffectDenied(failure),
        None if failure.relation_preimage_failure().is_some()
            || failure.preimage_retention_denial().is_some()
            || failure.candidate_support_failure().is_some() =>
        {
            Stop::PreEffectDenied(failure)
        }
        None => Stop::from(failure),
    }
}
#[cfg(test)]
#[path = "commit_preparation_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
