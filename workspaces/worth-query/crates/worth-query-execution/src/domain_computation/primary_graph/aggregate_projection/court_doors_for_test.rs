//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryAggregateProjections {
    #[doc(hidden)]
    pub fn refresh_after_commit_for_test(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        after: &SnapshotHandle,
        patches: &[PublishedAuthoritativeRecordPatch],
    ) -> Result<(), Cause> {
        Self::refresh_after_commit(self, execution, runtime, before, after, patches)
    }
}
