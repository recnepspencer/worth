//! Performed output facts share the sealed native witness's role inventory.

use std::sync::Arc;

use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;

use super::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct RegisteredOutputFacts {
    pub(super) facts: Arc<[Fact]>,
}

impl SourceInvalidationOwner {
    /// This fallible derived preparation runs after the performed publication.
    /// A stop requires full verification on the recorded lineage row; it never
    /// turns an incomplete output posting set into an authoritative Clean row.
    pub(in crate::domain_computation::primary_graph) fn prepare_performed_output_facts(
        &self,
        witness: &SealedNativeOutputWitness,
    ) -> Result<Option<RegisteredOutputFacts>, CompanionPreflightStop> {
        let Some(projection) = witness.prepare_fact_projection()? else {
            return Ok(None);
        };
        let count = projection.count();
        let mut facts = Vec::with_capacity(count);
        projection.append_into(&mut facts);
        let facts = Arc::from(facts.into_boxed_slice());
        Ok(Some(RegisteredOutputFacts { facts }))
    }
}

impl RegisteredOutputFacts {
    pub(in crate::domain_computation::primary_graph) fn facts(&self) -> &[Fact] {
        &self.facts
    }
}
