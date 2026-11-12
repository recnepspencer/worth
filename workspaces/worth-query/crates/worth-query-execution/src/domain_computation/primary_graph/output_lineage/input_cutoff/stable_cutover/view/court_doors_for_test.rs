//! Sealed test doors over this module's private members.
use super::*;

impl PublishedStableLineage {
    #[doc(hidden)]
    pub fn exact_settlement_for_test(
        &self,
    ) -> &Arc<super::super::super::super::RecordedSettlementIdentity> {
        Self::exact_settlement(self)
    }
}

impl PublishedStableLineage {
    #[doc(hidden)]
    pub fn output_correspondence_for_test(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence>
    {
        self.output_correspondence()
    }
    #[doc(hidden)]
    pub fn idempotency_key_identity_for_test(&self) -> [u8; 32] {
        self.idempotency_key_identity()
    }
}
