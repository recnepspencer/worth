//! A genuine process-killed V3 released drop, retained for C8/Store rejoin
//! tests that must share one pending descriptor and its physical controls.

use std::{
    cell::Cell,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, BlobCheckpointLimit, BlobIngestDeclaration,
    BlobIngestFailure, BlobReadLimits, BlobReclaimLimits, BlobReclaimRequest,
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    RecordAppendBatch,
};
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;

use super::*;

#[path = "pending_wal_world/first.rs"]
mod first;
#[path = "pending_wal_world/second.rs"]
mod second;
#[path = "pending_wal_world/selected_media.rs"]
mod selected_media;
#[path = "pending_wal_world/three_batch.rs"]
mod three_batch;

pub(super) fn selected_session_declared_extent(
    root: &Path,
) -> worth_store_physical_format::ExtentArenaRange {
    selected_media::selected_session_declared_extent(root)
}

const ROLE_ENV: &str = "WORTH_C11_PENDING_V3_CHILD_ROLE";
const MARKER_ENV: &str = "WORTH_C11_PENDING_V3_CHILD_MARKER";
const ROOT_ENV: &str = "WORTH_C11_PENDING_V3_CHILD_ROOT";
const WAL_SEGMENT_BYTES_ENV: &str = "WORTH_C11_PENDING_V3_WAL_SEGMENT_BYTES";
const CHILD_TEST: &str = "pending_wal_world::pending_wal_child";
const CHUNK: usize = 64 << 10;

pub(super) struct PendingWalWorld {
    root: PathBuf,
    _marker: tempfile::TempDir,
    next_descriptor: Cell<u64>,
}

impl PendingWalWorld {
    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    pub(super) fn kill_second_after_certified_retirement(&self) {
        self.kill_next_descriptor("second", true);
    }

    pub(super) fn kill_distinct_release_before_checkpoint(&self) {
        self.kill_next_descriptor("distinct", false);
    }

    fn kill_next_descriptor(&self, role: &str, checkpoint_changes: bool) {
        let ordinal = self.next_descriptor.get();
        self.next_descriptor
            .set(ordinal.checked_add(1).expect("bounded fixture sequence"));
        let marker = self
            ._marker
            .path()
            .join(format!("descriptor-{ordinal}-ready"));
        let checkpoint_before = fs::read(self.root.join("families/checkpoint.current")).unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
            .env(ROLE_ENV, role)
            .env(ROOT_ENV, &self.root)
            .env(MARKER_ENV, &marker)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(240);
        while !marker.is_file() {
            if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
                let _ = child.kill();
                let output = child.wait_with_output().unwrap();
                panic!(
                    "second pending V3 descriptor seam not reached: {} {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            thread::sleep(Duration::from_millis(10));
        }
        child.kill().unwrap();
        child.wait().unwrap();
        assert_eq!(fs::read(&marker).unwrap(), b"second-descriptor-wal");
        assert_eq!(
            fs::read(self.root.join("families/checkpoint.current")).unwrap() != checkpoint_before,
            checkpoint_changes,
            "only certified retirement may advance the checkpoint before the next descriptor"
        );
    }
}

impl Drop for PendingWalWorld {
    fn drop(&mut self) {
        let resolved = self
            .root
            .canonicalize()
            .expect("pending world root remains");
        assert!(resolved.starts_with(std::env::temp_dir().canonicalize().unwrap()));
        assert!(resolved
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("worth-store-c11-pending-v3-"));
        fs::remove_dir_all(resolved).expect("remove exact child-owned pending world");
    }
}

pub(super) fn first() -> PendingWalWorld {
    first_with_segment_bytes(None, false)
}

pub(super) fn first_with_wal_segment_bytes(bytes: NonZeroU64) -> PendingWalWorld {
    first_with_segment_bytes(Some(bytes), false)
}

pub(super) fn first_with_failed_ingest_control() -> PendingWalWorld {
    first_with_segment_bytes(None, true)
}

fn first_with_segment_bytes(
    wal_segment_bytes: Option<NonZeroU64>,
    failed_ingest_control: bool,
) -> PendingWalWorld {
    let marker_dir = tempfile::tempdir().unwrap();
    let marker = marker_dir.path().join("ready");
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(
            ROLE_ENV,
            if failed_ingest_control {
                "mixed-first"
            } else {
                "first"
            },
        )
        .env(MARKER_ENV, &marker)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(bytes) = wal_segment_bytes {
        command.env(WAL_SEGMENT_BYTES_ENV, bytes.get().to_string());
    }
    let mut child = command.spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(240);
    while !marker.is_file() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "pending V3 descriptor seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let root = PathBuf::from(fs::read_to_string(&marker).unwrap());
    assert!(root.is_dir());
    PendingWalWorld {
        root,
        _marker: marker_dir,
        next_descriptor: Cell::new(0),
    }
}

#[test]
fn pending_wal_world_reaches_verified_c8_and_sealed_serving() {
    let world = first();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        super::certified_release_serving::request(world.root()),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("real pending V3 WAL recovery failed: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("C8 pending V3 custody seal");
    super::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

#[test]
fn second_v3_descriptor_after_certified_retirement_checkpoint_is_durable() {
    let world = first();
    world.kill_second_after_certified_retirement();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        super::certified_release_serving::request(world.root()),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
                "third-process C8 blocked at {:?}, effects={}, planning={:?}",
                blocked.kind,
                blocked.recovery_effects(),
                blocked.evidence().planning_counters
            ),
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
                indeterminate,
            ) => panic!(
                "third-process C8 publication indeterminate, effects={}, handoff={:?}, reopen={:?}",
                indeterminate.recovery_effects(),
                indeterminate.handoff_failure(),
                indeterminate.reopen_failure()
            ),
            other => panic!("third-process C8 must recover second V3 WAL: {other:?}"),
        }
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("second V3 selected custody seal");
    super::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

#[test]
fn distinct_v3_descriptor_before_checkpoint_reaches_durable_wal() {
    let world = first();
    world.kill_distinct_release_before_checkpoint();
}

#[test]
fn distinct_v3_descriptor_before_checkpoint_is_durable() {
    let world = first();
    world.kill_distinct_release_before_checkpoint();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        super::certified_release_serving::request(world.root()),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
                "same-checkpoint distinct V3 blocked at {:?}, denial={:?}, artifact={:?}, effects={}",
                blocked.kind,
                blocked.evidence().planning_denial,
                blocked.evidence().artifact,
                blocked.recovery_effects()
            ),
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
                indeterminate,
            ) => panic!(
                "same-checkpoint distinct V3 publication indeterminate: handoff={:?}, reopen={:?}, effects={}",
                indeterminate.handoff_failure(),
                indeterminate.reopen_failure(),
                indeterminate.recovery_effects(),
            ),
            other => panic!("same-checkpoint distinct V3 sequence must recover: {other:?}"),
        }
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("two-batch C8/Store custody seal");
    super::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

