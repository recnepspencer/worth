//! Typed progression for one owner-validated Relational commit.

pub(crate) mod commit_execution;
pub(super) use commit_execution::PreparedTouchedRecords;
pub(crate) mod evidence_seal;
pub(in crate::domain_computation::primary_graph::provider) use commit_execution::ManagedUnpublishedAttempt;

pub(in crate::domain_computation::primary_graph) use commit_execution::WorthQueryPrimaryGraphCommittedApplication;

pub(in crate::domain_computation::primary_graph) use evidence_seal::RebaseVerificationFailure;
pub use evidence_seal::{FactlessCurrentness, OwnEffectOnReads, RebaseVerificationReason};
pub(in crate::domain_computation::primary_graph) use evidence_seal::{
    WorthQueryMutationWorkCommitSeal, WorthQueryPrimaryGraphCommitEvidence,
};

use super::super::super::WorthQueryPrimaryGraphProvider;
use super::WorthQueryPreparedApplicationCommit;
pub(super) struct WorthQueryCommitProgressionMint {
    _private: (),
}

impl WorthQueryCommitProgressionMint {
    fn witness() -> Self {
        Self { _private: () }
    }
}

pub(super) fn commit_owner_validated(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    prepared: WorthQueryPreparedApplicationCommit,
    history: Option<crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerCutover<'_>>,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    crate::domain_computation::WorthQueryProviderTerminalDescription,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let request = prepared.attempt.affinity().publication_request().clone();
    let committed = commit_execution::commit(
        execution,
        provider,
        prepared,
        WorthQueryCommitProgressionMint::witness(),
        history,
        allocation_policy,
    )
    .map_err(|stop| {
        use crate::domain_computation::{
            WorthQueryProviderSessionCommitControlStopped as Stopped,
            WorthQueryProviderSessionCommitStop as Stop,
            WorthQueryProviderSessionControlStopKind as Kind,
        };
        use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
        // Every pre-publication stage uses the original caller's posture.
        // A stopped lease with a live caller remains a preparation-retry denial.
        match (stop, request.interruption()) {
            (Stop::ControlStopped(stopped), Some(interruption)) => {
                let kind = match interruption {
                    WorthQueryRequestInterruption::Cancelled => Kind::Cancelled,
                    WorthQueryRequestInterruption::DeadlineExceeded => Kind::TimedOut,
                };
                Stop::ControlStopped(Stopped::new(kind, stopped.detail()))
            }
            (stop, _) => stop,
        }
    })?;
    finish_owner_validated(execution, provider, committed)
}

#[inline(never)]
fn finish_owner_validated(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    mut committed: commit_execution::WorthQueryCommittedApplicationSession,
) -> Result<
    crate::domain_computation::WorthQueryProviderTerminalDescription,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let evidence = evidence_seal::seal(&execution, provider, &mut committed);
    let mut predicate = committed.take_predicate_preparation();
    let mut retired = crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationRetirement::default();
    provider.graph.with_runtime(&execution, |runtime| {
        committed
            .publish_and_encode(
                &execution,
                provider,
                runtime,
                evidence,
                &mut predicate,
                &mut retired,
            )
            .map_err(crate::domain_computation::WorthQueryProviderSessionCommitStop::Denied)
    })
}

/// The recovered World movement rejoins the same evidence and pending
/// publication owner as a live application commit. The recovered session
/// carries the original Relational result and all pre-effect Query claims.
pub(in crate::domain_computation::primary_graph::provider) fn publish_recovered(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    mut committed: commit_execution::WorthQueryCommittedApplicationSession,
    producer_memory: Option<Option<worth_execution::ExecutionMemoryReservation>>,
    retain: impl FnOnce(Option<std::sync::Arc<crate::domain_computation::primary_graph::application_output_demand::RecoveredProducerOutputFacts>>),
) -> Result<
    crate::domain_computation::WorthQueryProviderTerminalDescription,
    crate::domain_computation::WorthQueryProviderSessionFailure,
> {
    let evidence = evidence_seal::seal(execution, provider, &mut committed);
    let facts =
        producer_memory.map(|memory| committed.retain_recovered_producer_facts(&evidence, memory));
    retain(facts);
    let mut predicate = committed.take_predicate_preparation();
    let mut retired = crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationRetirement::default();
    provider.graph.with_runtime(execution, |runtime| {
        committed.publish_and_encode(
            execution,
            provider,
            runtime,
            evidence,
            &mut predicate,
            &mut retired,
        )
    })
}

pub(in crate::domain_computation::primary_graph) use evidence_seal::{
    PreparedRebaseDenial, PreparedSourceFactRebase,
};

pub(in crate::domain_computation) use commit_execution::QueryPreparingApplicationCommit;
pub(in crate::domain_computation::primary_graph::provider) use commit_execution::RetainedQueryDurablePreparation;
#[cfg(test)]
#[path = "relational_commit_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
