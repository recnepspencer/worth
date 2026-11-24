//! Admit publication before taking branch coordination or changing a cell.
use super::{canonical_publication_routes::BranchTransitionPublication, HistorySubsystem};

pub(crate) fn begin_branch_transition(
    history: &HistorySubsystem,
) -> BranchTransitionPublication<'_> {
    history
        .canonical_publication_routes
        .begin_branch_transition()
}
