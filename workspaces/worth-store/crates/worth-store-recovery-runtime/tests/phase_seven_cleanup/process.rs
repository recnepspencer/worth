use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::phase_three_support::{limit_declaration, recovery_request_with_limits};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimits, PhysicalRecoveryOutcome, RecoveryCleanupTarget,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    PhysicalResidencyStoreWorld,
};

#[path = "process/artifact_snapshot.rs"]
mod artifact_snapshot;
#[path = "process/assertions.rs"]
mod assertions;
use artifact_snapshot::ArtifactSnapshot;
use assertions::assert_expected_posture;

const CHILD_ROOT: &str = "WORTH_C8_PHASE7_CHILD_ROOT";
const CLEANUP_BYTES: &str = "WORTH_C8_PHASE7_CLEANUP_BYTES";
const CLEANUP_CANDIDATES: &str = "WORTH_C8_PHASE7_CLEANUP_CANDIDATES";
const EXPECTED_WAL: &str = "WORTH_C8_PHASE7_EXPECTED_WAL";
const EXPECTED_POSTURE: &str = "WORTH_C8_PHASE7_EXPECTED_POSTURE";
const EXPECTED_DEFERRED: &str = "WORTH_C8_PHASE7_EXPECTED_DEFERRED";
const CANCEL_AFTER: &str = "WORTH_C8_PHASE7_CANCEL_AFTER";
const WRITER_MODE: &str = "WORTH_C8_PHASE7_WRITER_MODE";
const WRITER_TEST: &str = "process::phase_seven_writer_process";
const CLEANER_TEST: &str = "process::phase_seven_cleanup_process";

#[test]
#[ignore = "launched by the Phase 7 process-boundary parent"]
fn phase_seven_writer_process() {
    let marker = required_path(CHILD_ROOT);
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery_with_wal_segment_bytes(
        "c8-phase7-writer",
        NonZeroU64::new(24 * 1024).unwrap(),
    )
    .unwrap();
    let retained_root = world.retained_root();
    let mode = required_text(WRITER_MODE);
    let mut preserved_wal = BTreeMap::new();
    canonical_physical_mutation_acknowledgment(&world, [0x71; 32], b"before-checkpoint");
    if mode == "multiple-settled" {
        preserve_wal(&world, &mut preserved_wal);
        for ordinal in 0..10_u8 {
            let mut key = [0x70; 32];
            key[0] = ordinal;
            let payload = vec![ordinal; 4 * 1024];
            canonical_physical_mutation_acknowledgment(&world, key, &payload);
            preserve_wal(&world, &mut preserved_wal);
        }
    }
    publish_checkpoint(&world);
    match mode.as_str() {
        "settled" | "multiple-settled" => {
            canonical_physical_mutation_acknowledgment(
                &world,
                [0x72; 32],
                b"after-checkpoint-settled",
            );
        }
        "unresolved" => {
            canonical_durable_wal_attempt_without_execution(
                &world,
                [0x73; 32],
                b"after-checkpoint-unresolved",
            );
        }
        mode => panic!("unsupported writer mode: {mode}"),
    };
    drop(world);
    let root = retained_root.persist();
    restore_preserved_wal(&root, preserved_wal);
    assert!(
        wal_files(&root).len() >= 2,
        "ordinary writer must rotate WAL"
    );
    std::fs::write(marker, root.to_string_lossy().as_bytes()).unwrap();
}

#[test]
#[ignore = "launched by the Phase 7 process-boundary parent"]
fn phase_seven_cleanup_process() {
    let root = required_path(CHILD_ROOT);
    let mut artifacts = ArtifactSnapshot::capture(&root);
    let expected = required_text(EXPECTED_WAL);
    let cleanup_bytes = required_text(CLEANUP_BYTES).parse::<u64>().unwrap();
    let cleanup_candidates = required_text(CLEANUP_CANDIDATES).parse::<u64>().unwrap();
    let expected_deferred = required_text(EXPECTED_DEFERRED).parse::<u64>().unwrap();
    let limits = cleanup_limits(cleanup_bytes, cleanup_candidates);
    let request = recovery_request_with_limits(&root, limits);
    let reopened = request
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
        .unwrap();
    artifacts.include_recovery_created(
        &root,
        reopened.publication_expectation().created_artifacts(),
    );
    let outcome = match std::env::var(CANCEL_AFTER).ok() {
        Some(cancel_after) => {
            let cancellation = if cancel_after == "0" {
                reopened.cleanup_cancellation_before_first().unwrap()
            } else {
                reopened
                    .cleanup_cancellation_after_removal(cancel_after.parse::<u64>().unwrap() - 1)
                    .unwrap()
            };
            reopened.finish_with_cleanup_cancellation(cancellation)
        }
        None => reopened.finish(),
    };
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("cleanup debt must not invalidate recovered success")
    };
    let posture = handoff.cleanup_posture();
    let evidence = posture.evidence();
    let expected_identity = WalSegmentArtifactIdentity::parse(&expected).unwrap();
    let disposition = evidence
        .dispositions()
        .iter()
        .find(|disposition| disposition.target() == &RecoveryCleanupTarget::Wal(expected_identity))
        .expect("checkpoint-covered artifact has one disposition");
    artifacts.assert_reconciled(&root, evidence);
    assert_eq!(evidence.counters().eligible_after_cleanup, 0);
    assert_expected_posture(
        posture,
        disposition,
        expected_identity,
        expected_deferred,
        cleanup_bytes
            + std::fs::metadata(root.join("families").join("checkpoint.current"))
                .unwrap()
                .len(),
    );
}

pub(crate) struct ProcessWorld {
    parent: tempfile::TempDir,
    root: PathBuf,
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
        let root = PathBuf::from(std::fs::read_to_string(marker).unwrap());
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

fn preserve_wal(
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

fn restore_preserved_wal(root: &Path, preserved: BTreeMap<std::ffi::OsString, Vec<u8>>) {
    for (name, bytes) in preserved {
        let path = root.join("families").join("wal").join(name);
        if !path.exists() {
            std::fs::write(path, bytes).unwrap();
        }
    }
}

fn publish_checkpoint(world: &PhysicalResidencyStoreWorld) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x74; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}

fn cleanup_limits(cleanup_bytes: u64, cleanup_candidates: u64) -> PhysicalRecoveryLimits {
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
    PhysicalRecoveryLimits::admit(declaration).unwrap()
}

fn wal_files(root: &Path) -> Vec<PathBuf> {
    let mut files = std::fs::read_dir(root.join("families").join("wal"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .and_then(WalSegmentArtifactIdentity::parse)
                .is_some()
        })
        .collect::<Vec<_>>();
    files.sort_by_key(|path| {
        WalSegmentArtifactIdentity::parse(path.file_name().unwrap().to_str().unwrap()).unwrap()
    });
    files
}

fn run_child<const N: usize>(
    test: &str,
    temporary_root: &Path,
    environment: [(&str, std::ffi::OsString); N],
) -> Output {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", test, "--ignored", "--nocapture"])
        .envs(environment)
        .env("TMP", temporary_root)
        .env("TEMP", temporary_root)
        .env("TMPDIR", temporary_root);
    command.output().unwrap()
}

fn required_path(name: &str) -> PathBuf {
    std::env::var_os(name).map(PathBuf::from).unwrap()
}

fn required_text(name: &str) -> String {
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
