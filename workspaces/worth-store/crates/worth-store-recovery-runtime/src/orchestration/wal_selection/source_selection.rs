//! Required native WAL backing travels with the complete source selection.

use super::resident_allocation::WalSelectionAllocation;
use std::ops::Deref;
use worth_store_recovery_physics::PhysicalSourceSelection;

#[derive(Debug)]
pub(crate) struct ResidentSourceSelection {
    pub(super) selection: PhysicalSourceSelection,
    pub(super) allocation: WalSelectionAllocation,
}

impl ResidentSourceSelection {
    pub(crate) const fn facts(&self) -> &PhysicalSourceSelection {
        &self.selection
    }
    pub(crate) fn charged_bytes(&self) -> u64 {
        self.allocation.charged_bytes()
    }
}

impl Deref for ResidentSourceSelection {
    type Target = PhysicalSourceSelection;
    fn deref(&self) -> &Self::Target {
        self.facts()
    }
}
