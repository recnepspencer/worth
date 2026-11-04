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
        return Ok(receipt.aftermath_causality().cloned());
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

/// Compare the request's correction meaning with the immutable original rows.
pub(super) fn recovered_answers_aftermath(
    pending: Option<&WorthQueryPendingAftermathCausality>,
    receipt: &crate::domain_computation::primary_graph::provider::WorthQueryRecoveredApplicationCommitReceipt,
) -> bool {
    let Some(pending) = pending else {
        return true;
    };
    let Some(actual) = receipt.aftermath_causality() else {
        return false;
    };
    let coordinates_match = |actual: &crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceCoordinates,
        expected: &crate::domain_computation::application_aftermath::WorthQueryRecordedCorrectionSource| {
        actual.world_owner() == expected.world_owner_coordinate()
            && actual.world_commit() == expected.world().ordinal()
            && actual.relational() == expected.relational()
    };
    let optional_match = |actual: Option<
        &crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceCoordinates,
    >,
                          expected: Option<
        &crate::domain_computation::application_aftermath::WorthQueryRecordedCorrectionSource,
    >| {
        match (actual, expected) {
            (Some(actual), Some(expected)) => coordinates_match(actual, expected),
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        }
    };
    let lineage = pending.correction_lineage();
    actual.role() == pending.role()
        && actual.parent() == pending.parent()
        && actual.child() == receipt.commit_reference()
        && actual.child().branch_id == *pending.target_branch()
        && coordinates_match(actual.source(), pending.recorded_source())
        && coordinates_match(actual.root(), lineage.root())
        && optional_match(
            actual.selected_alternative(),
            lineage.selected_alternative(),
        )
        && optional_match(actual.active_reversal(), lineage.active_reversal())
        && optional_match(actual.prior_transition(), lineage.prior_transition())
}
