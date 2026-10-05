//! Exact installed native discontinuity for a selected live Product basis.

use worth_relational::facade::{
    history::CommitId, mvcc::CompanionPreflightStop, runtime::PositionedRelationalSnapshot,
};

use super::super::admission::IndexAdmission;
use super::{InvalidationEditAdmission, SourceInvalidationOwner};

impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph) fn selected_installed_discontinuity(
        &self,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<CommitId>, CompanionPreflightStop> {
        if selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        let Some(cell) = self.cell_for_read(selected, admission)? else {
            return Ok(None);
        };
        admission.work(6)?;
        let image = cell.read_image();
        if image.root_id() != selected.root_id()
            || image.commit_id() != selected.commit_id()
            || image.position() != selected.position()
        {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        Ok(image.payload().current.delivery_epoch.commit_id())
    }
}
