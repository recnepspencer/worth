//! A delivery report can be observed only at its exact Native image header.
use super::NativeMarkingReport;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceInvalidationOwner;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn native_marking_report(
        &self,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<Option<NativeMarkingReport>, CompanionPreflightStop> {
        if selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        let Some(cell) = self.cell_for_read(selected)? else {
            return Ok(None);
        };

        let image = cell.read_image();
        if image.position() != selected.position()
            || image.root_id() != selected.root_id()
            || image.commit_id() != selected.commit_id()
        {
            return Ok(None);
        }

        // Derived edits preserve this report. Reading it again is the same
        // delivery observation, never another Native publication event.
        Ok(image
            .payload()
            .last_native_marking
            .filter(|report| Some(report.commit) == selected.commit_id()))
    }
}
