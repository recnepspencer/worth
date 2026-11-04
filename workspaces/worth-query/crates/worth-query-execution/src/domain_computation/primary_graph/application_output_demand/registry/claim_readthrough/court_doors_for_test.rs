//! Sealed test doors over this module's private members.
use super::*;

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn prepare_claim_readthrough_for_test(
        &self,
        owner: &WorthQueryOutputDemandKey,
        upstream: &[Arc<WorthQueryOutputDemandKey>],
    ) -> Result<Option<ClaimBatch>, crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial> {
        Self::prepare_claim_readthrough(self, owner, upstream)
    }
}
