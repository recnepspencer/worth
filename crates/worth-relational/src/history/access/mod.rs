mod ancestry;
mod aspect_history_queries;
mod bounded_commit_patches;
mod commit_surfaces;
mod merge_branch_basis;
#[cfg(test)]
mod merge_branch_basis_foundational;
mod patch_stream_commit;

use crate::runtime::RelationalRuntime;

pub(crate) use ancestry::{CommitAncestryInspection, CommitAncestryPosture};
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
