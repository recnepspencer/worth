//! Native installs associations only from its selected reservation and judged set.
use super::{
    CompanionPreflightStop as Stop, PublicationCompanionPreflight, ReservedCompanionBranchCell,
};
use crate::validation::candidate_source::ManagedIndexBranchImage;
use std::sync::Arc;

impl PublicationCompanionPreflight<'_> {
    /// Initial managed directory backing is paid before its owning Arc exists.
    pub fn mint_managed_index_source_cell(
        &mut self,
    ) -> Result<super::super::PreparedCompanionBranchCell<ManagedIndexBranchImage>, Stop> {
        let image = ManagedIndexBranchImage::retain_header(
            self.execution,
            ManagedIndexBranchImage::default(),
        )?;
        self.mint_selected_branch_cell(image)
    }

    pub fn prepare_managed_source_image(
        &self,
        selected: &ReservedCompanionBranchCell<ManagedIndexBranchImage>,
    ) -> Result<Arc<ManagedIndexBranchImage>, Stop> {
        if selected.core.runtime_instance_id != self.binding.runtime_instance_id
            || selected.core.branch_id != self.binding.branch_id
            || selected.core.registration_generation != self.binding.registration_generation
            || selected.current().root_id() != self.binding.expected_root_id
        {
            return Err(Stop::SelectedSourceMismatch);
        }
        let mut next = (**selected.current().payload()).clone();
        for root in self.managed_sources().roots() {
            crate::authority::commit::pipeline::consult_stage(
                self.execution,
                crate::authority::commit::pipeline::CommitStage::Publication,
            )
            .map_err(|cause| Stop::Execution(cause.into()))?;
            self.checkpoint()?;
            next.prepare_source(
                self.execution,
                root.program(),
                root.scope(),
                root.physical_availability(),
                self.binding.next_commit_id,
                root.source_changed(),
                root.proposed(),
            )?;
        }
        self.checkpoint()?;
        ManagedIndexBranchImage::retain_header(self.execution, next)
    }
}
