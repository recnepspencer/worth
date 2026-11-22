//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPausedDirectGraphExecution {
    #[doc(hidden)]
    pub fn active_for_test(&self) -> &WorthQueryActiveDirectGraphExecution {
        &self.active
    }
}
