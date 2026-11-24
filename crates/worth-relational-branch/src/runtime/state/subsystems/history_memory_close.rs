//! Authentic stopped-owner head retirement without removing root inventory.
use super::HistorySubsystem;
use crate::history::retention::RetentionBusy;

impl HistorySubsystem {
    pub fn try_memory_targets_empty(&self) -> Result<bool, RetentionBusy> {
        self.branch_cells.try_memory_targets_empty()
    }

    pub fn try_retire_stopped_head(&self, index: usize) -> Result<Option<bool>, RetentionBusy> {
        self.branch_cells.try_retire_stopped_head(index)
    }
}
