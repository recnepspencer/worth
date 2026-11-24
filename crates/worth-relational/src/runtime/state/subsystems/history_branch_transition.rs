//! Admit publication before taking branch coordination or changing a cell.
use super::{
    canonical_publication_routes::BranchTransitionPublication,
    history_canonical_routes::RelationalPublicationHistory,
};

pub(crate) fn begin_branch_transition(
    history: &RelationalPublicationHistory,
) -> BranchTransitionPublication<'_> {
    history
        .canonical_publication_routes
        .begin_branch_transition()
}
