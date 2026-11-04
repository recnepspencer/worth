//! Sealed test doors over this module's private members.
use super::*;

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn resources_for_test(&self) -> &WorthQueryInvalidationResources {
        &self.resources
    }
}

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn branches_for_test(&self) -> &Mutex<BranchCells> {
        &self.branches
    }
}

impl BranchCells {
    #[doc(hidden)]
    pub fn cells_for_test(&self) -> &OrdMap<BranchId, CompanionBranchCellSlot<BranchMarkRoot>> {
        &self.cells
    }
}
