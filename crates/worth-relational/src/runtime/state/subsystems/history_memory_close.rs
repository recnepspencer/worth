//! Authentic stopped-owner head retirement without removing root inventory.
use super::HistorySubsystem;

impl HistorySubsystem {
    pub(crate) fn try_memory_targets_empty(&self) -> Result<bool, ()> {
        self.branch_cells.try_memory_targets_empty()
    }

    pub(crate) fn try_retire_stopped_head(&self, index: usize) -> Result<Option<bool>, ()> {
        self.branch_cells.try_retire_stopped_head(index)
    }
}
