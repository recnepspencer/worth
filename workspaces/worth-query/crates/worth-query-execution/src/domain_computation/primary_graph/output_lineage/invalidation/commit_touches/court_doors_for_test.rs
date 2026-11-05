//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::commit_touches::CommitTouchInterest;

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

impl TouchedCommits {
    #[doc(hidden)]
    pub fn caught_up_to_latest_for_test(&self) -> &bool {
        &self.caught_up_to_latest
    }
}

impl TouchedCommits {
    #[doc(hidden)]
    pub fn commits_for_test(&self) -> &Vec<TouchedCommit> {
        &self.commits
    }
}

impl TouchedCommits {
    #[doc(hidden)]
    pub fn next_cursor_for_test(&self) -> &Option<PatchStreamPosition> {
        &self.next_cursor
    }
}

impl TouchedCommit {
    #[doc(hidden)]
    pub fn commit_for_test(&self) -> &CommitId {
        &self.commit
    }
}
