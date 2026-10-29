pub(crate) use {std::num::NonZeroU32, std::path::Path};

pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::certification::MediaFaultDirective};
pub(crate) use {worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalGroupBarrierFailureCause, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome, worth_store::physical_runtime::SealedPhysicalDurabilityGroupMembers};
pub(crate) use {worth_store_physical_backend::ArtifactTreeFailureKind, worth_store_physical_backend::BackendTargetProfile, worth_store_physical_backend::CertificationMediaFaultActivation, worth_store_physical_backend::FilesystemAccessPosture, worth_store_physical_backend::MediaOperationRole, worth_store_physical_backend::WalDurabilityBarrier};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_group_limit};

pub(crate) fn append(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: PhysicalMutationIdempotencyMaterial,
) -> SealedPhysicalDurabilityGroupMembers {
    let prepared = worth_store_testkit_store::physical_record_journeys::durability_admission::wal_append::prepared(submission, placement, material, b"barrier-redo");
    match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(prepared, Vec::new())) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("barrier evidence requires one canonical appended WAL member"),
    }
}

pub(crate) fn required_barrier(profile: BackendTargetProfile) -> WalDurabilityBarrier {
    match profile {
        worth_store_physical_backend::BackendTargetProfile::PosixFileFsyncDirSync => worth_store_physical_backend::WalDurabilityBarrier::WalFileFsync,
        worth_store_physical_backend::BackendTargetProfile::WindowsFlushFileBuffers => {
            worth_store_physical_backend::WalDurabilityBarrier::WindowsFlushFileBuffers
        }
        _ => panic!("production filesystem admission must select a supported WAL barrier"),
    }
}

pub(crate) fn media_with_synchronization_fault(
    root: &Path,
    directive: MediaFaultDirective,
) -> (
    worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
    CertificationMediaFaultActivation,
) {
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::certification(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(worth_store_physical_backend::MediaOperationRole::SynchronizeFileState, 1, directive)
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => (media, activation),
        _ => panic!("fault-scheduled media admission must succeed"),
    }
}