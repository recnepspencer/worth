//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPausedWorkflowGraphExecution {
    #[doc(hidden)]
    pub fn active_for_test(&self) -> &WorthQueryActiveWorkflowGraphExecution {
        &self.active
    }
}
