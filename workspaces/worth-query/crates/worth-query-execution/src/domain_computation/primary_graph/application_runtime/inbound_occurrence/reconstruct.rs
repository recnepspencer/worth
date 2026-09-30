//! Explicit bounded reconstruction after a disposable terminal index is lost.

#[cfg(test)]
use std::num::NonZeroUsize;

use worth_runtime_world::facade::{
    CompositeCommitProvenance, CompositeComponentChangePosture, CompositeHistoryTraversal,
    ProductBranchIncarnation,
};

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
#[cfg(test)]
use crate::domain_computation::primary_graph::provider::WorthQueryCanonicalInboundCompletion;
use crate::domain_computation::primary_graph::provider::WorthQueryInboundTerminalIndexDenial as Denial;

#[cfg(test)]
impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// The committed dispatch owner supplies the exact original incarnation.
    /// Absence is authoritative only within that lineage's current indexed
    /// Relational basis; retirement or missing index currency is unavailable.
    pub(in crate::domain_computation) fn reconstruct_completed_inbound_on_original(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
        incarnation: ProductBranchIncarnation,
        maximum_world_commits: NonZeroUsize,
    ) -> Result<Option<WorthQueryCanonicalInboundCompletion>, Denial> {
        let observation = self
            .product_runtime
            .owner
            .observation_port()
            .observe_product_branch_occurrence(incarnation)
            .map_err(|_| Denial::IndexUnavailable)?;
        let row = self
            .primary_provider
            .lookup_inbound_completion_row(observation.basis().relational_basis(), correlation)
            .map_err(|_| Denial::IndexUnavailable)?;
        let Some(row) = row else {
            return Ok(None);
        };
        if row.original_incarnation_ordinal != incarnation.ordinal() {
            return Err(Denial::WorldPairMismatch);
        }
        let history = self
            .product_runtime
            .owner
            .inspection_port()
            .trace_ancestry(observation.selected_commit().clone(), maximum_world_commits)
            .map_err(|_| Denial::IndexUnavailable)?;
        let (original, completion, attempt) = pair_world_history(&history, &row, incarnation)?;
        Ok(Some(
            WorthQueryCanonicalInboundCompletion::from_verified_row(
                row,
                incarnation,
                original,
                completion,
                attempt,
            ),
        ))
    }
}

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
    _incarnation: ProductBranchIncarnation,
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
                return Err(Denial::WorldPairMismatch);
            };
            return Ok((commit.identity().clone(), completed, attempt));
        }
    }
    Err(if history.is_complete() {
        Denial::WorldPairMismatch
    } else {
        Denial::ReconstructionWorkExhausted
    })
}

fn matches_receipt(
    identity: &worth_relational::facade::history::RelationalCommitIdentity,
    receipt: &worth_relational::facade::history::RelationalCommitReceipt,
) -> bool {
    identity.commit_id() == receipt.commit_id
        && identity.version_id() == receipt.version_id
        && identity.authoring_branch() == &receipt.branch_id
}
