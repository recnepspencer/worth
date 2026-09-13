use worth_store_physical_integrity::PhysicalArtifactScope;

use super::RecoveryIntegrityIngressRejection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryIntegrityIngressObservation {
    scope: PhysicalArtifactScope,
    outcome: RecoveryIntegrityIngressObservationOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryIntegrityIngressObservationOutcome {
    Admitted,
    Rejected(RecoveryIntegrityIngressRejection),
}

impl RecoveryIntegrityIngressObservation {
    pub(super) const fn admitted(scope: PhysicalArtifactScope) -> Self {
        Self {
            scope,
            outcome: RecoveryIntegrityIngressObservationOutcome::Admitted,
        }
    }

    pub(super) const fn rejected(
        scope: PhysicalArtifactScope,
        rejection: RecoveryIntegrityIngressRejection,
    ) -> Self {
        Self {
            scope,
            outcome: RecoveryIntegrityIngressObservationOutcome::Rejected(rejection),
        }
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn outcome(self) -> RecoveryIntegrityIngressObservationOutcome {
        self.outcome
    }
}
