//! Capacity for the fields that semantic projection actually copies.
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::NodeColdData;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work,
};
use crate::data::trace::{RuntimeArtifactFinalizeImage, RuntimeArtifactState};
use crate::diagnostics::facts::{ExplanationFact, ProvenanceFact};
use crate::diagnostics::lineage::LineageRecord;

impl SignalGraph {
    /// Fixed part of the projection and fact-writer envelopes. Operational
    /// version overrides and invalidation companions never enter projection.
    /// Incoming result payload is admitted separately through its grant.
    pub(crate) fn epoch_semantic_diagnostic_fixed_bound(
        &self,
        node: NodeId,
        rewiring_heap: u64,
        links: usize,
        work: &mut Work<'_>,
    ) -> Result<u64, SignalError> {
        let contract = self.get_contract(node)?;
        let condition = &self.node_eval_config(node)?.condition;
        let index = node.index() as usize;
        let source = (|| {
            self.arena.warm[index]
                .runtime_artifact_state
                .retained_heap_charge(work)?
                .checked_add(self.arena.cold[index].retained_heap_charge(work)?)?
                // A computed result may install these owners even on a node
                // that has never held a runtime artifact or cold companion.
                .checked_add(arc_allocation_charge::<RuntimeArtifactState>()?)?
                .checked_add(Charge::capacity::<NodeColdData>(1)?)?
                .checked_add(
                    contract
                        .semantics
                        .partition_scope
                        .retained_heap_charge(work)?,
                )?
                .checked_add(condition.retained_heap_charge(work)?)
        })()
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        let source = source
            .bytes()
            .checked_add(rewiring_heap)
            .ok_or_else(|| SignalError::invalid_input("epoch diagnostic source overflow"))?;
        // Projection construction and the subsequent two-fact writer each
        // consume an envelope; their result-sized payload copies are separate.
        semantic_projection_allocation(source, links)?
            .checked_mul(2)
            .ok_or_else(|| SignalError::invalid_input("epoch diagnostic bound overflow"))
    }
}

pub(super) fn semantic_projection_allocation(
    source: u64,
    links: usize,
) -> Result<u64, SignalError> {
    source
        .checked_mul(16)
        .and_then(|bytes| {
            bytes.checked_add((links as u64).checked_mul(
                (std::mem::size_of::<crate::logic::explain::CausalLink>() + 256) as u64 * 16,
            )?)
        })
        .and_then(|bytes| {
            bytes.checked_add(
                (std::mem::size_of::<crate::logic::explain::NodeExplanation>()
                    + std::mem::size_of::<ExplanationFact>()
                    + std::mem::size_of::<ProvenanceFact>()
                    + std::mem::size_of::<NodeColdData>()
                    + std::mem::size_of::<RuntimeArtifactFinalizeImage>()
                    + std::mem::size_of::<LineageRecord>()) as u64,
            )
        })
        .ok_or_else(|| SignalError::invalid_input("epoch diagnostic projection size overflow"))
}

#[cfg(test)]
mod tests;
