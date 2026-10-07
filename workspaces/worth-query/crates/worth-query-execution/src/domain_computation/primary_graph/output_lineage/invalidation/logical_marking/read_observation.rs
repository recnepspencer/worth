//! A delivery report can be observed only at its exact Native image header.
use super::NativeMarkingReport;
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    admission::IndexAdmission, InvalidationEditAdmission, SourceInvalidationOwner,
};
use worth_relational::facade::{
    mvcc::CompanionPreflightStop, runtime::PositionedRelationalSnapshot,
};

impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn native_marking_report(
        &self,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<NativeMarkingReport>, CompanionPreflightStop> {
        admission.work(1)?;
        if selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        let Some(cell) = self.cell_for_read(selected, admission)? else {
            return Ok(None);
        };
        admission.work(4)?;
        let image = cell.read_image();
        if image.position() != selected.position()
            || image.root_id() != selected.root_id()
            || image.commit_id() != selected.commit_id()
        {
            return Ok(None);
        }
        admission.work(1)?;
        // Derived edits preserve this report. Reading it again is the same
        // delivery observation, never another Native publication event.
        Ok(image
            .payload()
            .last_native_marking
            .filter(|report| Some(report.commit) == selected.commit_id()))
    }
}
