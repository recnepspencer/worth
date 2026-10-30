pub(crate) use std::num::NonZeroU64;
pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {
    std::process::Child, std::process::Command, std::process::Output, std::process::Stdio,
};

pub(crate) use crate::phase_three_support::limit_declaration;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_store::physical_runtime::certification::MediaFaultDirective,
    worth_store::physical_runtime::certification::MediaFaultSchedule,
    worth_store::physical_runtime::certification::MediaOperationRole,
    worth_store::physical_runtime::certification::MediaPauseGate,
};
pub(crate) use {
    worth_store::physical_runtime::FilesystemAccessPosture,
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority,
    worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration,
};
pub(crate) use {
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};

pub(crate) const CHILD_ROOT: &str = "WORTH_C8_PHASE7_FAULT_CHILD_ROOT";
pub(crate) const CRASH_MARKER: &str = "WORTH_C8_PHASE7_CLEANUP_CRASH_MARKER";

pub(crate) struct CleanupWorld {
    pub(crate) _parent: tempfile::TempDir,
    pub(crate) root: PathBuf,
}

impl CleanupWorld {
    pub(crate) fn oldest_wal(&self) -> PathBuf {
        let mut wal = std::fs::read_dir(self.root.join("families").join("wal"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "wal"))
            .collect::<Vec<_>>();
        wal.sort();
        wal.into_iter().next().unwrap()
    }
}

pub(crate) fn cleanup_world(label: &str) -> CleanupWorld {
    let parent = tempfile::tempdir().unwrap();
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_wal_segment_bytes(
        label,
        std::num::NonZeroU64::new(24 * 1024).unwrap(),
    )
    .unwrap();
    let retained = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x75; 32], b"checkpoint-covered");
    publish_checkpoint(&world);
    canonical_physical_mutation_acknowledgment(&world, [0x76; 32], b"current-retained");
    drop(world);
    let root = parent.path().join("persisted-store");
    copy_directory(&retained.persist(), &root);
    CleanupWorld {
        _parent: parent,
        root,
    }
}

pub(crate) fn recover_with_schedule(
    root: &Path,
    schedule: MediaFaultSchedule,
) -> worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff {
    let reopened = reopen_with_schedule(root, schedule);
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
        reopened.finish()
    else {
        panic!("cleanup failure cannot invalidate recovered publication")
    };
    *handoff
}

pub(crate) fn reopen_with_schedule(
    root: &Path,
    schedule: MediaFaultSchedule,
) -> worth_store_recovery_runtime::ReopenedPhysicalRecovery {
    let limits = cleanup_limits();
    let configuration =
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current();
    let authority =
        worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority::acquire_for_certification(
            root.to_path_buf(),
            configuration.clone(),
            limits,
            schedule,
        )
        .unwrap();
    let profile = authority.qualified_backend_profile().clone();
    let request = worth_store_recovery_runtime::PhysicalRecoveryOpenRequest::declare(
        root.to_path_buf(),
        configuration,
        profile,
        limits,
        authority,
    );
    request
        .admit()
        .unwrap()
        .discover()
        .unwrap()
        .select()
        .unwrap()
        .plan()
        .unwrap()
        .stage()
        .unwrap()
        .publish()
        .unwrap()
        .reopen()
        .unwrap()
}

pub(crate) fn publish_checkpoint(world: &PhysicalResidencyStoreWorld) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x77; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
}

pub(crate) fn cleanup_fault(directive: MediaFaultDirective) -> MediaFaultSchedule {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    authority
        .schedule(vec![authority.rule(
            worth_store::physical_runtime::certification::MediaOperationRole::Delete,
            1,
            directive,
        )])
        .unwrap()
}

pub(crate) fn empty_fault_schedule() -> MediaFaultSchedule {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    admission
        .fault_schedule_authority()
        .schedule(Vec::new())
        .unwrap()
}

pub(crate) fn paused_cleanup_schedule() -> (MediaFaultSchedule, MediaPauseGate) {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority.rule(
            worth_store::physical_runtime::certification::MediaOperationRole::Delete,
            1,
            worth_store::physical_runtime::certification::MediaFaultDirective::PauseAfter(
                gate.clone(),
            ),
        )])
        .unwrap();
    (schedule, gate)
}

pub(crate) fn cleanup_limits() -> PhysicalRecoveryLimits {
    let mut declaration = limit_declaration(4, 16, 4 * 1024 * 1024);
    declaration.manifest_entries = 4_096;
    declaration.wal_bytes = 4 * 1024 * 1024;
    declaration.redo_targets = 4_096;
    declaration.redo_bytes = 8 * 1024 * 1024;
    declaration.distinct_pages_and_extents = 4_096;
    declaration.operation_bindings = 4_096;
    declaration.staging_bytes = 64 * 1024 * 1024;
    declaration.dirty_frames = 4_096;
    declaration.publication_effects = 64;
    declaration.cleanup_candidates = 8;
    declaration.cleanup_bytes = 4 * 1024 * 1024;
    declaration.observation_bytes = 64 * 1024 * 1024;
    worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap()
}

pub(crate) fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

pub(crate) fn run_child(test: &str, root: &Path) -> Output {
    std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--ignored", "--nocapture"])
        .env(CHILD_ROOT, root)
        .output()
        .unwrap()
}

pub(crate) fn spawn_crashing_child(test: &str, root: &Path, marker: &Path) -> Child {
    std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--ignored", "--nocapture"])
        .env(CHILD_ROOT, root)
        .env(CRASH_MARKER, marker)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap()
}

pub(crate) fn required_child_root() -> PathBuf {
    std::env::var_os(CHILD_ROOT)
        .map(std::path::PathBuf::from)
        .unwrap()
}

pub(crate) fn required_crash_marker() -> PathBuf {
    std::env::var_os(CRASH_MARKER)
        .map(std::path::PathBuf::from)
        .unwrap()
}

pub(crate) fn assert_child_succeeded(name: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{name} child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
