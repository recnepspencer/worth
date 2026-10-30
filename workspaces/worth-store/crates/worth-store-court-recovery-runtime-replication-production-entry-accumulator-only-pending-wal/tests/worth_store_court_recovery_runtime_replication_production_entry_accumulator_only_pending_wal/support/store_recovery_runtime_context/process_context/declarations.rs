pub(crate) use std::collections::BTreeMap;
pub(crate) use std::num::NonZeroU64;
pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {std::process::Command, std::process::Output};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity;
pub(crate) use {
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::RecoveryCleanupTarget,
};
pub(crate) use {
    worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution,
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};
pub(crate) use {
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
    worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits,
};

pub(crate) use crate::artifact_snapshot::process_context::ArtifactSnapshot;
pub(crate) use crate::assertions::process_context::assert_expected_posture;

pub(crate) const CHILD_ROOT: &str = "WORTH_C8_PHASE7_CHILD_ROOT";
pub(crate) const CLEANUP_BYTES: &str = "WORTH_C8_PHASE7_CLEANUP_BYTES";
pub(crate) const CLEANUP_CANDIDATES: &str = "WORTH_C8_PHASE7_CLEANUP_CANDIDATES";
pub(crate) const EXPECTED_WAL: &str = "WORTH_C8_PHASE7_EXPECTED_WAL";
pub(crate) const EXPECTED_POSTURE: &str = "WORTH_C8_PHASE7_EXPECTED_POSTURE";
pub(crate) const EXPECTED_DEFERRED: &str = "WORTH_C8_PHASE7_EXPECTED_DEFERRED";
pub(crate) const CANCEL_AFTER: &str = "WORTH_C8_PHASE7_CANCEL_AFTER";
pub(crate) const WRITER_MODE: &str = "WORTH_C8_PHASE7_WRITER_MODE";
pub(crate) const WRITER_TEST: &str =
    "process::recovery_runtime::phase_seven_cleanup::process_context::writer_process";
pub(crate) const CLEANER_TEST: &str =
    "process::recovery_runtime::phase_seven_cleanup::process_context::cleanup_process";

pub(crate) struct ProcessWorld {
    pub(crate) parent: tempfile::TempDir,
    pub(crate) root: PathBuf,
}

impl ProcessWorld {
    pub(crate) fn write(mode: &str) -> Self {
        let parent = tempfile::tempdir().unwrap();
        let marker = parent.path().join("persisted-root");
        let writer = run_child(
            WRITER_TEST,
            parent.path(),
            [
                (CHILD_ROOT, marker.as_os_str().to_os_string()),
                (WRITER_MODE, mode.into()),
            ],
        );
        assert_child_succeeded("writer", &writer);
        let root = std::path::PathBuf::from(std::fs::read_to_string(marker).unwrap());
        Self { parent, root }
    }

    pub(crate) fn cleanup(&self, bytes: u64, posture: &str, candidate: &Path) -> Output {
        self.cleanup_with_candidate_limit(bytes, 8, posture, candidate)
    }

    pub(crate) fn cleanup_with_candidate_limit(
        &self,
        bytes: u64,
        candidates: u64,
        posture: &str,
        candidate: &Path,
    ) -> Output {
        let expected_deferred = if posture == "candidate-limit" {
            wal_files(&self.root).len().saturating_sub(2) as u64
        } else {
            0
        };
        run_child(
            CLEANER_TEST,
            self.parent.path(),
            [
                (CHILD_ROOT, self.root.as_os_str().to_os_string()),
                (CLEANUP_BYTES, bytes.to_string().into()),
                (CLEANUP_CANDIDATES, candidates.to_string().into()),
                (EXPECTED_WAL, candidate.file_name().unwrap().to_os_string()),
                (EXPECTED_POSTURE, posture.into()),
                (EXPECTED_DEFERRED, expected_deferred.to_string().into()),
            ],
        )
    }

