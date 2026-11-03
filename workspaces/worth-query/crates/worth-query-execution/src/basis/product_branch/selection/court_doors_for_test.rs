//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductBranch {
    #[doc(hidden)]
    pub fn occurrence_for_test(self) -> worth_runtime_world::facade::ProductBranchIncarnation {
        Self::occurrence(self)
    }
}
