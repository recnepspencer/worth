use std::sync::Arc;

use super::{DemandRegistryState, WorthQueryOutputDemandKey};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

/// Bound the selected hash lookup and tree navigation before any prerequisite
/// membership mutates. The tree can grow by one member per selected key.
pub(super) fn predecessor_lookup_work(
    state: &DemandRegistryState,
    predecessors: &[Arc<WorthQueryOutputDemandKey>],
    maximum_key_work: usize,
) -> Result<usize, WorthQueryOutputDemandDenial> {
    let prospective_members = state
        .required_keys
        .len()
        .checked_add(predecessors.len())
        .and_then(|count| count.checked_add(1))
        .ok_or_else(work_denial)?;
    let tree_levels = usize::BITS as usize - prospective_members.leading_zeros() as usize;
    // records.get, required_keys.contains, prepare, then install. A B-tree
    // node may compare every resident key on each prospective level.
    let per_key = tree_levels
        .checked_mul(36)
        .and_then(|visits| visits.checked_add(1))
        .and_then(|visits| visits.checked_mul(maximum_key_work))
        .ok_or_else(work_denial)?;
    predecessors
        .len()
        .checked_mul(per_key)
        .ok_or_else(work_denial)
}

/// The prior edge set is released after the product effect, so charge its
/// selected required-index lookups and removals while preparation can stop.
pub(super) fn prior_release_work(
    required_members: usize,
    prior_count: usize,
    maximum_key_work: usize,
) -> Result<usize, WorthQueryOutputDemandDenial> {
    let levels = usize::BITS as usize - required_members.max(1).leading_zeros() as usize;
    prior_count
        .checked_mul(
            levels
                .checked_mul(12)
                .and_then(|n| n.checked_add(1))
                .ok_or_else(work_denial)?,
        )
        .and_then(|visits| visits.checked_mul(maximum_key_work))
        .ok_or_else(work_denial)
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "prerequisite registry lookup exceeds request work",
    )
}
