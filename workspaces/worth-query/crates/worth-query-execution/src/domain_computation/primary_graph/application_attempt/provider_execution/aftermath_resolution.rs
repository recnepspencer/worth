//! Exact recovery of a co-committed aftermath causal fact.

use crate::domain_computation::application_aftermath::{
    WorthQueryCommittedAftermathCausality, WorthQueryPendingAftermathCausality,
};
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication;

/// The commit's sealed aftermath relation is not the one asked for.
#[derive(Debug)]
pub(super) struct WorthQueryCommittedAftermathMismatch;

/// Answers from the relation sealed into the commit's evidence at
/// publication, so a replay after the commit's history retired still answers
/// it. A sealed relation other than the one asked for is genuine drift.
pub(super) fn resolve_exact_committed_aftermath(
    pending: Option<&WorthQueryPendingAftermathCausality>,
    receipt: &WorthQueryPrimaryGraphCommittedApplication,
) -> Result<Option<WorthQueryCommittedAftermathCausality>, WorthQueryCommittedAftermathMismatch> {
    let Some(pending) = pending else {
        return Ok(None);
    };
    receipt
        .aftermath_causality()
        .filter(|committed| {
            committed.answers(pending) && committed.child() == receipt.commit_reference()
        })
        .cloned()
        .map(Some)
        .ok_or(WorthQueryCommittedAftermathMismatch)
}
