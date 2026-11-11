//! Reconstruct an Empty source cell using its recorded schema and axes.
use super::*;
use crate::runtime::OriginalEmptySchemaObservation;

impl HistorySubsystem {
    pub(crate) fn restore_source_empty_schema(
        &self,
        original: &OriginalEmptySchemaObservation,
        advance_installation: bool,
    ) -> Result<(), String> {
        let mut cells = self.branch_cells.take_all();
        let result = (|| {
            let current = cells
                .get(&self.main_branch)
                .ok_or("source main cell missing")?;
            if !matches!(
                current.observation().target(),
                FoundationalBranchTarget::Empty
            ) {
                return Err("source initialization cannot replace a committed cell".to_owned());
            }
            let mut replacement = current.clone_for_head_replacement();
            if advance_installation {
                replacement
                    .advance_metadata()
                    .map_err(|cause| format!("source installation axes: {cause:?}"))?;
            }
            let observed = OriginalEmptySchemaObservation::from_empty_cell(&replacement)
                .ok_or("source Empty observation missing")?;
            if observed.generation != original.generation
                || observed.truth_version != original.truth_version
                || observed.lifecycle != original.lifecycle
            {
                return Err("source Empty schema axes disagree".to_owned());
            }
            let previous = current.root().ok_or("source Empty root missing")?;
            let reservation = self
                .reserve_branch_head_retirement(
                    current.identity(),
                    &previous,
                    current.head_retention(),
                )
                .map_err(|cause| format!("source Empty replacement retention: {cause:?}"))?;
            let next = RelationalBranchRoot::empty_with_schema_authority(original.schema.clone());
            replacement.install_root(next.clone());
            cells.insert(self.main_branch.clone(), replacement);
            reservation.transfer_head(&next).replace_head();
            Ok(())
        })();
        self.branch_cells.restore_all(cells);
        result
    }
}
