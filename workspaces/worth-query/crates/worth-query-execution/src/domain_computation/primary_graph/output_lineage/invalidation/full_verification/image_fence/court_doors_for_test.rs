//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;

impl FullVerificationImage<'_> {
    #[doc(hidden)]
    pub fn fence_semantic_image_for_test(
        &self,
        observed: &OrdSet<Arc<RecordedSettlementIdentity>>,
    ) -> Result<(), FullVerificationStop> {
        Self::fence_semantic_image(self, observed)
    }
}
