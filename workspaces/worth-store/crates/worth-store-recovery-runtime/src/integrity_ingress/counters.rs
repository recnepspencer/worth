use worth_store_physical_integrity::PhysicalIntegrityRejection;

use super::{
    RecoveryIntegrityIngressObservation, RecoveryIntegrityIngressObservationOutcome,
    RecoveryIntegrityIngressRejection,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecoveryIntegrityIngressCounters {
    pub attempted: u64,
    pub admitted: u64,
    pub rejected_damaged: u64,
    pub rejected_unsupported: u64,
    pub rejected_unknown: u64,
    pub rejected_indeterminate: u64,
    pub rejected_absent: u64,
    pub rejected_conflicting: u64,
    pub rejected_source_binding: u64,
    pub owner_projection_entries: u64,
    pub owner_decoder_entries: u64,
}

impl RecoveryIntegrityIngressCounters {
    pub(crate) const fn attempted(self) -> u64 {
        self.attempted
    }

    pub(crate) const fn admitted(self) -> u64 {
        self.admitted
    }

    pub(crate) const fn rejected(self) -> u64 {
        self.attempted.saturating_sub(self.admitted)
    }

    pub(crate) const fn owner_projection_entries(self) -> u64 {
        self.owner_projection_entries
    }

    pub(crate) const fn owner_decoder_entries(self) -> u64 {
        self.owner_decoder_entries
    }

    pub(super) const fn new() -> Self {
        Self {
            attempted: 0,
            admitted: 0,
            rejected_damaged: 0,
            rejected_unsupported: 0,
            rejected_unknown: 0,
            rejected_indeterminate: 0,
            rejected_absent: 0,
            rejected_conflicting: 0,
            rejected_source_binding: 0,
            owner_projection_entries: 0,
            owner_decoder_entries: 0,
        }
    }

    pub(crate) fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            attempted: self.attempted.checked_add(other.attempted)?,
            admitted: self.admitted.checked_add(other.admitted)?,
            rejected_damaged: self.rejected_damaged.checked_add(other.rejected_damaged)?,
            rejected_unsupported: self
                .rejected_unsupported
                .checked_add(other.rejected_unsupported)?,
            rejected_unknown: self.rejected_unknown.checked_add(other.rejected_unknown)?,
            rejected_indeterminate: self
                .rejected_indeterminate
                .checked_add(other.rejected_indeterminate)?,
            rejected_absent: self.rejected_absent.checked_add(other.rejected_absent)?,
            rejected_conflicting: self
                .rejected_conflicting
                .checked_add(other.rejected_conflicting)?,
            rejected_source_binding: self
                .rejected_source_binding
                .checked_add(other.rejected_source_binding)?,
            owner_projection_entries: self
                .owner_projection_entries
                .checked_add(other.owner_projection_entries)?,
            owner_decoder_entries: self
                .owner_decoder_entries
                .checked_add(other.owner_decoder_entries)?,
        })
    }

    pub(super) fn record(&mut self, observation: RecoveryIntegrityIngressObservation) {
        self.attempted += 1;
        match observation.outcome() {
            RecoveryIntegrityIngressObservationOutcome::Admitted => self.admitted += 1,
            RecoveryIntegrityIngressObservationOutcome::Rejected(rejection) => {
                self.record_rejection(rejection)
            }
        }
    }

    pub(super) fn record_owner_projection(&mut self) {
        self.owner_projection_entries += 1;
    }

    pub(super) fn record_owner_decoder(&mut self) {
        self.owner_decoder_entries += 1;
    }

    fn record_rejection(&mut self, rejection: RecoveryIntegrityIngressRejection) {
        match rejection {
            RecoveryIntegrityIngressRejection::Integrity(PhysicalIntegrityRejection::Damaged(
                _,
            )) => self.rejected_damaged += 1,
            RecoveryIntegrityIngressRejection::Integrity(
                PhysicalIntegrityRejection::Unsupported(_),
            ) => self.rejected_unsupported += 1,
            RecoveryIntegrityIngressRejection::Integrity(PhysicalIntegrityRejection::Unknown(
                _,
            )) => self.rejected_unknown += 1,
            RecoveryIntegrityIngressRejection::Integrity(
                PhysicalIntegrityRejection::Indeterminate(_),
            ) => self.rejected_indeterminate += 1,
            RecoveryIntegrityIngressRejection::Absent
            | RecoveryIntegrityIngressRejection::MissingBoundedArtifact => {
                self.rejected_absent += 1
            }
            RecoveryIntegrityIngressRejection::ConflictingDuplication { .. } => {
                self.rejected_conflicting += 1
            }
            RecoveryIntegrityIngressRejection::NonCanonicalEncoding
            | RecoveryIntegrityIngressRejection::SourceRangeOutsideObservation
            | RecoveryIntegrityIngressRejection::ScopeMismatch
            | RecoveryIntegrityIngressRejection::SourceIncarnationMismatch => {
                self.rejected_source_binding += 1
            }
        }
    }
}

pub(super) fn record_admission<T>(
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    outcome: &Result<T, RecoveryIntegrityIngressRejection>,
    counters: &mut RecoveryIntegrityIngressCounters,
) -> RecoveryIntegrityIngressObservation {
    let observation = match outcome {
        Ok(_) => RecoveryIntegrityIngressObservation::admitted(scope),
        Err(rejection) => RecoveryIntegrityIngressObservation::rejected(scope, *rejection),
    };
    counters.record(observation);
    observation
}
