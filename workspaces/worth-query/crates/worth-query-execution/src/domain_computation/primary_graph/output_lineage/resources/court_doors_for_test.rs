//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn checkpoint_facts_for_receipt_for_test(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Option<(
        crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        &super::super::SealedNativeOutputWitness,
    )> {
        Self::checkpoint_facts_for_publication(self, receipt.into())
    }
}
