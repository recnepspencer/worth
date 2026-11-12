//! Walk actual retained association metadata across Complete and uncovered originals.
use super::*;
impl RelationalCanonicalPublicationRoutes {
    pub(crate) fn next_associated_branch_transition(
        &self,
        after: RelationalBranchTransitionSequence,
    ) -> Option<Arc<PositionedBranchTransition>> {
        let state = self.branch_transitions.state();
        let start = state
            .log
            .partition_point(|entry| entry.positioned.sequence() <= after);
        state.log[start..]
            .iter()
            .find(|entry| entry.positioned.caller_association().is_some())
            .map(|entry| Arc::clone(&entry.positioned))
    }
}