#[test]
fn pending_wal_child() {
    let Some(role) = std::env::var_os(ROLE_ENV) else {
        return;
    };
    let marker = PathBuf::from(std::env::var_os(MARKER_ENV).expect("child marker"));
    match role.to_str() {
        Some("first") => first::child(&marker, false),
        Some("mixed-first") => first::child(&marker, true),
        Some("second") => second::child(
            Path::new(&std::env::var_os(ROOT_ENV).expect("second source root")),
            &marker,
        ),
        Some("distinct") => second::distinct_child(
            Path::new(&std::env::var_os(ROOT_ENV).expect("distinct source root")),
            &marker,
        ),
        other => panic!("unknown pending V3 child role: {other:?}"),
    }
}

fn park_at_descriptor_wal(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: BlobReclaimRequest,
    marker: &Path,
    marker_bytes: &[u8],
) {
    let first = serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterWalDurability);
    let cancelled = AtomicBool::new(false);
    thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(180);
            while !first.await_arrival() {
                if cancelled.load(Ordering::SeqCst) {
                    return;
                }
                assert!(Instant::now() < deadline, "manifest WAL durability");
            }
            let second =
                serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterWalDurability);
            first.release();
            while !second.await_arrival() {
                if cancelled.load(Ordering::SeqCst) {
                    return;
                }
                assert!(Instant::now() < deadline, "reservation WAL durability");
            }
            let third =
                serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterWalDurability);
            second.release();
            while !third.await_arrival() {
                if cancelled.load(Ordering::SeqCst) {
                    return;
                }
                assert!(Instant::now() < deadline, "descriptor WAL durability");
            }
            let pending = marker.with_extension("pending");
            fs::write(&pending, marker_bytes).unwrap();
            fs::rename(pending, marker).unwrap();
        });
        let result = serving
            .blobs()
            .unwrap()
            .reclaim(request)
            .and_then(|handle| handle.wait());
        cancelled.store(true, Ordering::SeqCst);
        panic!("released descriptor ended before durable WAL pause: {result:?}");
    });
}
