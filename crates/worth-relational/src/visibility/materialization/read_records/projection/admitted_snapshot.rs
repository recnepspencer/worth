//! Issued exact projection with admission before visibility-basis copies.

use super::super::reader::VisibilityReadContext;
use super::VisibilityProjectionView;
use crate::snapshots::data::SnapshotHandle;

#[derive(Debug)]
pub enum RelationalSnapshotProjectionAdmissionStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
}

impl<'runtime> VisibilityReadContext<'runtime> {
    /// The active/published registry owns the selected branch width; the
    /// caller's handle text is not used as a forecast for its owned basis.
    /// The admission precedes `project_snapshot`'s existing resolution and
    /// preserves its issued-root and unavailable semantics exactly.
    pub fn project_snapshot_admitted<Stop>(
        &self,
        handle: &SnapshotHandle,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        Option<VisibilityProjectionView<'runtime>>,
        RelationalSnapshotProjectionAdmissionStop<Stop>,
    > {
        use RelationalSnapshotProjectionAdmissionStop as Denial;
        admit(3, 0).map_err(Denial::Admission)?;
        let runtime = self.runtime();
        if handle.runtime_instance_id() != runtime.runtime_instance_id() {
            return Ok(None);
        }
        let Some(branch_bytes) = runtime
            .visibility
            .selected_snapshot_branch_bytes(handle.snapshot_id())
        else {
            return Ok(None);
        };
        // Active resolution copies the basis key into a binding, its branch
        // metadata into that binding, then the basis key into the view.
        // Published resolution uses two of these three copies.
        let bytes = branch_bytes
            .checked_mul(3)
            .ok_or(Denial::AccountingOverflow)?;
        let work = bytes.checked_add(4).ok_or(Denial::AccountingOverflow)?;
        admit(
            u64::try_from(work).map_err(|_| Denial::AccountingOverflow)?,
            u64::try_from(bytes).map_err(|_| Denial::AccountingOverflow)?,
        )
        .map_err(Denial::Admission)?;
        Ok(self.project_snapshot(handle))
    }
}
