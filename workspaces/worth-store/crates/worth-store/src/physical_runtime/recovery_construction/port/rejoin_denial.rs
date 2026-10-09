//! Preserve the resource boundary's typed cause across the Store handoff.

use super::super::selected_rejoin::SelectedMediaRejoinDenial;
use super::super::PhysicalRecoverySelectedRejoinMismatch as Mismatch;
use super::RecoveredPhysicalRuntimeConstructionDenial;

pub(super) fn construction_denial(
    denial: SelectedMediaRejoinDenial,
) -> RecoveredPhysicalRuntimeConstructionDenial {
    match denial {
        SelectedMediaRejoinDenial::WalReadOwnership(cause) => {
            RecoveredPhysicalRuntimeConstructionDenial::ResidentAdmission(cause)
        }
        SelectedMediaRejoinDenial::WalRead { boundary, cause } => {
            RecoveredPhysicalRuntimeConstructionDenial::RejoinWalRead { boundary, cause }
        }
        SelectedMediaRejoinDenial::WalBytes { boundary, cause } => {
            RecoveredPhysicalRuntimeConstructionDenial::RejoinWalBytes { boundary, cause }
        }
        SelectedMediaRejoinDenial::WalAdmission { boundary, cause } => {
            RecoveredPhysicalRuntimeConstructionDenial::RejoinWalAdmission { boundary, cause }
        }
        SelectedMediaRejoinDenial::BindingSampling { boundary, cause } => {
            RecoveredPhysicalRuntimeConstructionDenial::RejoinBindingSampling { boundary, cause }
        }
        SelectedMediaRejoinDenial::Resident(cause) => {
            RecoveredPhysicalRuntimeConstructionDenial::RejoinResident(cause)
        }
        SelectedMediaRejoinDenial::ResidentBoundary { boundary, cause } => {
            RecoveredPhysicalRuntimeConstructionDenial::RejoinResidentBoundary { boundary, cause }
        }
        SelectedMediaRejoinDenial::WalResident {
            boundary,
            stage,
            artifact_count,
            artifact_ordinal,
            frame_offset,
            cause,
        } => RecoveredPhysicalRuntimeConstructionDenial::RejoinWalResident {
            boundary,
            stage,
            artifact_count,
            artifact_ordinal,
            frame_offset,
            cause,
        },
        SelectedMediaRejoinDenial::ResidentRead {
            artifact,
            offset,
            requested,
            cause,
        } => RecoveredPhysicalRuntimeConstructionDenial::RejoinResidentRead {
            artifact,
            offset,
            requested,
            cause,
        },
        SelectedMediaRejoinDenial::ReadBufferLengthMismatch {
            artifact,
            offset,
            requested,
            observed,
        } => RecoveredPhysicalRuntimeConstructionDenial::RejoinReadBufferLengthMismatch {
            artifact,
            offset,
            requested,
            observed,
        },
        SelectedMediaRejoinDenial::RecordReadAllocation {
            artifact,
            offset,
            requested,
            cause,
        } => RecoveredPhysicalRuntimeConstructionDenial::RejoinRecordReadAllocation {
            artifact,
            offset,
            requested,
            cause,
        },
        SelectedMediaRejoinDenial::Qualification(cause) => {
            RecoveredPhysicalRuntimeConstructionDenial::RejoinQualification(cause)
        }
        SelectedMediaRejoinDenial::Discovery(cause) => {
            RecoveredPhysicalRuntimeConstructionDenial::RejoinDiscovery(cause)
        }
        SelectedMediaRejoinDenial::CanonicalRedo(cause) => {
            RecoveredPhysicalRuntimeConstructionDenial::RejoinCanonicalRedo(cause)
        }
        SelectedMediaRejoinDenial::RecoveryProjection(cause) => {
            RecoveredPhysicalRuntimeConstructionDenial::RejoinRecoveryProjection(cause)
        }
        SelectedMediaRejoinDenial::MissingSelector => semantic_mismatch(Mismatch::MissingSelector),
        SelectedMediaRejoinDenial::MissingRoot => semantic_mismatch(Mismatch::MissingRoot),
        SelectedMediaRejoinDenial::MissingCheckpoint => {
            semantic_mismatch(Mismatch::MissingCheckpoint)
        }
        SelectedMediaRejoinDenial::MissingRoute => semantic_mismatch(Mismatch::MissingRoute),
        SelectedMediaRejoinDenial::MissingFrame => semantic_mismatch(Mismatch::MissingFrame),
        SelectedMediaRejoinDenial::RootBinding => semantic_mismatch(Mismatch::RootBinding),
        SelectedMediaRejoinDenial::CheckpointBinding => {
            semantic_mismatch(Mismatch::CheckpointBinding)
        }
        SelectedMediaRejoinDenial::CertificateRoster => {
            semantic_mismatch(Mismatch::CertificateRoster)
        }
        SelectedMediaRejoinDenial::UnsupportedSelectedControl {
            record,
            content_class,
        } => RecoveredPhysicalRuntimeConstructionDenial::RejoinUnsupportedSelectedControl {
            record,
            content_class,
        },
        SelectedMediaRejoinDenial::RoutingFrame => semantic_mismatch(Mismatch::RoutingFrame),
        SelectedMediaRejoinDenial::UnsupportedSelectedPlacement => {
            semantic_mismatch(Mismatch::UnsupportedSelectedPlacement)
        }
        SelectedMediaRejoinDenial::ControlFrame => semantic_mismatch(Mismatch::ControlFrame),
        SelectedMediaRejoinDenial::BoundExceeded => semantic_mismatch(Mismatch::BoundExceeded),
        SelectedMediaRejoinDenial::WalFate => semantic_mismatch(Mismatch::WalFate),
    }
}

