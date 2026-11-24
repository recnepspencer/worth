//! Exact sharing costs for the prepared fork root and its selected predecessor.
use super::fork::RelationalForkDenial;
use super::reference::RelationalBranchReferenceCell;
use super::root::RelationalBranchRoot;
use crate::runtime::state::subsystems::history::history_costs::RelationalForkMaterializationCost;

pub(super) fn materialization_cost(
    target: &RelationalBranchReferenceCell,
    source_root: &RelationalBranchRoot,
) -> Result<RelationalForkMaterializationCost, RelationalForkDenial> {
    let fork_materialized_authoritative_bytes = match target
        .root()
        .filter(|target_root| !target_root.shares_authoritative_regions_with(source_root))
    {
        Some(target_root) => target_root
            .logical_partition_payload_bytes()
            .map_err(|_| RelationalForkDenial::AllocationAccountingOverflow)?,
        None => 0,
    };
    let copied_commit_envelopes = target
        .root()
        .filter(|target_root| !target_root.shares_canonical_envelope_with(source_root))
        .map_or(0, |_| 1);
    Ok(RelationalForkMaterializationCost {
        entity_count: 0,
        relation_count: 0,
        authoritative_bytes: fork_materialized_authoritative_bytes,
        copied_commit_envelopes,
    })
}
