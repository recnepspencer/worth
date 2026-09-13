use worth_store_physical_backend::ArtifactTreeFailureKind;
use worth_store_physical_format::PhysicalGeneration;

use super::super::{RecordAppendDenial, RecordAppendError};

pub(in crate::physical_runtime::record_serving) fn admitted_generation(
    value: Option<u64>,
) -> Result<PhysicalGeneration, RecordAppendError> {
    value
        .and_then(|value| PhysicalGeneration::from_raw(value).ok())
        .ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PhysicalIdentityExhausted,
        ))
}

pub(in crate::physical_runtime::record_serving) fn layout_failure(
    failure: super::super::residency::frame_loading::FrameLoadFailure,
) -> RecordAppendError {
    match failure.kind() {
        super::super::residency::frame_loading::FrameLoadFailureKind::Backend(failure)
            if failure.kind() == ArtifactTreeFailureKind::DeniedBeforeEffect =>
        {
            RecordAppendError::Denied(RecordAppendDenial::BackendUnavailable(failure))
        }
        super::super::residency::frame_loading::FrameLoadFailureKind::Residency(reason) => {
            RecordAppendError::Denied(RecordAppendDenial::from_residency(reason))
        }
        super::super::residency::frame_loading::FrameLoadFailureKind::Work(failure) => {
            canonical_work_failure(failure)
        }
        super::super::residency::frame_loading::FrameLoadFailureKind::FaultTerminated {
            cause,
            ..
        } => fault_cause(cause),
        super::super::residency::frame_loading::FrameLoadFailureKind::CoalescedFault(terminal) => {
            RecordAppendError::Denied(RecordAppendDenial::from_residency(
                worth_store_buffer_pool::PhysicalResidencyDenial::FrameLoadTerminated(terminal),
            ))
        }
        _ => RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged),
    }
}

fn fault_cause(
    cause: super::super::residency::frame_loading::FrameLoadFaultCause,
) -> RecordAppendError {
    match cause {
        super::super::residency::frame_loading::FrameLoadFaultCause::Backend(failure) => {
            layout_failure(
                super::super::residency::frame_loading::FrameLoadFailure::new(
                    super::super::residency::frame_loading::FrameLoadFailureKind::Backend(failure),
                ),
            )
        }
        super::super::residency::frame_loading::FrameLoadFaultCause::Work(failure) => {
            canonical_work_failure(failure)
        }
        super::super::residency::frame_loading::FrameLoadFaultCause::Residency(reason) => {
            RecordAppendError::Denied(RecordAppendDenial::from_residency(reason))
        }
    }
}

fn canonical_work_failure(failure: super::super::CanonicalRecordReadFailure) -> RecordAppendError {
    match failure {
        super::super::CanonicalRecordReadFailure::Backend(failure)
            if failure.kind() == ArtifactTreeFailureKind::DeniedBeforeEffect =>
        {
            RecordAppendError::Denied(RecordAppendDenial::BackendUnavailable(failure))
        }
        super::super::CanonicalRecordReadFailure::Backend(_) => {
            RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
        }
        super::super::CanonicalRecordReadFailure::Terminal(cause) => terminal_failure(cause),
        _ => RecordAppendError::Denied(RecordAppendDenial::PhysicalReadWorkUnavailable(
            failure
                .work_denial()
                .expect("non-backend canonical failures have work denials"),
        )),
    }
}

fn terminal_failure(
    cause: crate::physical_runtime::PhysicalWorkTerminalCause,
) -> RecordAppendError {
    match cause {
        crate::physical_runtime::PhysicalWorkTerminalCause::SchedulerRejectedAfterEffect => {
            RecordAppendError::Denied(RecordAppendDenial::PhysicalReadWorkUnavailable(
                super::super::RecordReadWorkDenial::SchedulerSettlementRejected,
            ))
        }
        crate::physical_runtime::PhysicalWorkTerminalCause::Backend(_)
        | crate::physical_runtime::PhysicalWorkTerminalCause::IncompleteRead { .. } => {
            RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
        }
    }
}

pub(in crate::physical_runtime::record_serving) fn manifest_lookup_failure(
    failure: super::super::access::manifest_routing::ManifestLookupFailure,
) -> RecordAppendError {
    match failure {
        super::super::access::manifest_routing::ManifestLookupFailure::Backend(failure) => {
            layout_failure(
                super::super::residency::frame_loading::FrameLoadFailure::new(
                    super::super::residency::frame_loading::FrameLoadFailureKind::Backend(failure),
                ),
            )
        }
        super::super::access::manifest_routing::ManifestLookupFailure::Frame(kind) => {
            layout_failure(super::super::residency::frame_loading::FrameLoadFailure::new(kind))
        }
        super::super::access::manifest_routing::ManifestLookupFailure::ResidentAdmission(
            denial,
        ) => resident_admission_failure(denial),
        super::super::access::manifest_routing::ManifestLookupFailure::Damaged => {
            RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
        }
        super::super::access::manifest_routing::ManifestLookupFailure::Residency(reason) => {
            RecordAppendError::Denied(RecordAppendDenial::from_residency(reason))
        }
    }
}

fn resident_admission_failure(
    denial: crate::physical_runtime::integrity::resident_admission::denial::ResidentIntegrityAdmissionDenial,
) -> RecordAppendError {
    use crate::physical_runtime::integrity::resident_admission::denial::ResidentIntegrityAdmissionDenial;
    use worth_store_physical_integrity::PhysicalIntegrityRejection;

    if let Some(residency) = denial.residency_unavailability() {
        return RecordAppendError::Denied(RecordAppendDenial::from_residency(residency));
    }
    let denial = match denial {
        ResidentIntegrityAdmissionDenial::LifecycleGenerationChanged => {
            RecordAppendDenial::PhysicalReadWorkUnavailable(
                super::super::RecordReadWorkDenial::RuntimeReleased,
            )
        }
        ResidentIntegrityAdmissionDenial::Validation(PhysicalIntegrityRejection::Damaged(_)) => {
            RecordAppendDenial::PublishedLayoutDamaged
        }
        ResidentIntegrityAdmissionDenial::Validation(PhysicalIntegrityRejection::Unsupported(
            _,
        ))
        | ResidentIntegrityAdmissionDenial::BootstrapUnsupportedFormat(_)
        | ResidentIntegrityAdmissionDenial::BootstrapScopeMismatch(_) => {
            RecordAppendDenial::PlacementFormatMismatch
        }
        ResidentIntegrityAdmissionDenial::Validation(
            PhysicalIntegrityRejection::Unknown(_) | PhysicalIntegrityRejection::Indeterminate(_),
        )
        | ResidentIntegrityAdmissionDenial::SourceScopeMismatch => {
            RecordAppendDenial::ServingRequiresInspection
        }
        ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch
        | ResidentIntegrityAdmissionDenial::FrameGenerationChanged
        | ResidentIntegrityAdmissionDenial::RetainedRecordInvalidated
        | ResidentIntegrityAdmissionDenial::RetainedRecordChanged
        | ResidentIntegrityAdmissionDenial::Frame(_) => {
            RecordAppendDenial::ServingRequiresInspection
        }
    };
    RecordAppendError::Denied(denial)
}
