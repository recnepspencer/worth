use super::{RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressObservation};
use worth_store_physical_integrity::PhysicalArtifactScope;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RecoveryIntegrityIngressTrace {
    counters: RecoveryIntegrityIngressCounters,
    observations: Vec<RecoveryIntegrityIngressObservation>,
}

impl RecoveryIntegrityIngressTrace {
    pub(crate) fn next_observation_capacity(&self, additional: usize) -> Option<usize> {
        let needed = self.observations.len().checked_add(additional)?;
        if needed <= self.observations.capacity() {
            Some(self.observations.capacity())
        } else {
            Some(
                needed
                    .max(self.observations.capacity().checked_mul(2)?)
                    .max(4),
            )
        }
    }

    pub(crate) fn observation_reservation_bytes(&self, additional: usize) -> Option<u64> {
        let capacity = self.next_observation_capacity(additional)?;
        if capacity == self.observations.capacity() {
            return Some(0);
        }
        u64::try_from(capacity)
            .ok()?
            .checked_mul(std::mem::size_of::<RecoveryIntegrityIngressObservation>() as u64)
    }

    pub(crate) fn reserve_observations(&mut self, additional: usize) -> Option<u64> {
        let old = self.owned_heap_bytes()?;
        let capacity = self.next_observation_capacity(additional)?;
        if capacity > self.observations.capacity() {
            self.observations
                .try_reserve_exact(capacity.checked_sub(self.observations.len())?)
                .ok()?;
        }
        self.owned_heap_bytes()?.checked_sub(old)
    }

    pub(crate) fn try_reserve_observation_capacity(
        &mut self,
        capacity: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        if capacity > self.observations.capacity() {
            self.observations
                .try_reserve_exact(capacity - self.observations.len())?;
        }
        Ok(())
    }

    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.observations.capacity())
            .ok()?
            .checked_mul(std::mem::size_of::<RecoveryIntegrityIngressObservation>() as u64)
    }

    pub(crate) const fn new() -> Self {
        Self {
            counters: RecoveryIntegrityIngressCounters::new(),
            observations: Vec::new(),
        }
    }

    pub(crate) fn counters_mut(&mut self) -> &mut RecoveryIntegrityIngressCounters {
        &mut self.counters
    }

    pub(crate) fn retain(&mut self, observation: RecoveryIntegrityIngressObservation) {
        self.observations.push(observation);
    }

    pub(crate) fn record(&mut self, observation: RecoveryIntegrityIngressObservation) {
        self.counters.record(observation);
        self.retain(observation);
    }

    pub(crate) fn reject(
        &mut self,
        scope: PhysicalArtifactScope,
        rejection: super::RecoveryIntegrityIngressRejection,
    ) -> super::RecoveryIntegrityIngressRejection {
        let outcome: Result<(), _> = Err(rejection);
        let observation = super::counters::record_admission(scope, &outcome, &mut self.counters);
        self.retain(observation);
        rejection
    }

    pub(crate) fn append(&mut self, mut other: Self) {
        self.counters.attempted += other.counters.attempted;
        self.counters.admitted += other.counters.admitted;
        self.counters.rejected_damaged += other.counters.rejected_damaged;
        self.counters.rejected_unsupported += other.counters.rejected_unsupported;
        self.counters.rejected_unknown += other.counters.rejected_unknown;
        self.counters.rejected_indeterminate += other.counters.rejected_indeterminate;
        self.counters.rejected_absent += other.counters.rejected_absent;
        self.counters.rejected_conflicting += other.counters.rejected_conflicting;
        self.counters.rejected_source_binding += other.counters.rejected_source_binding;
        self.counters.owner_projection_entries += other.counters.owner_projection_entries;
        self.counters.owner_decoder_entries += other.counters.owner_decoder_entries;
        self.observations.append(&mut other.observations);
    }

    pub(crate) fn observations(&self) -> &[RecoveryIntegrityIngressObservation] {
        &self.observations
    }

    pub(crate) const fn counters(&self) -> RecoveryIntegrityIngressCounters {
        self.counters
    }
}
