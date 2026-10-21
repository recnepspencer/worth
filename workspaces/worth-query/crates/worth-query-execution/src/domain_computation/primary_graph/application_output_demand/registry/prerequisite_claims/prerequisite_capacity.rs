use std::sync::Arc;

use super::super::{DemandRegistryState, WorthQueryOutputDemandKey};
use super::coverage_denial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

pub(super) fn validate_predecessors(
    state: &DemandRegistryState,
    predecessors: &[Arc<WorthQueryOutputDemandKey>],
) -> Result<(), WorthQueryOutputDemandDenial> {
    for key in predecessors {
        let record = state
            .records
            .get(key.as_ref())
            .ok_or_else(coverage_denial)?;
        if !record.has_cached_ready() || record.framework_required_count == usize::MAX {
            return Err(coverage_denial());
        }
    }
    Ok(())
}
