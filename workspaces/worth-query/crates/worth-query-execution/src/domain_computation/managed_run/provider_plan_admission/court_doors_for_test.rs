//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRunningDirectRun {
    #[doc(hidden)]
    pub fn provider_plan_bridge_basis_for_test(&self) -> &BridgeBoundExecutionBasis {
        Self::provider_plan_bridge_basis(self)
    }
}
