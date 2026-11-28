use super::RuntimeWorldRetentionOwner;
use crate::inspection::{
    RuntimeWorldRetentionEntry, RuntimeWorldRetentionInspectionDenial, RuntimeWorldRetentionKey,
};
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
            unique_pins: state.unique_slots(),
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
                live_claims: entry.counts.total(),
            }))
    }
}
