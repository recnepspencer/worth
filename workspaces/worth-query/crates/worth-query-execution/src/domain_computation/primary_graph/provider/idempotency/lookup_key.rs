//! Lookup affinity follows command meaning; it is not a second evidence owner.
use super::{WorthQueryApplicationIdempotencyBinding, WorthQueryProductIdempotencyAffinity};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryIdempotencyLookupKey {
    Ordinary(WorthQueryProductIdempotencyAffinity, [u8; 32]),
    Correction([u8; 32]),
}
impl WorthQueryIdempotencyLookupKey {
    pub(in crate::domain_computation::primary_graph) fn for_binding(
        product: &WorthQueryProductIdempotencyAffinity,
        binding: &WorthQueryApplicationIdempotencyBinding,
    ) -> Self {
        if binding.correction_identity().is_some() {
            Self::Correction(*binding.key_identity())
        } else {
            Self::Ordinary(product.clone(), *binding.key_identity())
        }
    }
}
