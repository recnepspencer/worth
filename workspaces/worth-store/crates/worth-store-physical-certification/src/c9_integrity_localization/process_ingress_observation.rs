use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProcessIngressCounters {
    pub(crate) attempted: u64,
    pub(crate) admitted: u64,
    pub(crate) rejected_damaged: u64,
    pub(crate) rejected_unsupported: u64,
    pub(crate) rejected_unknown: u64,
    pub(crate) rejected_indeterminate: u64,
    pub(crate) rejected_absent: u64,
    pub(crate) rejected_conflicting: u64,
    pub(crate) rejected_source_binding: u64,
    pub(crate) owner_projection_entries: u64,
    pub(crate) owner_decoder_entries: u64,
}

pub(super) fn project_counters(
    value: worth_store_recovery_runtime::PhysicalRecoveryIntegrityCounters,
) -> ProcessIngressCounters {
    ProcessIngressCounters {
        attempted: value.attempted,
        admitted: value.admitted,
        rejected_damaged: value.rejected_damaged,
        rejected_unsupported: value.rejected_unsupported,
        rejected_unknown: value.rejected_unknown,
        rejected_indeterminate: value.rejected_indeterminate,
        rejected_absent: value.rejected_absent,
        rejected_conflicting: value.rejected_conflicting,
        rejected_source_binding: value.rejected_source_binding,
        owner_projection_entries: value.owner_projection_entries,
        owner_decoder_entries: value.owner_decoder_entries,
    }
}
use worth_store_recovery_runtime::{
    PhysicalRecoveryIntegrityObservation, PhysicalRecoveryIntegrityObservationOutcome,
    PhysicalRecoveryIntegrityRejection, PhysicalRecoveryWalIntegrityObservation,
    PhysicalRecoveryWalIntegrityObservationOutcome,
};

use super::process_integrity_projection::{project_integrity_rejection, project_integrity_scope};
use super::process_recovery_observation::{ProcessIntegrityRejection, ProcessIntegrityScope};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProcessIngressObservation {
    pub(crate) scope: ProcessIntegrityScope,
    pub(crate) outcome: ProcessIngressOutcome,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessIngressOutcome {
    Admitted,
    Absent,
    ConflictingDuplication { observed_sources: u64 },
    Integrity(ProcessIntegrityRejection),
    NonCanonicalEncoding,
    MissingBoundedArtifact,
    SourceRangeOutsideObservation,
    ScopeMismatch,
    SourceIncarnationMismatch,
}

pub(super) fn project_ingress(
    observations: &[PhysicalRecoveryIntegrityObservation],
) -> Vec<ProcessIngressObservation> {
    observations
        .iter()
        .map(|observation| {
            let outcome = match observation.outcome() {
                PhysicalRecoveryIntegrityObservationOutcome::Admitted => {
                    ProcessIngressOutcome::Admitted
                }
                PhysicalRecoveryIntegrityObservationOutcome::Rejected(rejection) => match rejection
                {
                    PhysicalRecoveryIntegrityRejection::Absent => ProcessIngressOutcome::Absent,
                    PhysicalRecoveryIntegrityRejection::ConflictingDuplication {
                        observed_sources,
                    } => ProcessIngressOutcome::ConflictingDuplication { observed_sources },
                    PhysicalRecoveryIntegrityRejection::Integrity(rejection) => {
                        ProcessIngressOutcome::Integrity(project_integrity_rejection(rejection))
                    }
                    PhysicalRecoveryIntegrityRejection::NonCanonicalEncoding => {
                        ProcessIngressOutcome::NonCanonicalEncoding
                    }
                    PhysicalRecoveryIntegrityRejection::MissingBoundedArtifact => {
                        ProcessIngressOutcome::MissingBoundedArtifact
                    }
                    PhysicalRecoveryIntegrityRejection::SourceRangeOutsideObservation => {
                        ProcessIngressOutcome::SourceRangeOutsideObservation
                    }
                    PhysicalRecoveryIntegrityRejection::ScopeMismatch => {
                        ProcessIngressOutcome::ScopeMismatch
                    }
                    PhysicalRecoveryIntegrityRejection::SourceIncarnationMismatch => {
                        ProcessIngressOutcome::SourceIncarnationMismatch
                    }
                },
            };
            ProcessIngressObservation {
                scope: project_integrity_scope(observation.scope()),
                outcome,
            }
        })
        .collect()
}

pub(super) fn project_wal(
    observations: &[PhysicalRecoveryWalIntegrityObservation],
) -> Vec<ProcessIngressObservation> {
    observations
        .iter()
        .map(|observation| ProcessIngressObservation {
            scope: project_integrity_scope(observation.scope()),
            outcome: match observation.outcome() {
                PhysicalRecoveryWalIntegrityObservationOutcome::Admitted => {
                    ProcessIngressOutcome::Admitted
                }
                PhysicalRecoveryWalIntegrityObservationOutcome::Rejected(rejection) => {
                    ProcessIngressOutcome::Integrity(project_integrity_rejection(rejection))
                }
            },
        })
        .collect()
}
