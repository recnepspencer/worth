//! Canonical maintenance selects the prefix of the total declared key.
use super::{RetainedComponentPins, RuntimeWorldRetentionOwner};
use crate::retention::registry::RetentionReclamationReport;
use crate::retention::unique_component_pin::ExactComponentBasisKey;

impl<D, I, T> RuntimeWorldRetentionOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn reclaim(&self, requested: usize) -> RetentionReclamationReport {
        let mut state = self.lock();
        let keys = canonical_candidates(&state.entries, requested);
        let mut reclaimed = 0;
        for key in &keys {
            let eligible = state.entries.get(key).is_some_and(|entry| {
                entry.owner_lease.is_none()
                    && entry.counts.is_zero()
                    && !state.flights.contains_key(key)
            });
            state.costs.reclamation_entries_examined =
                state.costs.reclamation_entries_examined.saturating_add(1);
            if eligible && state.entries.remove(key).is_some() {
                state.unique_slots -= 1;
                reclaimed += 1;
                state.costs.reclamation_entries_reclaimed =
                    state.costs.reclamation_entries_reclaimed.saturating_add(1);
            }
        }
        RetentionReclamationReport {
            requested,
            examined: keys.len(),
            reclaimed,
            remaining_unique_pins: state.unique_slots,
        }
    }
}

fn canonical_candidates(
    entries: &RetainedComponentPins,
    maximum: usize,
) -> Vec<ExactComponentBasisKey> {
    // This maintenance pass visits the carried order in O(n) time and scratch.
    // Ordinary exact admission lookup remains hash-indexed, without a walk.
    entries
        .by_declared_basis()
        .take(maximum)
        .map(|(key, _)| key.clone())
        .collect()
}
