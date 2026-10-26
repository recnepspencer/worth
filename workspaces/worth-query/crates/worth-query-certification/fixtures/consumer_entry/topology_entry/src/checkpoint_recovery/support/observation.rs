//! Resources and observations used by observer-backed checkpoint proofs.
pub(in crate::checkpoint_recovery) use super::execution_policy::CHECKPOINT_EXECUTION_POLICY;

impl crate::producer::InitialPlanarProvider {
    pub(crate) fn with_uniform_decimal_key_width(mut self) -> Self {
        // Twenty decimal digits for every source identity, independent of hashing.
        self.key_mask = 3 << 62;
        self
    }
}
