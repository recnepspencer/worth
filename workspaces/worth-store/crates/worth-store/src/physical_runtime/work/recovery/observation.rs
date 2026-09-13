use worth_store_physical_backend::ArtifactTreeFailureKind;
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalIntegrityRejection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkRecoveryIngressRejection {
    InvalidPendingName,
    ReadFailure(ArtifactTreeFailureKind),
    Integrity(PhysicalIntegrityRejection),
    SourceIncarnationMismatch,
    OwnerProjectionRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkRecoveryAdmissionOutcome {
    Admitted(PhysicalArtifactScope),
    Rejected {
        scope: Option<PhysicalArtifactScope>,
        rejection: PhysicalWorkRecoveryIngressRejection,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalWorkRecoveryObservationSubject {
    Inventory,
    PendingFile(Box<str>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalWorkRecoveryAdmissionObservation {
    subject: PhysicalWorkRecoveryObservationSubject,
    outcome: PhysicalWorkRecoveryAdmissionOutcome,
}

impl PhysicalWorkRecoveryAdmissionObservation {
    pub(super) fn admitted(file_name: &str, scope: PhysicalArtifactScope) -> Self {
        Self {
            subject: PhysicalWorkRecoveryObservationSubject::PendingFile(file_name.into()),
            outcome: PhysicalWorkRecoveryAdmissionOutcome::Admitted(scope),
        }
    }

    pub(super) fn rejected(
        file_name: &str,
        scope: Option<PhysicalArtifactScope>,
        rejection: PhysicalWorkRecoveryIngressRejection,
    ) -> Self {
        Self {
            subject: PhysicalWorkRecoveryObservationSubject::PendingFile(file_name.into()),
            outcome: PhysicalWorkRecoveryAdmissionOutcome::Rejected { scope, rejection },
        }
    }

    pub(super) fn inventory_rejected(rejection: PhysicalWorkRecoveryIngressRejection) -> Self {
        Self {
            subject: PhysicalWorkRecoveryObservationSubject::Inventory,
            outcome: PhysicalWorkRecoveryAdmissionOutcome::Rejected {
                scope: None,
                rejection,
            },
        }
    }

    pub const fn subject(&self) -> &PhysicalWorkRecoveryObservationSubject {
        &self.subject
    }

    pub const fn outcome(&self) -> PhysicalWorkRecoveryAdmissionOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalWorkRecoveryAdmissionCounters {
    attempted: u64,
    admitted: u64,
    rejected_before_owner_interpretation: u64,
    owner_interpretation_entries: u64,
}

impl PhysicalWorkRecoveryAdmissionCounters {
    pub(super) fn attempt(&mut self) {
        self.attempted = self.attempted.saturating_add(1);
    }

    pub(super) fn admitted(&mut self) {
        self.admitted = self.admitted.saturating_add(1);
    }

    pub(super) fn rejected_before_owner_interpretation(&mut self) {
        self.rejected_before_owner_interpretation =
            self.rejected_before_owner_interpretation.saturating_add(1);
    }

    pub(super) fn owner_interpretation(&mut self) {
        self.owner_interpretation_entries = self.owner_interpretation_entries.saturating_add(1);
    }

    pub const fn attempted(self) -> u64 {
        self.attempted
    }

    pub const fn admitted_count(self) -> u64 {
        self.admitted
    }

    pub const fn rejected_before_owner_interpretation_count(self) -> u64 {
        self.rejected_before_owner_interpretation
    }

    pub const fn owner_interpretation_entries(self) -> u64 {
        self.owner_interpretation_entries
    }
}
