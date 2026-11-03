//! Settlement discovery after publication cannot refuse its occurrence walk.
use super::super::refreshed_rejoin::occurrence_rows;
use super::*;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
type Identity = Arc<RecordedSettlementIdentity>;

use std::sync::Arc;

pub(in super::super) fn after_publication(
    state: &DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) -> Vec<Identity> {
    discover(state, key)
}

fn discover(state: &DemandRegistryState, key: &WorthQueryOutputDemandKey) -> Vec<Identity> {
    if !state.records.contains_key(key) {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    for (other, record) in occurrence_rows(&state.records, key) {
        let same_row = other == key;
        let superseded = if same_row {
            record.settlements.len().saturating_sub(1)
        } else if key.replacement_order(other) == Some(std::cmp::Ordering::Greater) {
            record.settlements.len()
        } else {
            0
        };
        candidates.extend(record.settlements[..superseded].iter().map(Arc::clone));
    }
    candidates
}
