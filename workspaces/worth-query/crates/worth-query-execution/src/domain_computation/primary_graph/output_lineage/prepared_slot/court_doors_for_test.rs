//! Sealed test doors over this module's private members.
use super::*;

impl PreparedOutputLineageSlot {
    #[doc(hidden)]
    pub fn computation_for_test(&self) -> &PreparedComputationCustody {
        &self.computation
    }
}

impl PreparedOutputLineageSlot {
    #[doc(hidden)]
    pub fn retain_computation_for_test(
        &mut self,
        sealed: SealedComputationRetention,
        prior: Option<super::super::PriorComputationRecord>,
    ) {
        Self::retain_computation(self, sealed, prior)
    }
}
