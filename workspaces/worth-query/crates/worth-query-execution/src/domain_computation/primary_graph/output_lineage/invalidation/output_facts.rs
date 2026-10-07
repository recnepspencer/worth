//! Performed output facts share the sealed native witness's role inventory.

use std::{mem::size_of, sync::Arc};

use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::{
    execution_runtime::source_invalidation::RetainedInvalidationCapacity,
    primary_graph::WorthQueryApplicationObservedFact as Fact,
};

use super::{
    fact_retention::arc_slice_bytes, retention, InvalidationEditAdmission, SourceInvalidationOwner,
};
use crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct RegisteredOutputFacts {
    pub(super) facts: Arc<[Fact]>,
    pub(super) _capacity: Arc<RetainedInvalidationCapacity>,
}

impl SourceInvalidationOwner {
    /// This fallible derived preparation runs after the performed publication.
    /// A stop requires full verification on the recorded lineage row; it never
    /// turns an incomplete output posting set into an authoritative Clean row.
    pub(super) fn prepare_performed_output_facts(
        &self,
        witness: &SealedNativeOutputWitness,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<RegisteredOutputFacts>, CompanionPreflightStop> {
        let Some(projection) = witness.prepare_fact_projection(admission)? else {
            return Ok(None);
        };
        let count = projection.count();
        let vec_bytes = count
            .checked_mul(size_of::<Fact>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let arc_bytes = arc_slice_bytes::<Fact>(count)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let retained_bytes = arc_bytes
            .checked_add(projection.retained_payload_bytes())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let peak = vec_bytes
            .checked_add(retained_bytes)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        admission.admit_read_scratch(peak)?;
        let copy_visits = u64::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(2))
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        admission.charge_external_work(copy_visits)?;
        let capacity = retention::reserve(&self.resources, retained_bytes, admission)?;
        let mut facts = Vec::with_capacity(count);
        projection.append_into(&mut facts);
        let facts = Arc::from(facts.into_boxed_slice());
        Ok(Some(RegisteredOutputFacts {
            facts,
            _capacity: capacity,
        }))
    }
}
