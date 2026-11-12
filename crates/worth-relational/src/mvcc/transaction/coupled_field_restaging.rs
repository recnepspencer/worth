//! Replace only the command/index representation after Native participant application.
use super::{BranchBoundRelationalTransaction, DetachedRelationalTransactionOverlay};
impl BranchBoundRelationalTransaction {
    pub(crate) fn restage_coupled_field_batches(
        &mut self,
        batches: Vec<crate::transactions::data::WorkerIntentBatch>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), super::RelationalTransactionStagingDenial> {
        let mut overlay = DetachedRelationalTransactionOverlay::default();
        // Original writer reads remain required even when their commands are
        // represented by one disjoint-field patch. No conflict evidence drops.
        let mut footprint = self.footprint.clone();
        for batch in batches {
            policy.check_live()?;
            let (index, next) = overlay.prepare_stage(&batch, &footprint, policy)?;
            overlay.reserve_input_directory()?;
            overlay.stage(batch, index);
            footprint = next;
        }
        policy.check_live()?;
        overlay.normalized_client_keys = std::mem::take(&mut self.overlay.normalized_client_keys);
        overlay.normalization_generation = self.overlay.normalization_generation;
        self.overlay = overlay;
        self.footprint = footprint;
        self.last_merged_plan = None;
        Ok(())
    }
}
