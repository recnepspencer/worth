//! Retained consumed evidence comes only from a bound current accepted row.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::BoundCurrentAcceptedOutput;

impl ConsumedOutputEvidence {
    pub(in crate::domain_computation::primary_graph) fn retain_bound(
        bound: BoundCurrentAcceptedOutput<'_>,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, ConsumedOutputVerificationStop> {
        let facts = bound.consumed_facts();
        let selected = bound.consumed_root();
        let witness = bound
            .consumed_witness()
            .ok_or(ConsumedOutputVerificationStop::Unavailable)?;
        admission
            .charge_external_work(
                8 + size_of::<PositionedRelationalSnapshot>() as u64
                    + selected.branch_id().0.len() as u64,
            )
            .map_err(map_admission_stop)?;
        let capacity = owner
            .retain_consumed_output(facts, selected, Self::metadata_bytes(), admission)
            .map_err(map_admission_stop)?;
        Ok(Self::new(
            facts.computation(),
            Arc::clone(bound.consumed_identity()),
            facts.clone(),
            Arc::clone(bound.consumed_upstream()),
            None,
            Arc::clone(witness),
            Arc::new(selected.clone()),
            capacity,
        ))
    }

    pub(super) fn matches_output_witness(
        &self,
        witness: &SealedNativeOutputWitness,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, worth_relational::facade::mvcc::CompanionPreflightStop> {
        admission.charge_external_work(1)?;
        match self
            .native_output_witness
            .as_ref()
            .and_then(|cell| cell.get())
        {
            Some(prior) => prior.same_output_as(witness, admission),
            None => Ok(false),
        }
    }
}
