//! Original receipt correspondence is owned by Execution, not a source permit.
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryRecoveredApplicationCommitReceipt,
};
pub(super) fn matches_retained_original(
    live: &WorthQueryApplicationCommitReceipt,
    recorded: &WorthQueryRecoveredApplicationCommitReceipt,
) -> bool {
    live.matches_recorded_original(recorded)
}
pub(super) fn same_recorded_subject(
    left: &WorthQueryRecoveredApplicationCommitReceipt,
    right: &WorthQueryRecoveredApplicationCommitReceipt,
) -> bool {
    left.same_recorded_subject_as(right)
}
