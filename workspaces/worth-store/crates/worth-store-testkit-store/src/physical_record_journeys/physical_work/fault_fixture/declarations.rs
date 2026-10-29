pub use {std::path::Path};

pub use {worth_proof::TransitionOutcome};
pub use {worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy, worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWorkProfileDeclaration, worth_store::physical_runtime::ServingPhysicalRuntime};
pub use {worth_store_physical_backend::FilesystemAccessPosture, worth_store_physical_backend::MediaFaultDirective, worth_store_physical_backend::MediaFaultSchedule, worth_store_physical_backend::MediaOperationRole};

pub fn serving_from_open_with_schedule(
    root: &Path,
    schedule: MediaFaultSchedule,
) -> ServingPhysicalRuntime {
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("scheduled media should admit"),
    };
    let (format, _, access) = crate::physical_record_journeys::configuration();
    crate::physical_record_journeys::success(open_record_store!(media, |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    }))
}

pub fn serving_from_open_with_positioned_write_fault(
    root: &Path,
    profile: PhysicalWorkProfileDeclaration,
    directive: MediaFaultDirective,
) -> ServingPhysicalRuntime {
    serving_from_open_with_positioned_write_fault_at(root, profile, 1, directive)
}

pub fn serving_from_open_with_positioned_write_fault_at(
    root: &Path,
    profile: PhysicalWorkProfileDeclaration,
    ordinal: u64,
    directive: MediaFaultDirective,
) -> ServingPhysicalRuntime {
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let schedule = authority
        .schedule(vec![authority.rule(
            worth_store_physical_backend::MediaOperationRole::PositionedWrite,
            ordinal,
            directive,
        )])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("faulted media should admit"),
    };
    let (format, _, access) = crate::physical_record_journeys::configuration();
    crate::physical_record_journeys::success(open_record_store!(media, |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability).with_physical_work_profile(profile)
    },))
}

pub fn serving_from_open_with_positioned_read_fault(
    root: &Path,
    ordinal: u64,
    directive: MediaFaultDirective,
) -> ServingPhysicalRuntime {
    serving_from_open_with_positioned_read_fault_policy(root, ordinal, directive, None)
}

pub fn serving_from_open_with_positioned_read_fault_and_policy(
    root: &Path,
    ordinal: u64,
    directive: MediaFaultDirective,
    policy: AdmittedPhysicalRecordResidencyPolicy,
) -> ServingPhysicalRuntime {
    serving_from_open_with_positioned_read_fault_policy(root, ordinal, directive, Some(policy))
}

pub fn serving_from_open_with_positioned_read_fault_policy(
    root: &Path,
    ordinal: u64,
    directive: MediaFaultDirective,
    policy: Option<AdmittedPhysicalRecordResidencyPolicy>,
) -> ServingPhysicalRuntime {
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let schedule = authority
        .schedule(vec![authority.rule(
            worth_store_physical_backend::MediaOperationRole::PositionedRead,
            ordinal,
            directive,
        )])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("faulted media should admit"),
    };
    let (format, _, access) = crate::physical_record_journeys::configuration();
    let durability = crate::physical_record_journeys::durability(&media);
    let open = worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability);
    let open = match policy {
        Some(policy) => open.with_residency_policy(policy),
        None => open,
    };
    crate::physical_record_journeys::success(media.open_record_store(open))
}

pub fn serving_from_open_with_paused_positioned_read_failure(
    root: &Path,
    ordinal: u64,
) -> (
    ServingPhysicalRuntime,
    worth_store_physical_backend::MediaPauseGate,
) {
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority.rule(
            worth_store_physical_backend::MediaOperationRole::PositionedRead,
            ordinal,
            worth_store_physical_backend::MediaFaultDirective::PauseBeforeThenFailBefore {
                gate: gate.clone(),
                kind: std::io::ErrorKind::Other,
                raw_os_error: None,
            },
        )])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("paused failing media should admit"),
    };
    let (format, _, access) = crate::physical_record_journeys::configuration();
    (
        crate::physical_record_journeys::success(open_record_store!(media, |durability| {
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
        })),
        gate,
    )
}

pub fn serving_from_open_with_identified_positioned_read_fault(
    root: &Path,
    ordinal: u64,
    directive: MediaFaultDirective,
) -> ServingPhysicalRuntime {
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let schedule = authority
        .schedule(vec![authority
            .rule(worth_store_physical_backend::MediaOperationRole::PositionedRead, ordinal, directive)
            .for_identified_operation_ordinal()])
        .unwrap();
    serving_from_open_with_schedule(root, schedule)
}

pub fn serving_from_open_with_one_write_pause_and_profile(
    root: &Path,
    profile: PhysicalWorkProfileDeclaration,
) -> (
    ServingPhysicalRuntime,
    worth_store_physical_backend::MediaPauseGate,
) {
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::PositionedWrite,
                1,
                worth_store_physical_backend::MediaFaultDirective::PauseBefore(gate.clone()),
            )
            .for_identified_operation_ordinal()])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("paused media should admit"),
    };
    let (format, _, access) = crate::physical_record_journeys::configuration();
    (
        crate::physical_record_journeys::success(open_record_store!(media, |durability| {
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability).with_physical_work_profile(profile)
        },)),
        gate,
    )
}