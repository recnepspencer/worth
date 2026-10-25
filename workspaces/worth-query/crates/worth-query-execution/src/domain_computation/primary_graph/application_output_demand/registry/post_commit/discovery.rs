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
        let same_row = {
            #[cfg(test)]
            super::super::demand_key::observe_row_comparison();
            other == key
        };
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

#[cfg(test)]
mod tests {
    use super::super::super::tests::key;
    use super::super::super::tests::occurrence;
    use super::super::super::tests::record;
    use super::*;
    #[test]
    fn registry_completed_discovery_body_keeps_its_occurrence_bound() {
        let receipts = super::super::super::tests::walk_bounds::receipt_batch();
        let mut contacts = Vec::new();
        for (neighbors, entries) in [(1, 1), (64, 4)] {
            let registry = WorthQueryOutputDemandRegistry::default();
            let selected = key("posted-discovery", 3, 1);
            let mut state = registry.state.lock().unwrap();
            for generation in 1..=3 {
                state.records.insert(
                    key("posted-discovery", generation, 1),
                    record(occurrence(), DemandState::Running, 1),
                );
            }
            super::super::super::tests::walk_bounds::add_custody(
                &mut state,
                &receipts[..entries],
                neighbors,
            );
            let _published =
                SupersededSettlements::new(registry.clone(), Arc::new(selected.clone()));
            let before = super::super::super::record_map::occurrence_rows_visited();
            assert!(after_publication(&state, &selected).is_empty());
            contacts.push(super::super::super::record_map::occurrence_rows_visited() - before);
        }
        assert_eq!(contacts, [3, 3]);
    }
}
