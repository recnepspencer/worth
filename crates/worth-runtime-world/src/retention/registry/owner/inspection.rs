use super::RuntimeWorldRetentionOwner;
use crate::inspection::{
    RuntimeWorldRetentionEntry, RuntimeWorldRetentionInspectionDenial, RuntimeWorldRetentionKey,
};
use crate::retention::RetentionReclamationReport;
impl<D, I, T> RuntimeWorldRetentionOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn snapshot(
        &self,
        active_publication_attempts: usize,
    ) -> crate::inspection::RuntimeWorldRetentionSnapshot {
        let state = self.lock();
        crate::inspection::RuntimeWorldRetentionSnapshot {
            unique_pins: state.unique_slots,
            component_obligations: state.active_obligations,
            in_flight_acquisitions: state.active_reservations,
            reserved_unique_pins: state.reserved_unique_slots,
            reserved_acquisitions: state.reserved_in_flight_reservations,
            observations: self.active_observation_count(),
            active_publication_attempts,
        }
    }
    pub(crate) fn inspect_key(
        &self,
        key: &RuntimeWorldRetentionKey,
    ) -> Result<Option<RuntimeWorldRetentionEntry>, RuntimeWorldRetentionInspectionDenial> {
        let state = self.lock();
        if key.owner != state.owner_identity {
            return Err(RuntimeWorldRetentionInspectionDenial::ForeignOwner);
        }
        Ok(state
            .entries
            .get(&key.key)
            .map(|entry| RuntimeWorldRetentionEntry {
                dependencies: entry.counts,
                owner_lease_present: entry.owner_lease.is_some(),
                flight_present: state.flights.contains_key(&key.key),
            }))
    }
    /// Only the named prefix is examined. Hash-table capacity and unrelated
    /// pinned entries do not participate in this explicit maintenance request.
    pub(crate) fn reclaim_keys(
        &self,
        keys: &[RuntimeWorldRetentionKey],
        maximum: usize,
    ) -> Result<RetentionReclamationReport, RuntimeWorldRetentionInspectionDenial> {
        let keys = &keys[..keys.len().min(maximum)];
        let mut state = self.lock();
        if keys.iter().any(|key| key.owner != state.owner_identity) {
            return Err(RuntimeWorldRetentionInspectionDenial::ForeignOwner);
        }
        let mut reclaimed = 0;
        for key in keys {
            let eligible = state.entries.get(&key.key).is_some_and(|entry| {
                entry.owner_lease.is_none()
                    && entry.counts.is_zero()
                    && !state.flights.contains_key(&key.key)
            });
            state.costs.reclamation_entries_examined =
                state.costs.reclamation_entries_examined.saturating_add(1);
            if eligible {
                state.entries.remove(&key.key);
                state.unique_slots -= 1;
                reclaimed += 1;
                state.costs.reclamation_entries_reclaimed =
                    state.costs.reclamation_entries_reclaimed.saturating_add(1);
            }
        }
        Ok(RetentionReclamationReport {
            requested: maximum,
            examined: keys.len(),
            reclaimed,
            remaining_unique_pins: state.unique_slots,
        })
    }
}
