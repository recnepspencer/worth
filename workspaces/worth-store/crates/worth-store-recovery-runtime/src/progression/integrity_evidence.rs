use crate::entry::PhysicalRecoveryIntegrityObservations;
use crate::orchestration::AdmittedWalInventory;

pub(crate) struct RecoveryIntegrityEvidence {
    admitted_wal: AdmittedWalInventory,
    observations: PhysicalRecoveryIntegrityObservations,
}

impl RecoveryIntegrityEvidence {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        self.admitted_wal
            .owned_heap_bytes()?
            .checked_add(self.observations.owned_heap_bytes()?)
    }

    pub(crate) const fn new(
        admitted_wal: AdmittedWalInventory,
        observations: PhysicalRecoveryIntegrityObservations,
    ) -> Self {
        Self {
            admitted_wal,
            observations,
        }
    }

    pub(crate) const fn admitted_wal(&self) -> &AdmittedWalInventory {
        &self.admitted_wal
    }

    pub(crate) const fn observations(&self) -> &PhysicalRecoveryIntegrityObservations {
        &self.observations
    }

    pub(crate) fn into_observations(self) -> PhysicalRecoveryIntegrityObservations {
        self.observations
    }
}
