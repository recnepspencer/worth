use worth_store_physical_integrity::{
    IndeterminatePhysicalIntegrityCause, PhysicalArtifactScope, PhysicalBlastRadius,
    PhysicalByteRange, PhysicalDamageLocalization, PhysicalIntegrityRejection,
    PhysicalIntegrityVersionAxis, UnknownPhysicalIntegrityCause,
};

use super::process_integrity_vocabulary::{
    project_artifact_family, project_damage_cause, project_format_field,
};
use super::process_recovery_observation::{
    ProcessBlastRadius, ProcessByteRange, ProcessDamageLocalization,
    ProcessIndeterminateIntegrityCause, ProcessIntegrityRejection, ProcessIntegrityScope,
    ProcessIntegrityVersionAxis, ProcessUnknownIntegrityCause,
};

pub(super) fn project_integrity_rejection(
    rejection: PhysicalIntegrityRejection,
) -> ProcessIntegrityRejection {
    match rejection {
        PhysicalIntegrityRejection::Damaged(localization) => {
            ProcessIntegrityRejection::Damaged(project_damage_localization(localization))
        }
        PhysicalIntegrityRejection::Unsupported(posture) => {
            ProcessIntegrityRejection::Unsupported {
                scope: project_integrity_scope(posture.scope()),
                axis: match posture.axis() {
                    PhysicalIntegrityVersionAxis::EnvelopeSchema => {
                        ProcessIntegrityVersionAxis::EnvelopeSchema
                    }
                    PhysicalIntegrityVersionAxis::PhysicalFormat => {
                        ProcessIntegrityVersionAxis::PhysicalFormat
                    }
                    PhysicalIntegrityVersionAxis::PhysicalWorkObligation => {
                        ProcessIntegrityVersionAxis::PhysicalWorkObligation
                    }
                    PhysicalIntegrityVersionAxis::WalFrame => ProcessIntegrityVersionAxis::WalFrame,
                    PhysicalIntegrityVersionAxis::CheckpointRecordSchema => {
                        ProcessIntegrityVersionAxis::CheckpointRecordSchema
                    }
                },
                observed: posture.observed(),
            }
        }
        PhysicalIntegrityRejection::Unknown(posture) => ProcessIntegrityRejection::Unknown {
            scope: project_integrity_scope(posture.scope()),
            cause: match posture.cause() {
                UnknownPhysicalIntegrityCause::ExpectedArtifactAbsent => {
                    ProcessUnknownIntegrityCause::ExpectedArtifactAbsent
                }
                UnknownPhysicalIntegrityCause::UnrecognizedArtifact => {
                    ProcessUnknownIntegrityCause::UnrecognizedArtifact
                }
                UnknownPhysicalIntegrityCause::ExpectedScopeUnavailable => {
                    ProcessUnknownIntegrityCause::ExpectedScopeUnavailable
                }
            },
        },
        PhysicalIntegrityRejection::Indeterminate(posture) => {
            ProcessIntegrityRejection::Indeterminate {
                scope: project_integrity_scope(posture.scope()),
                cause: match posture.cause() {
                    IndeterminatePhysicalIntegrityCause::SourceChangedDuringInspection => {
                        ProcessIndeterminateIntegrityCause::SourceChangedDuringInspection
                    }
                    IndeterminatePhysicalIntegrityCause::ObservationBoundExhausted => {
                        ProcessIndeterminateIntegrityCause::ObservationBoundExhausted
                    }
                    IndeterminatePhysicalIntegrityCause::StableRangeNotProven => {
                        ProcessIndeterminateIntegrityCause::StableRangeNotProven
                    }
                },
                observed_range: posture.observed_range().map(project_byte_range),
            }
        }
    }
}

fn project_damage_localization(
    localization: PhysicalDamageLocalization,
) -> ProcessDamageLocalization {
    ProcessDamageLocalization {
        scope: project_integrity_scope(localization.scope()),
        cause: project_damage_cause(localization.cause()),
        damaged_range: project_byte_range(localization.damaged_range()),
        field: localization.field().map(project_format_field),
        blast_radius: match localization.blast_radius() {
            PhysicalBlastRadius::DamagedRange => ProcessBlastRadius::DamagedRange,
            PhysicalBlastRadius::CanonicalFrame => ProcessBlastRadius::CanonicalFrame,
            PhysicalBlastRadius::CompleteArtifact => ProcessBlastRadius::CompleteArtifact,
            PhysicalBlastRadius::ReachableSubtree => ProcessBlastRadius::ReachableSubtree,
        },
    }
}

pub(super) fn project_integrity_scope(scope: PhysicalArtifactScope) -> ProcessIntegrityScope {
    ProcessIntegrityScope {
        identity: super::process_scope_identity::project(scope),
        store_identity: scope.store_identity().bytes(),
        family: project_artifact_family(scope.artifact_family()),
        root_generation: scope.root_generation(),
        byte_range: project_byte_range(scope.byte_range()),
        record_format_identity: scope
            .durable_frame_record_format()
            .map(|format| format.canonical_identity_bytes()),
    }
}

fn project_byte_range(range: PhysicalByteRange) -> ProcessByteRange {
    ProcessByteRange {
        offset: range.offset(),
        length: range.length(),
    }
}
