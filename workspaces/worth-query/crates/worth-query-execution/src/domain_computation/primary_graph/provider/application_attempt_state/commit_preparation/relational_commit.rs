//! Typed progression for one owner-validated Relational commit.

mod commit_execution;
mod evidence_seal;

pub(in crate::domain_computation::primary_graph) use commit_execution::WorthQueryPrimaryGraphCommittedApplication;

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
    provider: &WorthQueryPrimaryGraphProvider,
    prepared: WorthQueryPreparedApplicationCommit,
) -> Result<
    crate::domain_computation::WorthQueryProviderTerminalDescription,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let committed = commit_execution::commit(
        provider,
        prepared,
        WorthQueryCommitProgressionMint::witness(),
    )?;
    let evidence = evidence_seal::seal(&committed);
    provider.graph.with_runtime_mut(|runtime| {
        committed
            .publish_and_encode(provider, runtime, evidence)
            .map_err(crate::domain_computation::WorthQueryProviderSessionCommitStop::Denied)
    })
}
