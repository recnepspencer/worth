use std::sync::Arc;

use super::super::{DemandRegistryState, WorthQueryOutputDemandKey};
use super::{capacity_denial, coverage_denial};
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

pub(super) fn validate_predecessors(
    state: &DemandRegistryState,
    predecessors: &[Arc<WorthQueryOutputDemandKey>],
) -> Result<usize, WorthQueryOutputDemandDenial> {
    let mut bytes = 0usize;
    for key in predecessors {
        let record = state
            .records
            .get(key.as_ref())
            .ok_or_else(coverage_denial)?;
        if !record.has_cached_ready() || record.framework_required_count == usize::MAX {
            return Err(coverage_denial());
        }
        if !state.required_keys.contains(key.as_ref()) {
            bytes = bytes
                .checked_add(
                    super::super::required_members::member_bytes(key)
                        .ok_or_else(capacity_denial)?,
                )
                .ok_or_else(capacity_denial)?;
        }
    }
    Ok(bytes)
}
