//! Resolve an issued snapshot under the original native execution request.
use super::super::reader::VisibilityReadContext;
use super::VisibilityProjectionView;
use crate::snapshots::data::SnapshotHandle;
impl<'runtime> VisibilityReadContext<'runtime> {
    pub fn project_snapshot_in_request(
        &self,
        handle: &SnapshotHandle,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<VisibilityProjectionView>, crate::execution::RelationalExecutionDenialCause>
    {
        crate::execution::check_request(execution)?;
        Ok(self.project_snapshot(handle))
    }
}
