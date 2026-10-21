//! Retained consumed evidence comes only from a bound current accepted row.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::BoundCurrentAcceptedOutput;

impl ConsumedOutputEvidence {
    pub(in crate::domain_computation::primary_graph) fn retain_bound(
        bound: BoundCurrentAcceptedOutput<'_>,
    ) -> Result<Self, ConsumedOutputVerificationStop> {
        let facts = bound.consumed_facts();
        let selected = bound.consumed_root();
        let witness = bound
            .consumed_witness()
            .ok_or(ConsumedOutputVerificationStop::Unavailable)?;

        Ok(Self::new(
            facts.computation(),
            Arc::clone(bound.consumed_identity()),
            facts.clone(),
            Arc::clone(bound.consumed_upstream()),
            None,
            Arc::clone(witness),
            Arc::new(selected.clone()),
        ))
    }

    pub(super) fn matches_output_witness(
        &self,
        witness: &SealedNativeOutputWitness,
    ) -> Result<bool, worth_relational::facade::mvcc::CompanionPreflightStop> {
        match self
            .native_output_witness
            .as_ref()
            .and_then(|cell| cell.get())
        {
            Some(prior) => prior.same_output_as(witness),
            None => Ok(false),
        }
    }
}
