use worth_store_physical_backend::ArtifactTreeFailureKind;

use super::super::super::{RecordReadDenial, RecordReadWorkDenial};
use super::super::{
    super::residency::frame_loading::{
        FrameLoadFailure, FrameLoadFailureKind, FrameLoadFaultCause,
    },
    manifest_routing::ManifestLookupFailure,
};

pub(in crate::physical_runtime::record_serving) fn read_failure(
    failure: FrameLoadFailure,
) -> RecordReadDenial {
    match failure.kind() {
        FrameLoadFailureKind::Backend(failure) => backend_denial(failure),
        FrameLoadFailureKind::Residency(reason) => RecordReadDenial::from_residency(reason),
        FrameLoadFailureKind::Work(failure) => work_denial(failure),
        FrameLoadFailureKind::FaultTerminated { cause, .. } => fault_denial(cause),
        FrameLoadFailureKind::CoalescedFault(terminal) => RecordReadDenial::from_residency(
            worth_store_buffer_pool::PhysicalResidencyDenial::FrameLoadTerminated(terminal),
        ),
        _ => RecordReadDenial::ArtifactDamaged,
    }
}

fn fault_denial(cause: FrameLoadFaultCause) -> RecordReadDenial {
    match cause {
        FrameLoadFaultCause::Backend(failure) => backend_denial(failure),
        FrameLoadFaultCause::Work(failure) => work_denial(failure),
        FrameLoadFaultCause::Residency(reason) => RecordReadDenial::from_residency(reason),
    }
}

fn work_denial(
    failure: crate::physical_runtime::record_serving::CanonicalRecordReadFailure,
) -> RecordReadDenial {
    match failure {
        crate::physical_runtime::record_serving::CanonicalRecordReadFailure::Backend(failure) => {
            backend_denial(failure)
        }
        crate::physical_runtime::record_serving::CanonicalRecordReadFailure::Terminal(cause) => {
            terminal_denial(cause)
        }
        _ => RecordReadDenial::PhysicalWork(
            failure
                .work_denial()
                .expect("non-backend canonical failures have work denials"),
        ),
    }
}

fn terminal_denial(cause: crate::physical_runtime::PhysicalWorkTerminalCause) -> RecordReadDenial {
    match cause {
        crate::physical_runtime::PhysicalWorkTerminalCause::Backend(failure) => {
            backend_denial(failure)
        }
        crate::physical_runtime::PhysicalWorkTerminalCause::IncompleteRead { .. } => {
            RecordReadDenial::ArtifactDamaged
        }
        crate::physical_runtime::PhysicalWorkTerminalCause::SchedulerRejectedAfterEffect => {
            RecordReadDenial::PhysicalWork(RecordReadWorkDenial::SchedulerSettlementRejected)
        }
    }
}

fn backend_denial(failure: worth_store_physical_backend::ArtifactTreeFailure) -> RecordReadDenial {
    match failure.kind() {
        ArtifactTreeFailureKind::Absent => RecordReadDenial::ArtifactUnavailable,
        ArtifactTreeFailureKind::AccessLimitExceeded => RecordReadDenial::AccessLimitExceeded,
        ArtifactTreeFailureKind::DeniedBeforeEffect => {
            RecordReadDenial::BackendUnavailable(failure)
        }
        _ => RecordReadDenial::ArtifactDamaged,
    }
}

pub(in crate::physical_runtime::record_serving) fn manifest_failure(
    failure: ManifestLookupFailure,
) -> RecordReadDenial {
    match failure {
        ManifestLookupFailure::Backend(failure) => read_failure(FrameLoadFailure::new(
            FrameLoadFailureKind::Backend(failure),
        )),
        ManifestLookupFailure::Frame(kind) => read_failure(FrameLoadFailure::new(kind)),
        ManifestLookupFailure::ResidentAdmission(denial) => resident_admission_failure(denial),
        ManifestLookupFailure::Damaged => RecordReadDenial::ArtifactDamaged,
        ManifestLookupFailure::Residency(reason) => RecordReadDenial::from_residency(reason),
    }
}

fn resident_admission_failure(
    denial: crate::physical_runtime::integrity::resident_admission::denial::ResidentIntegrityAdmissionDenial,
) -> RecordReadDenial {
    use crate::physical_runtime::integrity::resident_admission::denial::ResidentIntegrityAdmissionDenial;
    use worth_store_physical_integrity::PhysicalIntegrityRejection;

    if let Some(residency) = denial.residency_unavailability() {
        return RecordReadDenial::from_residency(residency);
    }
    match denial {
        ResidentIntegrityAdmissionDenial::LifecycleGenerationChanged => {
            RecordReadDenial::PhysicalWork(RecordReadWorkDenial::RuntimeReleased)
        }
        ResidentIntegrityAdmissionDenial::Validation(PhysicalIntegrityRejection::Damaged(_)) => {
            RecordReadDenial::ArtifactDamaged
        }
        ResidentIntegrityAdmissionDenial::Validation(PhysicalIntegrityRejection::Unsupported(
            _,
        ))
        | ResidentIntegrityAdmissionDenial::BootstrapUnsupportedFormat(_)
        | ResidentIntegrityAdmissionDenial::BootstrapScopeMismatch(_) => {
            RecordReadDenial::FormatMismatch
        }
        ResidentIntegrityAdmissionDenial::Validation(
            PhysicalIntegrityRejection::Unknown(_) | PhysicalIntegrityRejection::Indeterminate(_),
        )
        | ResidentIntegrityAdmissionDenial::SourceScopeMismatch => {
            RecordReadDenial::ArtifactUnavailable
        }
        ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch
        | ResidentIntegrityAdmissionDenial::FrameGenerationChanged
        | ResidentIntegrityAdmissionDenial::RetainedRecordInvalidated
        | ResidentIntegrityAdmissionDenial::RetainedRecordChanged
        | ResidentIntegrityAdmissionDenial::Frame(_) => RecordReadDenial::ArtifactUnavailable,
    }
}

#[cfg(test)]
pub(in crate::physical_runtime) fn assert_actual_lifecycle_manifest_denial_maps_without_damage(
    denial: crate::physical_runtime::integrity::resident_admission::denial::ResidentIntegrityAdmissionDenial,
) {
    use super::super::manifest_routing::ManifestLookupFailure;
    use crate::physical_runtime::record_serving::{
        RecordAppendDenial, RecordAppendError, RecordReadWorkDenial,
    };

    assert_eq!(
        manifest_failure(ManifestLookupFailure::ResidentAdmission(denial)),
        RecordReadDenial::PhysicalWork(RecordReadWorkDenial::RuntimeReleased)
    );
    assert!(matches!(
        crate::physical_runtime::record_serving::planning::inline_plan_failure::manifest_lookup_failure(
            ManifestLookupFailure::ResidentAdmission(denial)
        ),
        RecordAppendError::Denied(RecordAppendDenial::PhysicalReadWorkUnavailable(
            RecordReadWorkDenial::RuntimeReleased
        ))
    ));
}
