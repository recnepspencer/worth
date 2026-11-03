//! Sealed test doors over this module's private members.
use super::*;

impl RequiredOutputDemandContext {
    #[doc(hidden)]
    pub fn prepare_prerequisites_for_test<'a>(
        self, inputs: impl ExactSizeIterator<Item = &'a crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>, source_owner: &crate::domain_computation::primary_graph::SourceInvalidationOwner) -> Result<PreparedPrerequisiteClaims, WorthQueryOutputDemandDenial> {
        Self::prepare_prerequisites(self, inputs, source_owner)
    }
}
