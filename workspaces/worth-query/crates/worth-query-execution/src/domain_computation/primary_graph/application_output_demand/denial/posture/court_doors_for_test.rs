//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandDenialKind {
    #[doc(hidden)]
    pub fn of_execution_interruption_for_test(
        interruption: crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption,
    ) -> Self {
        Self::of_execution_interruption(interruption)
    }
}
