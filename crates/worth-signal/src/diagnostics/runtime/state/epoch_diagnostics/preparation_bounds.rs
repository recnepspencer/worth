//! O(1) selected-root shape bound for one diagnostics epoch.

use std::sync::Arc;

use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::retained_storage::{
    arc_allocation_charge, ordered_edit_growth_charge, ordered_index_charge,
    RetainedStorageCharge as Charge, RetainedStorageForkCharge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, SignalConditionalRetentionReservation as Reservation,
};
use crate::diagnostics::facts::{ExplanationFact, ProvenanceFact};
use crate::diagnostics::lineage::LineageRecord;
use crate::diagnostics::policy::ArtifactRetentionPolicy;

use super::{map_node_edit_accounting, DiagnosticsState};

/// Physical request growth for the selected draft. Historical frames and
/// immutable map bases are shared; `reserves` below still holds their full
/// logical custody in the retained ledger.
pub(super) fn request_growth(
    state: &DiagnosticsState,
    maximum_fact_nodes: usize,
    artifact: RetainedStorageForkCharge,
    node: RetainedStorageForkCharge,
    explanation: RetainedStorageForkCharge,
    provenance: RetainedStorageForkCharge,
) -> Result<Charge, SignalError> {
    let history_end = state
        .lineage_records
        .len()
        .checked_add(maximum_fact_nodes)
        .ok_or_else(|| SignalError::invalid_input("diagnostics history bound overflow"))?;
    let limit = state
        .installed_retention_budget
        .history_limit
        .max(1)
        .checked_mul(32)
        .ok_or_else(|| SignalError::invalid_input("diagnostics history limit overflow"))?;
    // Shrinking a retention policy can evict pre-existing frames on this
    // epoch. Every appended or evicted frame can edit the global history and
    // both indexed histories once; their old frames remain shared.
    let history_edits = if maximum_fact_nodes == 0 {
        0
    } else {
        maximum_fact_nodes
            .checked_add(history_end.saturating_sub(limit))
            .ok_or_else(|| SignalError::invalid_input("diagnostics history edits overflow"))?
    };
    let history_paths = ordered_edit_growth_charge::<u64, Arc<LineageRecord>>(history_end)
        .and_then(|path| path.checked_mul(history_edits))
        .and_then(|paths| paths.checked_mul(3))
        .map_err(map_node_edit_accounting)?;
    let lineage_maps = state
        .lineage_records_by_artifact
        .batch_edit_structure_growth_bound(history_edits)
        .and_then(|artifact| {
            artifact.checked_add(
                state
                    .lineage_records_by_node
                    .batch_edit_structure_growth_bound(history_edits)?,
            )
        })
        .map_err(map_node_edit_accounting)?;
    let explanation_edits = if state.installed_retention_budget.explanation_retention
        == ArtifactRetentionPolicy::Retain
    {
        maximum_fact_nodes
    } else {
        0
    };
    let provenance_edits = if state.installed_retention_budget.provenance_retention
        == ArtifactRetentionPolicy::Retain
    {
        maximum_fact_nodes
    } else {
        0
    };
    let fact_maps = state
        .explanation_facts
        .batch_edit_structure_growth_bound(explanation_edits)
        .and_then(|explanation| {
            explanation.checked_add(
                state
                    .provenance_facts
                    .batch_edit_structure_growth_bound(provenance_edits)?,
            )
        })
        .map_err(map_node_edit_accounting)?;
    let reservation_count = maximum_fact_nodes
        .checked_add(2)
        .ok_or_else(|| SignalError::invalid_input("diagnostics custody bound overflow"))?;
    let publication_frames =
        super::super::lineage_publication::epoch_lineage_publication_frame_charge()
            .and_then(|frame| frame.checked_mul(maximum_fact_nodes))
            .map_err(map_node_edit_accounting)?;
    artifact
        .source_growth
        .checked_add(node.source_growth)
        .and_then(|charge| charge.checked_add(explanation.source_growth))
        .and_then(|charge| charge.checked_add(provenance.source_growth))
        .and_then(|charge| charge.checked_add(history_paths))
        .and_then(|charge| charge.checked_add(lineage_maps))
        .and_then(|charge| charge.checked_add(fact_maps))
        .and_then(|charge| charge.checked_add(publication_frames))
        .and_then(|charge| charge.checked_add(Charge::capacity::<Reservation>(reservation_count)?))
        .and_then(|charge| {
            charge.checked_add(
                arc_allocation_charge::<Reservation>()?.checked_mul(reservation_count)?,
            )
        })
        .map_err(map_node_edit_accounting)
}

