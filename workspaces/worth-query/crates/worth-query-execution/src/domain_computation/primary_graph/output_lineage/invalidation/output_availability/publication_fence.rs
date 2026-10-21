//! Pin the original image before full validation and carry it through publication.
use super::super::source_alignment::BranchMarkRoot;
use super::*;
use worth_relational::facade::mvcc::CompanionBranchCell;
use worth_relational::facade::mvcc::CompanionBranchImage;
use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;

pub(in crate::domain_computation::primary_graph) struct SourcePublicationReadFence {
    cell: CompanionBranchCell<BranchMarkRoot>,
    image: CompanionBranchImage<BranchMarkRoot>,
}

impl SourcePublicationReadFence {
    pub(in crate::domain_computation::primary_graph) fn bind(
        self,
        candidate: &mut PreparedRelationalCommitCandidate,
    ) -> Result<(), CompanionPreflightStop> {
        candidate.require_companion_image(&self.cell, self.image)
    }
}

impl SourceInvalidationOwner {
    /// Select ONE original image before full source/candidate comparisons.
    /// Check every outer completion slot here, then retain this exact image
    /// through nested validation and native preflight. A later matching outer
    /// slot cannot replace the image whose validation was admitted.
    pub(in crate::domain_computation::primary_graph) fn source_publication_read_fence(
        &self,
        observations: &[Arc<ObservedSourcePublication>],
        selected: &PositionedRelationalSnapshot,
    ) -> Result<SourcePublicationReadFence, CompanionPreflightStop> {
        let cell = self
            .cell_for_read(selected)?
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        let image = cell.read_image();

        if image.root_id() != selected.root_id()
            || image.commit_id() != selected.commit_id()
            || image.position() != selected.position()
        {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        for observed in observations {
            let current = image
                .payload()
                .current
                .source_publications
                .get(&observed.address)
                .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
            if !Arc::ptr_eq(current, &observed.state) {
                return Err(CompanionPreflightStop::SelectedSourceMismatch);
            }
        }
        Ok(SourcePublicationReadFence { cell, image })
    }
}
