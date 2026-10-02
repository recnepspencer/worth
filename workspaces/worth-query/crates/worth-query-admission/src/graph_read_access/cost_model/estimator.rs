use super::{
    WorthQueryGraphReadAccessCostEstimate, WorthQueryGraphReadCostAttributionRow,
    WorthQueryGraphReadCostEstimateCounters, WorthQueryGraphReadCostEvidence,
    WorthQueryGraphReadIntrinsicCostContribution, WorthQueryGraphReadIntrinsicCostEstimate,
    WorthQueryGraphReadMemoryByteEstimate, WorthQueryGraphReadSupportedCostContribution,
    WorthQueryGraphReadSupportedCostEstimate,
};
use crate::graph_read_access::digest_text::{admitted_digest_text, AdmittedDigestTextStop};
use crate::graph_read_access::{
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadAccessRequirementRow,
    WorthQueryGraphReadAccessRequirementSet, WorthQueryGraphReadPredicateFamily,
    WorthQueryGraphReadResultPressure,
};

const UNKNOWN_RELATION_TOUCH_UNIT: usize = 32;
const UNKNOWN_FRONTIER_UNIT_BYTES: usize = 256;
const UNKNOWN_SET_UNIT_BYTES: usize = 192;
const UNKNOWN_FIELD_SUPPORT_BYTES: usize = 512;
const UNKNOWN_PROOF_BYTES: usize = 256;

pub fn estimate_graph_read_access_cost(
    requirements: &WorthQueryGraphReadAccessRequirementSet,
    evidence: WorthQueryGraphReadCostEvidence,
) -> WorthQueryGraphReadAccessCostEstimate {
    let attribution_rows = estimate_cost_attribution_rows(requirements.rows());
    let intrinsic = estimate_intrinsic_cost(&attribution_rows);
    let supported = estimate_supported_cost(&attribution_rows);
    let counters = estimate_cost_counters(requirements.rows());
    WorthQueryGraphReadAccessCostEstimate::new(
        *requirements.digest().as_digest(),
        &evidence,
        intrinsic,
        supported,
        counters,
        attribution_rows,
    )
}

