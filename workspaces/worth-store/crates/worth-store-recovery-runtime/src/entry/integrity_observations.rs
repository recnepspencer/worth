use crate::orchestration::NativeWalRoster;
use std::sync::Arc;
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalIntegrityRejection};

mod assembly;
pub(crate) use assembly::WalIntegrityObservationBuilder;

#[derive(Debug, Clone, Default)]
pub struct PhysicalRecoveryIntegrityObservations {
    storage: ObservationStorage,
}

#[derive(Debug, Clone, Default)]
enum ObservationStorage {
    #[default]
    Empty,
    Shared(Arc<ObservationData>),
}

#[derive(Debug)]
struct ObservationData {
    roster: NativeWalRoster<PhysicalRecoveryWalIntegrityObservation>,
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
    pub(crate) const fn empty() -> Self {
        Self {
            storage: ObservationStorage::Empty,
        }
    }

    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        match &self.storage {
            ObservationStorage::Empty => Some(0),
            ObservationStorage::Shared(data) => data.roster.owned_heap_bytes(),
        }
    }

    pub fn charged_bytes(&self) -> u64 {
        match &self.storage {
            ObservationStorage::Empty => 0,
            ObservationStorage::Shared(data) => data.roster.charged_bytes(),
        }
    }

    pub fn wal(&self) -> &[PhysicalRecoveryWalIntegrityObservation] {
        match &self.storage {
            ObservationStorage::Empty => &[],
            ObservationStorage::Shared(data) => data.roster.as_slice(),
        }
    }
}

impl PartialEq for PhysicalRecoveryIntegrityObservations {
    fn eq(&self, other: &Self) -> bool {
        self.wal() == other.wal()
    }
}

impl Eq for PhysicalRecoveryIntegrityObservations {}

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

#[cfg(all(test, feature = "certification-test-authority"))]
mod tests;
