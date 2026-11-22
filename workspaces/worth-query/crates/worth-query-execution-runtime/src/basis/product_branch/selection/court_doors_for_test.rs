//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductBranch {
    #[doc(hidden)]
    pub fn occurrence_for_test(self) -> worth_runtime_world::facade::ProductBranchIncarnation {
        Self::occurrence(self)
    }
}

impl WorthQueryProductBranch {
    #[doc(hidden)]
    pub fn from_occurrence_for_test(
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Self {
        Self::from_occurrence(occurrence)
    }
}
