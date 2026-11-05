//! Moved whole from `domain_computation::primary_graph::output_lineage::input_cutoff::verification`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::output_lineage::derived::SourceSettlementCurrentness;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason;

/// What the cutoff does not verify: a row still in its checkpoint posture,
/// and a settlement no mark row answers for under the selected source.
/// Selection asks this before it calls a live output exact, so the two
/// cannot disagree about one row.
pub(in crate::domain_computation::primary_graph) fn cutoff_declines(
    requirement: Option<FullVerificationReason>,
    settlement: Option<&SourceSettlementCurrentness>,
) -> bool {
    requirement == Some(FullVerificationReason::CheckpointRestore)
        || settlement.is_some_and(SourceSettlementCurrentness::no_row_answers)
}
