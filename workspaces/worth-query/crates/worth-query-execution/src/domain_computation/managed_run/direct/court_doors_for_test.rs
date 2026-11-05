//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunTerminalKind;

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
