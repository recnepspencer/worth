use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalIntegrityRejection};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhysicalRecoveryIntegrityObservations {
    wal: Vec<PhysicalRecoveryWalIntegrityObservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecoveryWalIntegrityObservation {
    scope: PhysicalArtifactScope,
    outcome: PhysicalRecoveryWalIntegrityObservationOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryWalIntegrityObservationOutcome {
    Admitted,
    Rejected(PhysicalIntegrityRejection),
}

impl PhysicalRecoveryIntegrityObservations {
    pub(crate) const fn new(wal: Vec<PhysicalRecoveryWalIntegrityObservation>) -> Self {
        Self { wal }
    }

    pub fn wal(&self) -> &[PhysicalRecoveryWalIntegrityObservation] {
        &self.wal
    }
}

impl PhysicalRecoveryWalIntegrityObservation {
    pub(crate) const fn new(
        scope: PhysicalArtifactScope,
        outcome: PhysicalRecoveryWalIntegrityObservationOutcome,
    ) -> Self {
        Self { scope, outcome }
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn outcome(self) -> PhysicalRecoveryWalIntegrityObservationOutcome {
        self.outcome
    }
}
