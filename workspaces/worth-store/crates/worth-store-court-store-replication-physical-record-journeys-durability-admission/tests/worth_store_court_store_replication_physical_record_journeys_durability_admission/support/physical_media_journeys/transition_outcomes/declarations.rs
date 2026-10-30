pub(crate) use {
    crate::physical_media_journeys::admit_runtime, crate::physical_media_journeys::media_admission,
    crate::physical_media_journeys::process_contender::spawn_lease_holder,
    worth_proof::TransitionOutcome, worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::MediaShutdownOutcome,
    worth_store_physical_backend::FilesystemAccessPosture,
};

pub(crate) use worth_store::physical_runtime::certification::MediaFaultDirective;
pub(crate) use worth_store::physical_runtime::MediaAdmissionInspectionCause;
pub(crate) use {
    worth_store_physical_backend::MediaOperationRole,
    worth_store_physical_backend::MediaQualificationDenial,
    worth_store_physical_backend::MediaQualificationRebindRequired,
};

pub(crate) fn success(
    runtime: worth_store::physical_runtime::AdmittedPhysicalRuntime,
    admission: FilesystemMediaAdmission,
) -> worth_store::physical_runtime::MediaOwnedPhysicalRuntime {
    match runtime.try_admit_filesystem_media(admission).into_raw() {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("expected media admission success"),
    }
}
