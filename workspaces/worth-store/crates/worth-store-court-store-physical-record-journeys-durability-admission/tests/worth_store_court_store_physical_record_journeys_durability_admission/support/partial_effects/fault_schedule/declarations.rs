pub(crate) use {worth_store::physical_runtime::certification::CertificationMediaFaultAuthority, worth_store::physical_runtime::certification::MediaFaultDirective, worth_store::physical_runtime::certification::MediaPauseGate};
pub(crate) use {worth_store::physical_runtime::FilesystemMediaAdmission};
pub(crate) use {worth_store_physical_backend::FilesystemAccessPosture, worth_store_physical_backend::MediaOperationRole};

pub(crate) fn fault_admission(case: &str) -> (FilesystemMediaAdmission, Option<MediaPauseGate>) {
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::certification(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let (role, ordinal, directive, gate) = match case {
        "pass-through" => return (admission, None),
        "before-root-creation" => fail_before(worth_store_physical_backend::MediaOperationRole::CreateDirectory, 1),
        "short-identity-prefix" => (
            worth_store_physical_backend::MediaOperationRole::PositionedWrite,
            1,
            worth_store::physical_runtime::certification::MediaFaultDirective::AllowPrefix { bytes: 17 },
            None,
        ),
        "directory-barrier-indeterminate" => {
            fail_barrier(worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication, 1)
        }
        "abrupt-after-replacement" => pause_after(&authority, worth_store_physical_backend::MediaOperationRole::AtomicReplace, 1),
        "after-fixed-directories" => (
            worth_store_physical_backend::MediaOperationRole::CreateDirectory,
            4,
            worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
            None,
        ),
        "before-staged-identity-create" => fail_before(worth_store_physical_backend::MediaOperationRole::CreateNew, 1),
        "after-staged-identity-create" => fail_before(worth_store_physical_backend::MediaOperationRole::PositionedWrite, 1),
        "after-complete-identity-write" => fail_before(worth_store_physical_backend::MediaOperationRole::SynchronizeFileState, 1),
        "file-barrier-denial" => fail_barrier(worth_store_physical_backend::MediaOperationRole::SynchronizeFileState, 1),
        "after-identity-file-sync" => fail_before(worth_store_physical_backend::MediaOperationRole::AtomicReplace, 1),
        "after-directory-sync-before-observation" => pause_after(
            &authority,
            worth_store_physical_backend::MediaOperationRole::SynchronizeRootParentPublication,
            1,
        ),
        "qualification-positioned-write" => (
            worth_store_physical_backend::MediaOperationRole::PositionedWrite,
            2,
            worth_store::physical_runtime::certification::MediaFaultDirective::AllowPrefix { bytes: 31 },
            None,
        ),
        "qualification-append" => fail_before(worth_store_physical_backend::MediaOperationRole::Append, 1),
        "qualification-truncate" => fail_before(worth_store_physical_backend::MediaOperationRole::Truncate, 1),
        "qualification-allocation" => fail_before(worth_store_physical_backend::MediaOperationRole::Allocate, 1),
        "qualification-metadata" => fail_before(worth_store_physical_backend::MediaOperationRole::ReadMetadata, 1),
        "qualification-list" => fail_before(worth_store_physical_backend::MediaOperationRole::ListDirectory, 1),
        "cleanup-delete" => fail_before(worth_store_physical_backend::MediaOperationRole::Delete, 1),
        "cleanup-directory-barrier" => {
            fail_barrier(worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication, 3)
        }
        "before-lock-release" => {
            let gate = authority.pause_gate();
            let schedule = authority
                .schedule(Vec::new())
                .unwrap()
                .pause_before_lease_release(gate.clone());
            return (admission.with_fault_schedule(schedule), Some(gate));
        }
        "after-lock-release" => {
            pause_after(&authority, worth_store_physical_backend::MediaOperationRole::ReleaseMutationLease, 1)
        }
        _ => panic!("unknown fault case: {case}"),
    };
    let schedule = authority
        .schedule(vec![authority.rule(role, ordinal, directive)])
        .unwrap();
    (admission.with_fault_schedule(schedule), gate)
}

pub(crate) fn fail_before(
    role: MediaOperationRole,
    ordinal: u64,
) -> (
    MediaOperationRole,
    u64,
    MediaFaultDirective,
    Option<MediaPauseGate>,
) {
    (
        role,
        ordinal,
        worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::PermissionDenied,
            raw_os_error: None,
        },
        None,
    )
}

pub(crate) fn fail_barrier(
    role: MediaOperationRole,
    ordinal: u64,
) -> (
    MediaOperationRole,
    u64,
    MediaFaultDirective,
    Option<MediaPauseGate>,
) {
    (
        role,
        ordinal,
        worth_store::physical_runtime::certification::MediaFaultDirective::FailBarrier {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
        None,
    )
}

pub(crate) fn pause_after(
    authority: &CertificationMediaFaultAuthority,
    role: MediaOperationRole,
    ordinal: u64,
) -> (
    MediaOperationRole,
    u64,
    MediaFaultDirective,
    Option<MediaPauseGate>,
) {
    let gate = authority.pause_gate();
    (
        role,
        ordinal,
        worth_store::physical_runtime::certification::MediaFaultDirective::PauseAfter(gate.clone()),
        Some(gate),
    )
}