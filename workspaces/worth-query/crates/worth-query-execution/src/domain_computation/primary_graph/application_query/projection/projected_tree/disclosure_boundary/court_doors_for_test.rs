//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationDisclosedProjectionTree {
    #[doc(hidden)]
    pub fn clear_for_test(&mut self) {
        Self::clear(self)
    }
}

impl WorthQueryApplicationWorkingProjectionTree {
    #[doc(hidden)]
    pub fn into_disclosed_for_test(
        self,
        governance: &WorthQueryApplicationQueryGovernance,
    ) -> WorthQueryApplicationDisclosedProjectionTree {
        Self::into_disclosed(self, governance)
    }
}

impl WorthQueryApplicationDisclosedProjectionTree {
    #[doc(hidden)]
    pub fn take_for_test(&mut self) -> Self {
        Self::take(self)
    }
}
