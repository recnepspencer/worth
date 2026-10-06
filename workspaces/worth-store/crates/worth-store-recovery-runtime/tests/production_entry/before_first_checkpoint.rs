//! A store killed before its first checkpoint recovers from the
//! generation-zero basis: the whole WAL from the canonical origin.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use worth_store::physical_runtime::{
    ExternalPhysicalRecordLocator, PhysicalCheckpointStartDenial,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, RecordAppendBatch,
    RecordByteLimit, RecordReadLimits, ServingPhysicalRuntime,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryOutcome, RecoveredPhysicalRuntimeHandoff, WorthStoreRecovery,
};

use super::*;

#[path = "before_first_checkpoint/still_denied.rs"]
mod still_denied;

const ROLE_ENV: &str = "WORTH_GEN0_CHILD_ROLE";
const MARKER_ENV: &str = "WORTH_GEN0_CHILD_MARKER";
const CHILD_TEST: &str = "before_first_checkpoint::before_first_checkpoint_child";
const RECORD: &[u8] = b"written before the first checkpoint";
const CHUNK: usize = 64 << 10;
const SCOPE: &str = "c11.recovery.generation-zero.scope";

/// What the killed child wrote after initialization, with no checkpoint.
#[derive(Clone, Copy, PartialEq, Eq)]
enum FirstWrite {
    Nothing,
    Record,
    TwoChunkBlob,
}

impl FirstWrite {
    const ALL: [Self; 3] = [Self::Nothing, Self::Record, Self::TwoChunkBlob];

    const fn role(self) -> &'static str {
        match self {
            Self::Nothing => "nothing",
            Self::Record => "record",
            Self::TwoChunkBlob => "two-chunk-blob",
        }
    }
}

/// The root a killed child left, and what it wrote there.
struct KilledWorld {
    root: PathBuf,
    write: FirstWrite,
    /// The record's external locator or the blob's object identity.
    written: Vec<u8>,
    _marker: tempfile::TempDir,
}

impl KilledWorld {
    fn launch(write: FirstWrite) -> Self {
        let marker_dir = tempfile::tempdir().unwrap();
        let marker = marker_dir.path().join("ready");
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
            .env(ROLE_ENV, write.role())
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
                    "generation-zero child did not park: {} {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            thread::sleep(Duration::from_millis(10));
        }
        child.kill().unwrap();
        child.wait().unwrap();
        let root = PathBuf::from(fs::read_to_string(&marker).unwrap());
        assert!(
            !root.join("families/checkpoint.current").exists(),
            "the child must die before its first checkpoint"
        );
        Self {
            root,
            write,
            written: fs::read(marker.with_extension("written")).unwrap(),
            _marker: marker_dir,
        }
    }

    fn recover(&self, stage: &str) -> RecoveredPhysicalRuntimeHandoff {
        match WorthStoreRecovery::recover(certified_release_serving::request(&self.root)) {
            PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
            PhysicalRecoveryOutcome::Blocked(block) => panic!(
                "{stage} blocked: kind={:?}; artifact={:?}; sources={:?}; cause={:?}; effects={}",
                block.cause(),
                block.evidence().artifact.as_deref(),
                block.evidence().source_denials,
                block.evidence().planning_denial,
                block.recovery_effects()
            ),
            PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
                "{stage} indeterminate: handoff={:?}; reopen={:?}",
                failure.handoff_failure(),
                failure.reopen_failure()
            ),
            _ => panic!("{stage} did not recover"),
        }
    }

    fn serve(&self, stage: &str) -> ServingPhysicalRuntime {
        let seal = self
            .recover(stage)
            .into_core()
            .into_checkpoint_custody()
            .expect("generation-zero custody seal");
        certified_release_serving::admit_serving_with_seal(&self.root, seal)
    }

    /// Requires what the child wrote to read back byte-exact.
    fn assert_reads_back(&self, serving: &ServingPhysicalRuntime) {
        match self.write {
            FirstWrite::Nothing => assert!(self.written.is_empty()),
            FirstWrite::Record => {
                let locator =
                    ExternalPhysicalRecordLocator::decode(self.written.clone().try_into().unwrap())
                        .unwrap();
                let mut record = serving
                    .records()
                    .expect("read protection admission")
                    .open_external(
                        locator,
                        RecordReadLimits::new(RecordByteLimit::new(1 << 16).unwrap()),
                    )
                    .unwrap();
                let mut observed = vec![0_u8; RECORD.len() + 1];
                let mut used = 0;
                loop {
                    let count = record.read_next(&mut observed[used..]).unwrap();
                    if count == 0 {
                        break;
                    }
                    used += count;
                }
                assert_eq!(&observed[..used], RECORD);
            }
            FirstWrite::TwoChunkBlob => {
                let scope = admitted_blob_scope(SCOPE);
                let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
                let blobs = serving.blobs().unwrap();
                let object = self.written.clone().try_into().unwrap();
                let resolved = blobs
                    .resolve_publication(object, 1, &scope, limits)
                    .unwrap();
                let expected = blob_bytes();
                let mut read = blobs
                    .read(resolved, &scope, 0, expected.len() as u64, limits)
                    .unwrap();
                let mut observed = vec![0_u8; expected.len()];
                let mut used = 0;
                while used < observed.len() {
                    let count = read.read_next(&mut observed[used..]).unwrap();
                    assert!(count > 0, "the blob ended at byte {used}");
                    used += count;
                }
                assert!(observed == expected, "the blob read back changed");
            }
        }
    }
}

