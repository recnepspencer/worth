//! One settlement selection body, with request or completed-publication payment.
use super::super::refreshed_rejoin::{infallible, occurrence_rows, OccurrenceSearchCharge};
#[cfg(test)]
use super::super::refreshed_rejoin::{WalkAdmission, WalkKind};
#[cfg(test)]
use super::super::settlement_retirement::admit_slots;
use super::*;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
type Identity = Arc<RecordedSettlementIdentity>;
use super::walk_charge::PostCommitWalk;
#[cfg(test)]
use crate::domain_computation::primary_graph::InvalidationEditAdmission;
#[cfg(test)]
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use core::convert::Infallible;
use std::sync::Arc;

trait DiscoveryCharge: OccurrenceSearchCharge {
    fn slots(&mut self, count: usize) -> Result<(), Self::Refusal>;
}
#[cfg(test)]
impl DiscoveryCharge for WalkAdmission<'_> {
    fn slots(&mut self, count: usize) -> Result<(), Self::Refusal> {
        admit_slots(count, self.admission_mut())
    }
}

impl DiscoveryCharge for PostCommitWalk {
    fn slots(&mut self, _: usize) -> Result<(), Infallible> {
        Ok(())
    }
}

pub(in super::super) fn after_publication(
    state: &DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
    published: &SupersededSettlements,
) -> Vec<Identity> {
    infallible(discover(
        state,
        key,
        &mut PostCommitWalk::after_registered(published),
    ))
}

#[cfg(test)]
pub(in super::super) fn under_request(
    state: &DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
    admission: &mut InvalidationEditAdmission,
) -> Result<Vec<Identity>, WorthQueryOutputDemandDenial> {
    state.charge_record_lookup(key, admission)?;
    discover(
        state,
        key,
        &mut WalkAdmission::new(admission, WalkKind::SettlementRetirement),
    )
}

fn discover<C: DiscoveryCharge>(
    state: &DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
    charge: &mut C,
) -> Result<Vec<Identity>, C::Refusal> {
    if !state.records.contains_key(key) {
        return Ok(Vec::new());
    }
    let mut candidates = Vec::new();
    for (other, record) in occurrence_rows(&state.records, key, charge)? {
        charge.inspect_row()?;
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
        charge.slots(superseded)?;
        candidates.extend(
            record.settlements[..superseded]
                .iter()
                .map(|(identity, _)| Arc::clone(identity)),
        );
    }
    Ok(candidates)
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::{key, occurrence, record};
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
            let published =
                SupersededSettlements::new(registry.clone(), Arc::new(selected.clone()));
            let before = super::super::super::record_map::occurrence_rows_visited();
            assert!(after_publication(&state, &selected, &published).is_empty());
            contacts.push(super::super::super::record_map::occurrence_rows_visited() - before);
        }
        assert_eq!(contacts, [3, 3]);
    }
}