    pub(crate) fn cleanup_with_cancellation(
        &self,
        bytes: u64,
        cancel_after: u64,
        candidate: &Path,
    ) -> Output {
        run_child(
            CLEANER_TEST,
            self.parent.path(),
            [
                (CHILD_ROOT, self.root.as_os_str().to_os_string()),
                (CLEANUP_BYTES, bytes.to_string().into()),
                (CLEANUP_CANDIDATES, "8".into()),
                (EXPECTED_WAL, candidate.file_name().unwrap().to_os_string()),
                (EXPECTED_POSTURE, format!("cancel-{cancel_after}").into()),
                (EXPECTED_DEFERRED, "0".into()),
                (CANCEL_AFTER, cancel_after.to_string().into()),
            ],
        )
    }

    pub(crate) fn oldest_wal(&self) -> PathBuf {
        self.wal_files().into_iter().next().unwrap()
    }

    pub(crate) fn newest_wal(&self) -> PathBuf {
        self.wal_files().into_iter().last().unwrap()
    }

    pub(crate) fn wal_files(&self) -> Vec<PathBuf> {
        wal_files(&self.root)
    }
}

pub(crate) fn preserve_wal(
    world: &PhysicalResidencyStoreWorld,
    preserved: &mut BTreeMap<std::ffi::OsString, Vec<u8>>,
) {
    for path in wal_files(world.root()) {
        preserved.insert(
            path.file_name().unwrap().to_owned(),
            std::fs::read(path).unwrap(),
        );
    }
}

pub(crate) fn restore_preserved_wal(root: &Path, preserved: BTreeMap<std::ffi::OsString, Vec<u8>>) {
    for (name, bytes) in preserved {
        let path = root.join("families").join("wal").join(name);
        if !path.exists() {
            std::fs::write(path, bytes).unwrap();
        }
    }
}

pub(crate) fn publish_checkpoint(world: &PhysicalResidencyStoreWorld) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x74; 32]),
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

pub(crate) fn cleanup_limits(
    cleanup_bytes: u64,
    cleanup_candidates: u64,
) -> PhysicalRecoveryLimits {
    let mut declaration = limit_declaration(4, 16, 4 * 1024 * 1024);
    declaration.manifest_entries = 4_096;
    declaration.wal_bytes = 4 * 1024 * 1024;
    declaration.redo_targets = 4_096;
    declaration.redo_bytes = 8 * 1024 * 1024;
    declaration.distinct_pages_and_extents = 4_096;
    declaration.operation_bindings = 4_096;
    declaration.staging_bytes = 64 * 1024 * 1024;
    declaration.recovery_memory_bytes = 64 * 1024 * 1024;
    declaration.dirty_frames = 4_096;
    declaration.publication_effects = 64;
    declaration.cleanup_candidates = cleanup_candidates;
    declaration.cleanup_bytes = cleanup_bytes;
    declaration.observation_bytes = 64 * 1024 * 1024;
    worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap()
}

pub(crate) fn wal_files(root: &Path) -> Vec<PathBuf> {
    let mut files = std::fs::read_dir(root.join("families").join("wal"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .and_then(
                    worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity::parse,
                )
                .is_some()
        })
        .collect::<Vec<_>>();
    files.sort_by_key(|path| {
        worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity::parse(
            path.file_name().unwrap().to_str().unwrap(),
        )
        .unwrap()
    });
    files
}

pub(crate) fn run_child<const N: usize>(
    test: &str,
    temporary_root: &Path,
    environment: [(&str, std::ffi::OsString); N],
) -> Output {
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", test, "--ignored", "--nocapture"])
        .envs(environment)
        .env("TMP", temporary_root)
        .env("TEMP", temporary_root)
        .env("TMPDIR", temporary_root);
    command.output().unwrap()
}

pub(crate) fn required_path(name: &str) -> PathBuf {
    std::env::var_os(name)
        .map(std::path::PathBuf::from)
        .unwrap()
}

pub(crate) fn required_text(name: &str) -> String {
    std::env::var(name).unwrap()
}

pub(crate) fn assert_child_succeeded(name: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{name} child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
