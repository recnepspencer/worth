pub use {std::sync::atomic::AtomicBool, std::sync::atomic::Ordering};

pub use worth_proof::TransitionOutcome;
pub use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::PhysicalReadProtectionPolicy,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalPolicy,
    worth_store::physical_runtime::PhysicalWorkProfileDeclaration,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};
pub use {
    worth_store_physical_backend::CertificationMediaFaultActivation,
    worth_store_physical_backend::MediaPauseGate,
};
pub use {
    worth_store_physical_backend::FilesystemAccessPosture,
    worth_store_physical_backend::MediaFaultDirective,
    worth_store_physical_backend::MediaOperationRole,
};

pub use {
    crate::physical_record_journeys::maintenance_interference::append,
    crate::physical_record_journeys::maintenance_interference::initialize,
    crate::physical_record_journeys::maintenance_interference::limits,
    crate::physical_record_journeys::maintenance_interference::placement,
};

pub struct FileSyncPauseRelease<'gate>(pub &'gate MediaPauseGate);

impl<'gate> FileSyncPauseRelease<'gate> {
    pub fn new(gate: &'gate MediaPauseGate) -> Self {
        Self(gate)
    }
}

impl Drop for FileSyncPauseRelease<'_> {
    fn drop(&mut self) {
        self.0.release();
    }
}

pub fn open_with_file_sync_pause(
    root: &std::path::Path,
    profile: PhysicalWorkProfileDeclaration,
) -> (
    worth_store::physical_runtime::ServingPhysicalRuntime,
    MediaPauseGate,
    CertificationMediaFaultActivation,
) {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::production(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::SynchronizeFileState,
                1,
                worth_store_physical_backend::MediaFaultDirective::PauseBefore(gate.clone()),
            )
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap(),
    )
    .unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("faulted media should admit"),
    };
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let durability = crate::physical_record_journeys::durability_with_wal_policy(
        &media,
        worth_store::physical_runtime::PhysicalWalPolicy::segmented(
            worth_store::physical_runtime::WalSegmentByteLimit::new(
                std::num::NonZeroU64::new(512 * 1024).unwrap(),
            ),
            worth_store::physical_runtime::WalSegmentInventoryLimit::new(
                std::num::NonZeroU32::new(64).unwrap(),
            ),
        ),
    );
    let initialization = worth_store::physical_runtime::PhysicalRecordInitialization::new(
        format,
        crate::physical_record_journeys::maintenance_interference::placement_for(format),
        access,
        durability,
    )
    .with_residency_policy(
        crate::physical_record_journeys::maintenance_interference::residency(
            format,
            crate::physical_record_journeys::maintenance_interference::RESIDENT_BYTES,
        ),
    )
    .with_read_protection_policy(
        worth_store::physical_runtime::PhysicalReadProtectionPolicy::default(),
    )
    .with_physical_work_profile(profile);
    let serving =
        crate::physical_record_journeys::success(media.initialize_record_store(initialization));
    (serving, gate, activation)
}

pub fn reopen_with_file_sync_pause(
    root: &std::path::Path,
    profile: PhysicalWorkProfileDeclaration,
) -> (
    worth_store::physical_runtime::ServingPhysicalRuntime,
    MediaPauseGate,
    CertificationMediaFaultActivation,
) {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::production(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::SynchronizeFileState,
                1,
                worth_store_physical_backend::MediaFaultDirective::PauseBefore(gate.clone()),
            )
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap(),
    )
    .unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("faulted media should admit"),
    };
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let durability = crate::physical_record_journeys::durability_with_wal_policy(
        &media,
        worth_store::physical_runtime::PhysicalWalPolicy::segmented(
            worth_store::physical_runtime::WalSegmentByteLimit::new(
                std::num::NonZeroU64::new(512 * 1024).unwrap(),
            ),
            worth_store::physical_runtime::WalSegmentInventoryLimit::new(
                std::num::NonZeroU32::new(64).unwrap(),
            ),
        ),
    );
    let open = worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
        .with_residency_policy(
            crate::physical_record_journeys::maintenance_interference::residency(
                format,
                crate::physical_record_journeys::maintenance_interference::RESIDENT_BYTES,
            ),
        )
        .with_read_protection_policy(
            worth_store::physical_runtime::PhysicalReadProtectionPolicy::default(),
        )
        .with_physical_work_profile(profile);
    let serving = crate::physical_record_journeys::success(media.open_record_store(open));
    (serving, gate, activation)
}
