//! A genuine process-killed V3 released drop, retained for C8/Store rejoin
//! tests that must share one pending descriptor and its physical controls.

use std::{
    cell::Cell,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobIngestFailure, BlobReadFailure, BlobReadLimits,
    BlobReadOpenFailure, BlobReclaimLimits, BlobReclaimRequest, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, RecordAppendBatch,
    RecordReadDenial,
};
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;

use super::*;

#[path = "pending_wal_world/descriptor_park.rs"]
mod descriptor_park;
#[path = "pending_wal_world/first.rs"]
mod first;
#[path = "pending_wal_world/second.rs"]
mod second;
#[path = "pending_wal_world/selected_media.rs"]
mod selected_media;
#[path = "pending_wal_world/successor.rs"]
mod successor;
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
const FIRST_WORLD_ENV: &str = "WORTH_C11_PENDING_V3_FIRST_WORLD";
const CHILD_TEST: &str = "pending_wal_world::pending_wal_child";
const CHUNK: usize = 64 << 10;

use descriptor_park::park_at_descriptor_wal;
pub(super) use first::{
    first, first_with_failed_ingest_control, first_with_resume_frontier,
    first_with_wal_segment_bytes, published_above_checkpoint, Tail, Workload,
};

pub(super) struct PendingWalWorld {
    root: PathBuf,
    first: first::World,
    objects: Vec<[u8; 16]>,
    _marker: tempfile::TempDir,
    next_descriptor: Cell<u64>,
}

impl PendingWalWorld {
    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    /// A recovery request with the limits this world's WAL and batches need.
    pub(super) fn recovery_request(
        &self,
    ) -> worth_store_recovery_runtime::PhysicalRecoveryOpenRequest {
        self.first.recovery_request(&self.root)
    }

    /// Requires every object the first child published and did not release
    /// to read back byte-exact, and the released one to be gone.
    pub(super) fn assert_objects_read_back(
        &self,
        serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    ) {
        let scope = admitted_blob_scope("c11.recovery.pending-v3.scope");
        let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
        let blobs = serving.blobs().unwrap();
        let expected = self.first.object_bytes();
        assert_eq!(self.objects.len(), expected.len());
        for (index, (object, expected)) in self.objects.iter().zip(expected).enumerate() {
            let resolved = blobs.resolve_publication(*object, 1, &scope, limits);
            if index == 0 && self.first.releases_first_object() {
                match resolved {
                    Err(BlobReadOpenFailure::PublicationNotFound) => {}
                    Err(BlobReadOpenFailure::Read(BlobReadFailure::RecordRead(error)))
                        if error.denial() == RecordReadDenial::RecordNotFound => {}
                    other => panic!("released first object remained visible: {other:?}"),
                }
                continue;
            }
            let mut read = blobs
                .read(resolved.unwrap(), &scope, 0, expected.len() as u64, limits)
                .unwrap();
            let mut observed = vec![0_u8; expected.len()];
            let mut used = 0;
            while used < observed.len() {
                let count = read.read_next(&mut observed[used..]).unwrap();
                assert!(count > 0, "object {index} ended at byte {used}");
                used += count;
            }
            assert!(observed == expected, "object {index} read back changed");
        }
    }

    pub(super) fn kill_second_after_certified_retirement(&self) {
        self.kill_next_descriptor("second", true);
    }

    pub(super) fn kill_distinct_release_before_checkpoint(&self) {
        self.kill_next_descriptor("distinct", false);
    }

    /// Parks the next one-record batch of the first object. No retirement and
    /// no checkpoint: the batch extends whatever the recovered world holds.
    pub(super) fn kill_successor_of_first_object(&self) {
        self.kill_next_descriptor("successor-first", false);
    }

    /// Parks the next batch of the first object at the whole manifest
    /// capacity. A remainder that fits leaves whole, which makes the batch
    /// terminal.
    pub(super) fn kill_full_batch_successor_of_first_object(&self) {
        self.kill_next_descriptor("successor-first-full", false);
    }

    /// Parks the next one-record batch of the one distinct object.
    pub(super) fn kill_successor_of_distinct_object(&self) {
        self.kill_next_descriptor("successor-distinct", false);
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
            .env(FIRST_WORLD_ENV, self.first.role())
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
    if let Some(first) = role.to_str().and_then(first::World::of_role) {
        return first::child(&marker, first);
    }
    match role.to_str() {
        Some("second") => second::child(
            Path::new(&std::env::var_os(ROOT_ENV).expect("second source root")),
            &marker,
        ),
        Some("distinct") => second::distinct_child(
            Path::new(&std::env::var_os(ROOT_ENV).expect("distinct source root")),
            &marker,
        ),
        Some("successor-first") => successor::child(
            Path::new(&std::env::var_os(ROOT_ENV).expect("successor source root")),
            &marker,
            successor::FIRST_OBJECT,
            1,
        ),
        Some("successor-first-full") => successor::child(
            Path::new(&std::env::var_os(ROOT_ENV).expect("successor source root")),
            &marker,
            successor::FIRST_OBJECT,
            successor::FULL_BATCH,
        ),
        Some("successor-distinct") => successor::child(
            Path::new(&std::env::var_os(ROOT_ENV).expect("successor source root")),
            &marker,
            successor::DISTINCT_OBJECT,
            1,
        ),
        other => panic!("unknown pending V3 child role: {other:?}"),
    }
}
