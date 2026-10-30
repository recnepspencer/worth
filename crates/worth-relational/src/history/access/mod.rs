mod ancestry;
mod aspect_history;
mod aspect_history_queries;
mod bounded_commit_patches;
mod branch_envelope_reads;
mod commit_surfaces;
mod envelope_reads;
mod merge_branch_basis;
#[cfg(test)]
mod merge_branch_basis_foundational;
mod patch_stream_commit;
pub(crate) mod relevant_writes;
pub use relevant_writes::{
    RelationalAuthoredEntityIndexDefinition, RelationalAuthoredWriteCursor,
    RelationalAuthoredWriteDenial, RelationalAuthoredWriteOccurrence, RelationalAuthoredWritePage,
    RelationalAuthoredWritePageLimits, RelationalAuthoredWriteScope,
    RelationalAuthoredWriteSelector, RelationalPreparedAuthoredWritePage,
};

use crate::runtime::RelationalRuntime;

pub(crate) use ancestry::{CommitAncestryInspection, CommitAncestryPosture};
pub use aspect_history::{
    RelationalCanonicalRecordHistoryCursor, RelationalPreparedRecordHistoryPage,
    RelationalRecordHistoryChange, RelationalRecordHistoryDenial, RelationalRecordHistoryPage,
    RelationalRecordHistoryPageLimits, RelationalRecordHistorySelection,
    RelationalRecordHistorySelectionLimits, RelationalRecordHistorySubject,
};
pub use bounded_commit_patches::BoundedCanonicalCommitPatchDenial;

pub struct HistoryAccess<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl RelationalRuntime {
    pub(crate) fn current_version_id(&self) -> crate::identity::data::VersionId {
        self.history.current_version_id()
    }

    pub(crate) fn history_access(&self) -> HistoryAccess<'_> {
        HistoryAccess::new(self)
    }
}

impl<'runtime> HistoryAccess<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}