pub(crate) fn estimate_graph_read_access_cost_admitted<Stop>(
    requirements: &WorthQueryGraphReadAccessRequirementSet,
    evidence: WorthQueryGraphReadCostEvidence,
    mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<WorthQueryGraphReadAccessCostEstimate, AdmittedDigestTextStop<Stop>> {
    let rows = requirements.rows();
    let scan = u64::try_from(rows.len())
        .ok()
        .and_then(|count| count.checked_mul(9))
        .ok_or(AdmittedDigestTextStop::AccountingOverflow)?;
    admit(scan, 0).map_err(AdmittedDigestTextStop::Admission)?;
    let attribution_rows = estimate_cost_attribution_rows_admitted(rows, &mut admit)?;
    let intrinsic = estimate_intrinsic_cost(&attribution_rows);
    let supported = estimate_supported_cost(&attribution_rows);
    let counters = estimate_cost_counters(rows);
    WorthQueryGraphReadAccessCostEstimate::new_admitted(
        *requirements.digest().as_digest(),
        &evidence,
        intrinsic,
        supported,
        counters,
        attribution_rows,
        admit,
    )
}

pub(crate) fn estimate_cost_attribution_rows_admitted<Stop>(
    rows: &[WorthQueryGraphReadAccessRequirementRow],
    mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<Vec<WorthQueryGraphReadCostAttributionRow>, AdmittedDigestTextStop<Stop>> {
    let count =
        u64::try_from(rows.len()).map_err(|_| AdmittedDigestTextStop::AccountingOverflow)?;
    let backing = rows
        .len()
        .checked_mul(std::mem::size_of::<WorthQueryGraphReadCostAttributionRow>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(AdmittedDigestTextStop::AccountingOverflow)?;
    admit(count, backing).map_err(AdmittedDigestTextStop::Admission)?;
    let mut attribution = Vec::with_capacity(rows.len());
    for row in rows {
        let visits = row
            .digest_visit_work()
            .ok_or(AdmittedDigestTextStop::AccountingOverflow)?;
        let digest =
            admitted_digest_text(visits, |output| row.write_digest_part(output), &mut admit)?;
        // The shared builder reads kind, intrinsic dimensions and supported
        // dimensions after the digest has been materialized.
        admit(11, 0).map_err(AdmittedDigestTextStop::Admission)?;
        attribution.push(estimate_cost_attribution_row_with_digest(row, digest));
    }
    Ok(attribution)
}

fn estimate_intrinsic_cost(
    rows: &[WorthQueryGraphReadCostAttributionRow],
) -> WorthQueryGraphReadIntrinsicCostEstimate {
    let frontier_breadth = rows
        .iter()
        .map(|row| row.intrinsic().frontier_breadth())
        .sum::<usize>()
        .max(1);
    let edge_touches = rows
        .iter()
        .map(|row| row.intrinsic().edge_touches())
        .sum::<usize>();
    let candidate_roots = rows
        .iter()
        .map(|row| row.intrinsic().candidate_roots())
        .sum::<usize>()
        .max(1);
    let intermediate_set_size = rows
        .iter()
        .map(|row| row.intrinsic().intermediate_set_size())
        .sum::<usize>();
    WorthQueryGraphReadIntrinsicCostEstimate::new(
        frontier_breadth,
        edge_touches,
        candidate_roots,
        intermediate_set_size,
    )
}

fn estimate_supported_cost(
    rows: &[WorthQueryGraphReadCostAttributionRow],
) -> WorthQueryGraphReadSupportedCostEstimate {
    let mut memory = WorthQueryGraphReadMemoryByteEstimate::empty();
    let allocation_lifecycle_count = rows
        .iter()
        .map(|row| row.supported().allocation_lifecycle_count())
        .sum::<usize>();
    for row in rows {
        add_memory_contribution(&mut memory, row.supported().memory());
    }
    WorthQueryGraphReadSupportedCostEstimate::new(memory, allocation_lifecycle_count)
}

fn estimate_cost_attribution_rows(
    rows: &[WorthQueryGraphReadAccessRequirementRow],
) -> Vec<WorthQueryGraphReadCostAttributionRow> {
    rows.iter().map(estimate_cost_attribution_row).collect()
}

fn estimate_cost_attribution_row(
    row: &WorthQueryGraphReadAccessRequirementRow,
) -> WorthQueryGraphReadCostAttributionRow {
    estimate_cost_attribution_row_with_digest(row, row.digest_part())
}

fn estimate_cost_attribution_row_with_digest(
    row: &WorthQueryGraphReadAccessRequirementRow,
    digest: String,
) -> WorthQueryGraphReadCostAttributionRow {
    WorthQueryGraphReadCostAttributionRow::new(
        digest,
        row.kind().clone(),
        intrinsic_contribution(row),
        supported_contribution(row),
    )
}

fn intrinsic_contribution(
    row: &WorthQueryGraphReadAccessRequirementRow,
) -> WorthQueryGraphReadIntrinsicCostContribution {
    WorthQueryGraphReadIntrinsicCostContribution::new(
        relation_frontier_breadth(row).unwrap_or(0),
        relation_edge_touch_estimate(row).unwrap_or(0),
        candidate_root_pressure(row),
        intermediate_set_pressure(row),
    )
}

fn supported_contribution(
    row: &WorthQueryGraphReadAccessRequirementRow,
) -> WorthQueryGraphReadSupportedCostContribution {
    let mut memory = WorthQueryGraphReadMemoryByteEstimate::empty();
    let mut allocation_lifecycle_count = 0;
    match row.kind() {
        WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency => {
            memory.add_adjacency_bytes(relation_memory_bytes(row));
        }
        WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency => {
            memory.add_reverse_adjacency_bytes(relation_memory_bytes(row));
        }
        WorthQueryGraphReadAccessRequirementKind::TraversalWorkset => {
            memory.add_frontier_bytes(frontier_memory_bytes(row));
        }
        WorthQueryGraphReadAccessRequirementKind::VisitedSet => {
            memory.add_visited_bytes(set_memory_bytes(row));
        }
        WorthQueryGraphReadAccessRequirementKind::DedupSet => {
            memory.add_dedup_bytes(set_memory_bytes(row));
        }
        WorthQueryGraphReadAccessRequirementKind::PredicateSupport => {
            memory.add_predicate_bytes(predicate_memory_bytes(row));
        }
        WorthQueryGraphReadAccessRequirementKind::OrderingSupport => {
            memory.add_ordering_bytes(ordering_memory_bytes(row));
        }
        WorthQueryGraphReadAccessRequirementKind::ProofSupport => {
            memory.add_proof_bytes(UNKNOWN_PROOF_BYTES);
        }
        WorthQueryGraphReadAccessRequirementKind::ResultBuffer => {
            memory.add_result_bytes(result_memory_bytes(row));
        }
        WorthQueryGraphReadAccessRequirementKind::MaterializationLifecycle
        | WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport
        | WorthQueryGraphReadAccessRequirementKind::DomainOperationCapabilityRegistration => {
            allocation_lifecycle_count = 1;
        }
    }
    WorthQueryGraphReadSupportedCostContribution::new(memory, allocation_lifecycle_count)
}

fn add_memory_contribution(
    target: &mut WorthQueryGraphReadMemoryByteEstimate,
    contribution: &WorthQueryGraphReadMemoryByteEstimate,
) {
    target.add_adjacency_bytes(contribution.adjacency_bytes());
    target.add_reverse_adjacency_bytes(contribution.reverse_adjacency_bytes());
    target.add_frontier_bytes(contribution.frontier_bytes());
    target.add_visited_bytes(contribution.visited_bytes());
    target.add_dedup_bytes(contribution.dedup_bytes());
    target.add_predicate_bytes(contribution.predicate_bytes());
    target.add_ordering_bytes(contribution.ordering_bytes());
    target.add_proof_bytes(contribution.proof_bytes());
    target.add_result_bytes(contribution.result_bytes());
}

fn estimate_cost_counters(
    rows: &[WorthQueryGraphReadAccessRequirementRow],
) -> WorthQueryGraphReadCostEstimateCounters {
    let estimated_relation_row_count = rows
        .iter()
        .filter(|row| row.relation_name().is_some())
        .count();
    let estimated_workset_row_count = rows
        .iter()
        .filter(|row| {
            matches!(
                row.kind(),
                WorthQueryGraphReadAccessRequirementKind::TraversalWorkset
                    | WorthQueryGraphReadAccessRequirementKind::VisitedSet
                    | WorthQueryGraphReadAccessRequirementKind::DedupSet
            )
        })
        .count();
    let estimated_buffer_row_count = rows
        .iter()
        .filter(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::ResultBuffer)
        .count();
    WorthQueryGraphReadCostEstimateCounters::new(
        rows.len(),
        estimated_relation_row_count,
        estimated_workset_row_count,
        estimated_buffer_row_count,
    )
}

fn relation_frontier_breadth(row: &WorthQueryGraphReadAccessRequirementRow) -> Option<usize> {
    row.relation_name()
        .map(|_| row.relation_depth().unwrap_or(1).max(1) * UNKNOWN_RELATION_TOUCH_UNIT)
}

fn relation_edge_touch_estimate(row: &WorthQueryGraphReadAccessRequirementRow) -> Option<usize> {
    row.relation_name()
        .map(|_| row.relation_depth().unwrap_or(1).max(1) * UNKNOWN_RELATION_TOUCH_UNIT)
}

fn candidate_root_pressure(row: &WorthQueryGraphReadAccessRequirementRow) -> usize {
    match row.predicate_family() {
        Some(WorthQueryGraphReadPredicateFamily::None) | None => 0,
        Some(WorthQueryGraphReadPredicateFamily::Equality) => 4,
        Some(WorthQueryGraphReadPredicateFamily::Range)
        | Some(WorthQueryGraphReadPredicateFamily::Membership) => 8,
        Some(WorthQueryGraphReadPredicateFamily::Text)
        | Some(WorthQueryGraphReadPredicateFamily::Presence)
        | Some(WorthQueryGraphReadPredicateFamily::Mixed) => 16,
    }
}

fn intermediate_set_pressure(row: &WorthQueryGraphReadAccessRequirementRow) -> usize {
    match row.kind() {
        WorthQueryGraphReadAccessRequirementKind::TraversalWorkset
        | WorthQueryGraphReadAccessRequirementKind::VisitedSet
        | WorthQueryGraphReadAccessRequirementKind::DedupSet => {
            row.relation_depth().unwrap_or(1).max(1) * UNKNOWN_RELATION_TOUCH_UNIT
        }
        _ => 0,
    }
}

fn relation_memory_bytes(row: &WorthQueryGraphReadAccessRequirementRow) -> usize {
    row.relation_depth().unwrap_or(1).max(1) * UNKNOWN_RELATION_TOUCH_UNIT * 8
}

fn frontier_memory_bytes(row: &WorthQueryGraphReadAccessRequirementRow) -> usize {
    row.relation_depth().unwrap_or(1).max(1) * UNKNOWN_FRONTIER_UNIT_BYTES
}

fn set_memory_bytes(row: &WorthQueryGraphReadAccessRequirementRow) -> usize {
    row.relation_depth().unwrap_or(1).max(1) * UNKNOWN_SET_UNIT_BYTES
}

fn predicate_memory_bytes(row: &WorthQueryGraphReadAccessRequirementRow) -> usize {
    row.predicate_field_authorities().len().max(1) * UNKNOWN_FIELD_SUPPORT_BYTES
}

fn ordering_memory_bytes(row: &WorthQueryGraphReadAccessRequirementRow) -> usize {
    row.ordering_field_authorities().len().max(1) * UNKNOWN_FIELD_SUPPORT_BYTES
}

fn result_memory_bytes(row: &WorthQueryGraphReadAccessRequirementRow) -> usize {
    let per_result_bytes: usize = match row.result_pressure() {
        Some(WorthQueryGraphReadResultPressure::Detail) => 256,
        Some(WorthQueryGraphReadResultPressure::CollectionNarrow) => 1024,
        Some(WorthQueryGraphReadResultPressure::CollectionWide) | None => 2048,
    };
    per_result_bytes.saturating_mul(row.maximum_cardinality().unwrap_or(1).max(1))
}
