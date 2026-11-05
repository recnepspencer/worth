//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRunningWorkflowRun {
    #[doc(hidden)]
    pub fn terminate_for_convergence_for_test(
        self,
        kind: WorthQueryManagedRunTerminalKind,
    ) -> WorthQueryWorkflowRunTerminal {
        Self::terminate_for_convergence(self, kind)
    }
}
