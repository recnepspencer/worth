//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn observe_correction_key_for_test(
        &self,
        product: &worth_runtime_world::facade::ProductBranchObservation,
        key: &[u8; 32],
    ) -> CorrectionIdempotencyObservation {
        Self::observe_correction_key(self, product, key)
    }
}