fn semantic_mismatch(cause: Mismatch) -> RecoveredPhysicalRuntimeConstructionDenial {
    RecoveredPhysicalRuntimeConstructionDenial::RejoinSelectedMedia(cause)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_runtime::{
        PhysicalRecoveryRejoinResidentBoundary as Boundary,
        PhysicalRecoveryRejoinResidentDenial as Cause,
    };

    #[test]
    fn resident_boundary_preserves_the_owner_cause_through_public_handoff() {
        let cause = Cause::BudgetExceeded {
            required: 101,
            admitted: 100,
        };
        let denial = SelectedMediaRejoinDenial::Resident(cause.clone())
            .at_resident_boundary(Boundary::FinalWalAdmission);
        assert_eq!(
            construction_denial(denial),
            RecoveredPhysicalRuntimeConstructionDenial::RejoinResidentBoundary {
                boundary: Boundary::FinalWalAdmission,
                cause,
            }
        );
    }

    #[test]
    fn custody_mismatch_is_not_relabelled_as_resident_exhaustion() {
        let denial = SelectedMediaRejoinDenial::CertificateRoster
            .at_resident_boundary(Boundary::FinalSelectedMediaObservation);
        assert_eq!(
            construction_denial(denial),
            RecoveredPhysicalRuntimeConstructionDenial::RejoinSelectedMedia(
                Mismatch::CertificateRoster
            )
        );
    }

    #[test]
    fn sampling_denial_preserves_native_requirement_and_rejoin_pass() {
        let cause = crate::physical_runtime::StoreRecoveryBindingSampleAllocationDenial::Backing {
            requested: 23,
            cause: Cause::BudgetExceeded {
                required: 101,
                admitted: 100,
            },
        };
        let denial = SelectedMediaRejoinDenial::BindingSampling {
            boundary: None,
            cause: cause.clone(),
        }
        .at_resident_boundary(Boundary::FinalSelectedMediaObservation);
        assert_eq!(
            construction_denial(denial),
            RecoveredPhysicalRuntimeConstructionDenial::RejoinBindingSampling {
                boundary: Some(Boundary::FinalSelectedMediaObservation),
                cause,
            }
        );
    }
}
