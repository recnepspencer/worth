//! Sealed test doors over this module's private members.
use super::*;

impl RequiredOutputDemandContext {
    #[doc(hidden)]
    pub fn prepare_prerequisites_for_test<'a>(
        self, inputs: impl ExactSizeIterator<Item = &'a crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>, source_owner: &crate::domain_computation::primary_graph::SourceInvalidationOwner) -> Result<PreparedPrerequisiteClaims, WorthQueryOutputDemandDenial> {
        Self::prepare_prerequisites(self, inputs, source_owner)
    }
}

impl PreparedPrerequisiteClaims {
    #[doc(hidden)]
    pub fn claim_batch_for_test(&self) -> &Option<super::super::claim_readthrough::ClaimBatch> {
        &self.claim_batch
    }
}

impl PreparedPrerequisiteClaims {
    #[doc(hidden)]
    pub fn predecessors_for_test(&self) -> &Vec<Arc<WorthQueryOutputDemandKey>> {
        &self.predecessors
    }
}

impl PreparedPrerequisiteClaims {
    #[doc(hidden)]
    pub fn claim_batch_mut_for_test(&mut self) -> &mut Option<super::super::claim_readthrough::ClaimBatch> {
        &mut self.claim_batch
    }
}

impl PreparedPrerequisiteClaims {
    #[doc(hidden)]
    pub fn predecessors_mut_for_test(&mut self) -> &mut Vec<Arc<WorthQueryOutputDemandKey>> {
        &mut self.predecessors
    }
}