impl DiagnosticsState {
    /// Selected old fact payloads and Arc cells that the epoch writer charges
    /// again when replacing this node's explanation and provenance facts.
    pub(crate) fn epoch_fact_replacement_capacity_bound(
        &self,
        node: NodeId,
        work: &mut Work,
    ) -> Result<u64, SignalError> {
        let mut charge = Charge::ZERO;
        if self.installed_retention_budget.explanation_retention == ArtifactRetentionPolicy::Retain
        {
            work.reserve_visits(self.explanation_facts.lookup_steps())
                .map_err(map_node_edit_accounting)?;
            let old = self
                .explanation_facts
                .get(&node)
                .map(|fact| fact.retained_heap_charge(work))
                .transpose()
                .map_err(map_node_edit_accounting)?
                .unwrap_or(Charge::ZERO);
            charge = charge
                .checked_add(old)
                .and_then(|sum| sum.checked_add(arc_allocation_charge::<ExplanationFact>()?))
                .and_then(|sum| sum.checked_add(arc_allocation_charge::<NodeId>()?))
                .map_err(map_node_edit_accounting)?;
        }
        if self.installed_retention_budget.provenance_retention == ArtifactRetentionPolicy::Retain {
            work.reserve_visits(self.provenance_facts.lookup_steps())
                .map_err(map_node_edit_accounting)?;
            let old = self
                .provenance_facts
                .get(&node)
                .map(|fact| fact.retained_heap_charge(work))
                .transpose()
                .map_err(map_node_edit_accounting)?
                .unwrap_or(Charge::ZERO);
            charge = charge
                .checked_add(old)
                .and_then(|sum| sum.checked_add(arc_allocation_charge::<ProvenanceFact>()?))
                .and_then(|sum| sum.checked_add(arc_allocation_charge::<NodeId>()?))
                .map_err(map_node_edit_accounting)?;
        }
        charge
            .checked_mul(2)
            .map_err(map_node_edit_accounting)
            .map(Charge::bytes)
    }
}

pub(super) fn reserves(
    state: &DiagnosticsState,
    maximum_fact_nodes: usize,
    history: Charge,
    artifact: RetainedStorageForkCharge,
    node: RetainedStorageForkCharge,
    explanation: RetainedStorageForkCharge,
    provenance: RetainedStorageForkCharge,
) -> Result<(Charge, Charge), SignalError> {
    let lineage_roots = history
        .checked_add(artifact.retained)
        .and_then(|charge| charge.checked_add(node.retained))
        .map_err(map_node_edit_accounting)?;
    let lineage_paths = maximum_fact_nodes
        .checked_mul(tree_height(state.lineage_records.len(), maximum_fact_nodes))
        .ok_or_else(|| SignalError::invalid_input("diagnostics lineage bound overflow"))?;
    let lineage_index_growth = ordered_index_charge::<u64, Arc<LineageRecord>>(lineage_paths)
        .and_then(|charge| charge.checked_mul(3))
        .map_err(map_node_edit_accounting)?;
    let lineage_reserve = lineage_roots
        .checked_mul(2)
        .and_then(|charge| charge.checked_add(lineage_index_growth))
        .and_then(|charge| charge.checked_add(artifact.source_growth))
        .and_then(|charge| charge.checked_add(node.source_growth))
        .and_then(|charge| charge.checked_add(Charge::capacity::<Reservation>(2)?))
        .and_then(|charge| charge.checked_add(arc_allocation_charge::<Reservation>()?))
        .map_err(map_node_edit_accounting)?;
    let fact_roots = explanation
        .retained
        .checked_add(provenance.retained)
        .map_err(map_node_edit_accounting)?;
    // Persistent B-tree insertion copies at most a binary-height path.
    let explanation_paths = maximum_fact_nodes
        .checked_mul(tree_height(
            state.explanation_facts.len(),
            maximum_fact_nodes,
        ))
        .ok_or_else(|| SignalError::invalid_input("diagnostics index bound overflow"))?;
    let provenance_paths = maximum_fact_nodes
        .checked_mul(tree_height(
            state.provenance_facts.len(),
            maximum_fact_nodes,
        ))
        .ok_or_else(|| SignalError::invalid_input("diagnostics index bound overflow"))?;
    let fact_index_growth = ordered_index_charge::<
        Arc<crate::data::handle::NodeId>,
        Arc<ExplanationFact>,
    >(explanation_paths)
    .and_then(|charge| {
        charge.checked_add(ordered_index_charge::<
            Arc<crate::data::handle::NodeId>,
            Arc<ProvenanceFact>,
        >(provenance_paths)?)
    })
    .map_err(map_node_edit_accounting)?;
    let fact_reserve = fact_roots
        .checked_mul(2)
        .and_then(|charge| charge.checked_add(explanation.source_growth))
        .and_then(|charge| charge.checked_add(provenance.source_growth))
        .and_then(|charge| charge.checked_add(fact_index_growth))
        .and_then(|charge| charge.checked_add(Charge::capacity::<Reservation>(2)?))
        .map_err(map_node_edit_accounting)?;
    Ok((lineage_reserve, fact_reserve))
}

fn tree_height(existing: usize, added: usize) -> usize {
    let maximum = existing.saturating_add(added).max(1);
    (usize::BITS - maximum.leading_zeros()) as usize + 1
}
