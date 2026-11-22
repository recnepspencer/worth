use std::sync::Arc;

use super::super::{DemandRegistryState, WorthQueryOutputDemandKey};
use super::coverage_denial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

pub fn validate_predecessors<'a>(
    state: &DemandRegistryState,
    predecessors: &[Arc<WorthQueryOutputDemandKey>],
    inputs: impl Iterator<Item = &'a crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence> + Clone,
) -> Result<(), WorthQueryOutputDemandDenial> {
    for key in predecessors {
        let record = state
            .records
            .get(key.as_ref())
            .ok_or_else(coverage_denial)?;
        let actual_appended = inputs.clone().any(|input| {
            input
                .applied()
                .and_then(|link| link.prerequisite_key())
                .is_some_and(|actual| actual == key)
        });
        if (!record.has_cached_ready()
            && !(actual_appended && matches!(record.state, super::super::DemandState::Running)))
            || record.framework_required_count == usize::MAX
        {
            return Err(coverage_denial());
        }
    }
    Ok(())
}
