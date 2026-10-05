//! Typed progression for one owner-validated Relational commit.

mod commit_execution;
mod evidence_seal;
pub(in crate::domain_computation::primary_graph::provider) use commit_execution::ManagedUnpublishedAttempt;
pub(in crate::domain_computation::primary_graph) use commit_execution::RetainedTouchedRecords;

pub(in crate::domain_computation::primary_graph) use commit_execution::WorthQueryPrimaryGraphCommittedApplication;

pub(in crate::domain_computation::primary_graph::provider) use evidence_seal::PreparedSourceFactRebase;
pub(in crate::domain_computation::primary_graph) use evidence_seal::{
    OwnEffectOnReads, RebaseVerificationReason, WorthQueryMutationWorkCommitSeal,
    WorthQueryPrimaryGraphCommitEvidence,
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
    provider: &WorthQueryPrimaryGraphProvider,
    prepared: WorthQueryPreparedApplicationCommit,
) -> Result<
    crate::domain_computation::WorthQueryProviderTerminalDescription,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let mut committed = commit_execution::commit(
        provider,
        prepared,
        WorthQueryCommitProgressionMint::witness(),
    )?;
    let (evidence, publication_admission) = evidence_seal::seal(provider, &mut committed);
    provider.graph.with_runtime_mut_unwind_isolated(|runtime| {
        committed
            .publish_and_encode(provider, runtime, evidence, publication_admission)
            .map_err(crate::domain_computation::WorthQueryProviderSessionCommitStop::Denied)
    })
}

/// The recovered World movement rejoins the same evidence and pending
/// publication owner as a live application commit. The recovered session
/// carries the original Relational result and all pre-effect Query claims.
pub(in crate::domain_computation::primary_graph::provider) fn publish_recovered(
    provider: &WorthQueryPrimaryGraphProvider,
    mut committed: commit_execution::WorthQueryCommittedApplicationSession,
) -> Result<
    crate::domain_computation::WorthQueryProviderTerminalDescription,
    crate::domain_computation::WorthQueryProviderSessionFailure,
> {
    let (evidence, publication_admission) = evidence_seal::seal(provider, &mut committed);
    provider.graph.with_runtime_mut_unwind_isolated(|runtime| {
        committed.publish_and_encode(provider, runtime, evidence, publication_admission)
    })
}
