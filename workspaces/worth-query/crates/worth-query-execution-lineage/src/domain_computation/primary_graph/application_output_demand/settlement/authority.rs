//! Compare actual accepted owners; a display posture cannot establish authority.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
impl WorthQueryOutputDemandSettlement {
    pub fn require_original_provenance(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let original = self
            .observation
            .original_settled_producer()
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSettlement,
                    "settlement lost its original producer provenance",
                )
            })?;
        original.require_settlement(execution, self)
    }

    pub fn matches_authority(&self, authority: &WorthQueryAcceptedOutputAuthority) -> bool {
        match authority {
            WorthQueryAcceptedOutputAuthority::Committed(receipt) => self
                .application_commit_receipt()
                .is_some_and(|actual| actual.is_same_authoritative_commit(receipt)),
            WorthQueryAcceptedOutputAuthority::Recovered(publication) => self
                .receipt
                .as_ref()
                .and_then(SettlementReceiptCustody::recovered)
                .is_some_and(|actual| actual.same_publication(publication)),
            WorthQueryAcceptedOutputAuthority::Stable(stable) => self
                .stable
                .as_ref()
                .is_some_and(|actual| actual.same_publication(stable)),
            WorthQueryAcceptedOutputAuthority::Restored(restored) => self
                .restored_source
                .as_ref()
                .is_some_and(|actual| actual.same_authority(restored)),
        }
    }
    pub fn retained_read_ref(&self) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.observation
    }
}
