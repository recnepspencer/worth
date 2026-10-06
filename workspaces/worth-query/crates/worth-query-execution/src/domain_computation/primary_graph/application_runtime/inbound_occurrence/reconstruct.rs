//! World pairing for completion rows and exact cleanup verification.

use worth_runtime_world::facade::{
    CompositeCommitProvenance, CompositeComponentChangePosture, CompositeHistoryTraversal,
};

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
use crate::domain_computation::primary_graph::provider::WorthQueryInboundTerminalIndexDenial as Denial;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation) fn verify_completed_inbound_for_cleanup(
        &self,
        correlations: &[ExternalEffectCorrelationIdentity],
        now_unix_seconds: u64,
    ) -> Result<(), Denial> {
        self.primary_provider
            .verify_completed_inbound_for_cleanup(correlations, now_unix_seconds)
    }
}

pub(super) fn pair_world_history(
    history: &CompositeHistoryTraversal,
    row: &crate::domain_computation::primary_graph::provider::WorthQueryCanonicalCompletionRow,
) -> Result<
    (
        worth_runtime_world::facade::CompositeCommitIdentity,
        worth_runtime_world::facade::CompositeCommitIdentity,
        worth_runtime_world::facade::CompositePublicationAttemptIdentity,
    ),
    Denial,
> {
    let mut completion = None;
    for commit in history.commits() {
        let Some(relational) = commit.relational_publication_identity() else {
            continue;
        };
        if matches_receipt(relational, &row.completion_commit) {
            let CompositeCommitProvenance::Publication(attempt) = commit.provenance() else {
                return Err(Denial::WorldPairMismatch);
            };
            if commit.relational_change() != CompositeComponentChangePosture::Published
                || completion.is_some()
            {
                return Err(Denial::WorldPairMismatch);
            }
            completion = Some((commit.identity().clone(), attempt.clone()));
        }
        if matches_receipt(relational, &row.original_commit) {
            let Some((completed, attempt)) = completion else {
                return Err(retired_or(history, Denial::WorldPairMismatch));
            };
            return Ok((commit.identity().clone(), completed, attempt));
        }
    }
    Err(retired_or(
        history,
        if history.is_complete() {
            Denial::WorldPairMismatch
        } else {
            Denial::ReconstructionWorkExhausted
        },
    ))
}

/// A walk that skipped or stopped at retired history cannot prove a pair
/// absent or mismatched; the retired commits are unavailable.
fn retired_or(history: &CompositeHistoryTraversal, denial: Denial) -> Denial {
    if history.crossed_retired_history() {
        Denial::IndexUnavailable
    } else {
        denial
    }
}

fn matches_receipt(
    identity: &worth_relational::facade::history::RelationalCommitIdentity,
    receipt: &worth_relational::facade::history::RelationalCommitReceipt,
) -> bool {
    identity.commit_id() == receipt.commit_id
        && identity.version_id() == receipt.version_id
        && identity.authoring_branch() == &receipt.branch_id
}