/// Recovers in this fresh process and reads back, recovers again with no
/// checkpoint between, then checkpoints and requires the next recovery to
/// find nothing left to do. With nothing written there is no durable WAL to
/// checkpoint, exactly as for a store that was never killed.
fn assert_recovers_reads_back_and_reopens(write: FirstWrite, key: u8) {
    let world = KilledWorld::launch(write);
    let serving = world.serve("first recovery");
    world.assert_reads_back(&serving);
    serving.close();
    let serving = world.serve("second recovery before any checkpoint");
    world.assert_reads_back(&serving);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let wrote = write != FirstWrite::Nothing;
    match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) if wrote => assert!(matches!(
            handle.wait(),
            PhysicalCheckpointOutcome::Completed(_)
        )),
        TransitionOutcome::Denied(PhysicalCheckpointStartDenial::NoDurableWalSource) if !wrote => {}
        _ => panic!("the first checkpoint over the recovered store must admit"),
    }
    serving.close();
    assert_eq!(
        world.root.join("families/checkpoint.current").is_file(),
        wrote
    );
    let reopened = world.recover("reopen after the first checkpoint");
    assert_eq!(reopened.core().recovery_effect_count(), 0);
    let seal = reopened
        .into_core()
        .into_checkpoint_custody()
        .expect("reopened custody seal");
    let serving = certified_release_serving::admit_serving_with_seal(&world.root, seal);
    world.assert_reads_back(&serving);
    serving.close();
}

#[test]
fn a_store_killed_after_initialization_reopens_clean() {
    assert_recovers_reads_back_and_reopens(FirstWrite::Nothing, 0xd1);
}

#[test]
fn a_record_written_before_the_first_checkpoint_recovers_byte_exact() {
    assert_recovers_reads_back_and_reopens(FirstWrite::Record, 0xd2);
}

#[test]
fn a_two_chunk_blob_published_before_the_first_checkpoint_recovers_byte_exact() {
    assert_recovers_reads_back_and_reopens(FirstWrite::TwoChunkBlob, 0xd3);
}

#[test]
fn a_checkpoint_appearing_before_construction_rereads_denies_the_generation_zero_basis() {
    let world = KilledWorld::launch(FirstWrite::Record);
    let current = world.root.join("families/checkpoint.current");
    let outcome = WorthStoreRecovery::certification_recover_with_custody_pauses(
        certified_release_serving::request(&world.root),
        |_| {},
        move || fs::write(current, []).unwrap(),
    );
    assert!(
        !matches!(outcome, PhysicalRecoveryOutcome::Recovered(_)),
        "construction must re-read the absent checkpoint: {outcome:?}"
    );
}

fn blob_bytes() -> Vec<u8> {
    (0..2_u8)
        .flat_map(|ordinal| {
            let mut chunk = vec![0x5a ^ ordinal; CHUNK];
            chunk[0] = ordinal;
            chunk
        })
        .collect()
}

/// Initializes a store, writes `write` with no checkpoint, announces the
/// root and idles until the parent kills it.
fn child(marker: &Path, write: FirstWrite) -> ! {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("c11-generation-zero")
        .expect("generation-zero store");
    let written = match write {
        FirstWrite::Nothing => Vec::new(),
        FirstWrite::Record => write_record(&world),
        FirstWrite::TwoChunkBlob => publish_blob(&world),
    };
    fs::write(marker.with_extension("written"), written).unwrap();
    let pending = marker.with_extension("pending");
    fs::write(&pending, world.root().to_string_lossy().as_bytes()).unwrap();
    fs::rename(pending, marker).unwrap();
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn write_record(world: &PhysicalResidencyStoreWorld) -> Vec<u8> {
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0xd0; 32]))
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([RECORD]).unwrap(),
                world.placement(),
                PhysicalMutationRequest::platform_durable(
                    key,
                    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("the first record must prepare")
    };
    let PhysicalMutationOutcome::Completed(completed) = prepared.execute() else {
        panic!("the first record must complete")
    };
    let record = completed.persisted_records()[0];
    let mut locator = world.serving().store_identity().bytes().to_vec();
    locator.extend(record.allocation_epoch());
    locator.extend(record.ordinal().to_le_bytes());
    locator
}

fn publish_blob(world: &PhysicalResidencyStoreWorld) -> Vec<u8> {
    let scope = admitted_blob_scope(SCOPE);
    let blobs = world.serving().blobs().unwrap();
    let read = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read).unwrap();
    let bytes = blob_bytes();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        bytes.len() as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK as u64, read)
        .unwrap();
    for chunk in bytes.chunks(CHUNK) {
        ingest.push(chunk).unwrap();
    }
    match ingest.finish() {
        Ok(_) | Err(BlobIngestFailure::PublishedIndexPending { .. }) => {}
        Err(failure) => panic!("publication before the kill: {failure:?}"),
    }
    object.bytes().to_vec()
}

#[test]
fn before_first_checkpoint_child() {
    let Some(role) = std::env::var_os(ROLE_ENV) else {
        return;
    };
    let marker = PathBuf::from(std::env::var_os(MARKER_ENV).expect("child marker"));
    let write = FirstWrite::ALL
        .into_iter()
        .find(|write| Some(write.role()) == role.to_str())
        .expect("generation-zero child role");
    child(&marker, write);
}
