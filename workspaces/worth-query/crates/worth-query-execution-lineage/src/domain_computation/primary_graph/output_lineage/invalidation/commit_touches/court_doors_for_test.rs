//! Sealed test doors over this module's private members.
use super::*;

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn latest_position_for_test(&self) -> Option<PatchStreamPosition> {
        Self::latest_position(self)
    }
}

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn touched_commits_after_for_test(
        &self,
        branch: &BranchId,
        cursor: Option<PatchStreamPosition>,
        ceiling: Option<CommitId>,
        limit: usize,
        interest: CommitTouchInterest<'_>,
    ) -> TouchedCommits {
        Self::touched_commits_after(self, branch, cursor, ceiling, limit, interest)
    }
}
