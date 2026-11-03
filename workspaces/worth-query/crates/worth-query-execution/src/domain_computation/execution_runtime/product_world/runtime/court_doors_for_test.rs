//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn default_branch_for_test(&self) -> &ProductBranchIdentity {
        Self::default_branch(self)
    }
}
