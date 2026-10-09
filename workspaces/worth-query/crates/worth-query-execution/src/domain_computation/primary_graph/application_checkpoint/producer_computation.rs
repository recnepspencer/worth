//! Original producer facts admitted under the checkpoint's exclusion contract.
use super::facts;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact,
    output_lineage::ComputationSourceEvidence,
};
use std::sync::Arc;

/// Only the checkpoint decoder can create this provenance. Current-version
/// producer facts cannot originate from a known own-write-stale computation.
pub(in crate::domain_computation::primary_graph) struct CheckpointProducerFacts {
    facts: Arc<[WorthQueryApplicationObservedFact]>,
}

impl CheckpointProducerFacts {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        Arc<[WorthQueryApplicationObservedFact]>,
        ComputationSourceEvidence,
    ) {
        let evidence = ComputationSourceEvidence::from_checkpoint_facts(&self);
        (self.facts, evidence)
    }
}

pub(in crate::domain_computation::primary_graph) fn decode(
    bytes: &[u8],
    wire_version: u16,
    request: Option<
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    >,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<CheckpointProducerFacts, facts::FactDecodeDenial> {
    facts::decode_for_wire_version(bytes, wire_version, request, allocation_policy)
        .map(|facts| CheckpointProducerFacts { facts })
}
