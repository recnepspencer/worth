mod ancestry;
pub(crate) use worth_relational_branch::internal_port::history::access::ancestry_walk;
mod aspect_history;
pub(crate) use worth_relational_branch::internal_port::history::access::aspect_history_projection;
mod aspect_history_queries;
mod bounded_commit_patches;
mod branch_envelope_reads;
mod commit_surfaces;
mod envelope_reads;
pub(crate) use worth_relational_branch::internal_port::history::access::merge_basis_resolution;
mod merge_branch_basis;
mod patch_stream_commit;
pub(crate) mod relevant_writes;
pub use relevant_writes::live_cursor::RelationalAuthoredWriteCursor;
pub use relevant_writes::{
    RelationalAuthoredEntityIndexDefinition, RelationalAuthoredManagedSourceImpact,
    RelationalAuthoredManagedSourceMaterial, RelationalAuthoredManagedSourcePayloads,
    RelationalAuthoredManagedSourceSelector, RelationalAuthoredWriteDenial,
    RelationalAuthoredWriteOccurrence, RelationalAuthoredWritePage,
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
#[cfg(test)]
pub(crate) use worth_relational_branch::internal_port::history::access::merge_branch_basis_foundational;
