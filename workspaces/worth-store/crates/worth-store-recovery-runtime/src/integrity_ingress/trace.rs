use super::{RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressObservation};
use worth_store_physical_integrity::PhysicalArtifactScope;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RecoveryIntegrityIngressTrace {
    counters: RecoveryIntegrityIngressCounters,
    observations: Vec<RecoveryIntegrityIngressObservation>,
}

impl RecoveryIntegrityIngressTrace {
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
