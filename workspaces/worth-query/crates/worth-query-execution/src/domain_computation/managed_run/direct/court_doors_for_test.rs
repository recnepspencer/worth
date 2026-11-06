//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRunningDirectRun {
    #[doc(hidden)]
    pub fn terminate_for_convergence_for_test(
        self,
        kind: WorthQueryManagedRunTerminalKind,
    ) -> WorthQueryDirectRunTerminal {
        Self::terminate_for_convergence(self, kind)
    }
}

impl WorthQueryRunningDirectRun {
    #[doc(hidden)]
    pub fn execution_snapshot_reference_for_test(&self) -> String {
        Self::execution_snapshot_reference(self)
    }
}
