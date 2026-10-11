//! One settlement selection body, with request or completed-publication payment.
use super::super::refreshed_rejoin::{infallible, occurrence_rows, OccurrenceSearchCharge};
#[cfg(test)]
use super::super::refreshed_rejoin::{WalkAdmission, WalkKind};
#[cfg(test)]
use super::admit_slots;
use super::{DemandRegistryState, Identity, WorthQueryOutputDemandKey};
#[cfg(test)]
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
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

// Only this completed-publication entry can construct this payer. Its bound
// is one occurrence's rows and their superseded settlement postings.
struct PublishedOccurrenceWalk;
impl OccurrenceSearchCharge for PublishedOccurrenceWalk {
    type Refusal = Infallible;
    fn occurrence_path(&mut self, _: usize) -> Result<(), Infallible> {
        Ok(())
    }
    fn inspect_row(&mut self) -> Result<(), Infallible> {
        Ok(())
    }
}
impl DiscoveryCharge for PublishedOccurrenceWalk {
    fn slots(&mut self, _: usize) -> Result<(), Infallible> {
        Ok(())
    }
}

pub(super) fn after_publication(
    state: &DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) -> Vec<Identity> {
    infallible(discover(state, key, &mut PublishedOccurrenceWalk))
}

#[cfg(test)]
pub(super) fn under_request(
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
