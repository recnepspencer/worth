use crate::entry::{
    PhysicalRecoveryIntegrityObservations, PhysicalRecoveryWalIntegrityObservation,
};
use crate::orchestration::AdmittedWalInventory;

pub(crate) struct RecoveryIntegrityEvidence {
    admitted_wal: AdmittedWalInventory,
    observations: PhysicalRecoveryIntegrityObservations,
}

impl RecoveryIntegrityEvidence {
    pub(crate) const fn new(
        admitted_wal: AdmittedWalInventory,
        wal_observations: Vec<PhysicalRecoveryWalIntegrityObservation>,
    ) -> Self {
        Self {
            admitted_wal,
            observations: PhysicalRecoveryIntegrityObservations::new(wal_observations),
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
