//! A selected Tier checkpoint and process-killed V3 drop share one recovery suffix.

use super::*;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};
use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, BlobReclaimLimits, RecordByteLimit, RecordCountLimit,
    RecordScanOutcome, RecordScanRequest,
};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimSourceBasisV1, BlobRecordKind, BlobRecordV1,
    SelectedRecordContentClass,
};

#[path = "tier_release_pending_wal/checkpoint.rs"]
mod checkpoint;
use checkpoint::*;

const ROLE_ENV: &str = "WORTH_C11_TIER_RELEASE_PENDING_CHILD";
const MARKER_ENV: &str = "WORTH_C11_TIER_RELEASE_PENDING_MARKER";
const CHILD_TEST: &str = "tier_release_pending_wal::mixed_tier_pending_child";
const BASELINE: &[u8] = b"c11-tier-release-surviving-record";
const CHUNK: usize = 64 << 10;

#[test]
fn tier_checkpoint_then_killed_released_wal_reopens_once() {
    thread::Builder::new()
        .name("mixed-tier-release-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(run_tier_checkpoint_then_killed_released_wal)
        .unwrap()
        .join()
        .expect("mixed Tier/V3 recovery worker");
}

fn run_tier_checkpoint_then_killed_released_wal() {
    let world = kill_producer_after_descriptor_wal();
    let checkpoint = selected_checkpoint(world.root());
    assert_tier_and_no_release(&checkpoint);

    let first = recover_and_open(world.root());
    assert!(selected_checkpoint(world.root()) == checkpoint);
    assert_selected_result(&first);
    let before = first.certification_charged_growth_bytes();
    first
        .retire_displaced_segment()
        .expect("verified released drop must owe one native extent retirement");
    assert!(first.certification_charged_growth_bytes() < before);
    assert_eq!(
        first.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Absent),
        "one released drop must not retire twice"
    );
    let retired_checkpoint = selected_checkpoint(world.root());
    assert!(
        retired_checkpoint != checkpoint,
        "retirement folds a successor checkpoint"
    );
    assert_tier_and_batch(&retired_checkpoint);
    first.close();

    let repeated = recover_and_open(world.root());
    assert!(selected_checkpoint(world.root()) == retired_checkpoint);
    assert_selected_result(&repeated);
    assert_eq!(
        repeated.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Absent),
        "historical WAL must not recreate a retired native obligation"
    );
    repeated.close();
}

#[test]
fn mixed_tier_pending_child() {
    let Some(marker) = std::env::var_os(MARKER_ENV) else {
        return;
    };
    assert_eq!(std::env::var(ROLE_ENV).as_deref(), Ok("produce"));
    produce_pending_world(Path::new(&marker));
}

fn produce_pending_world(marker: &Path) {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("c11-tier-pending-v3")
        .expect("mixed Tier/V3 producer Store");
    canonical_physical_mutation_acknowledgment(&world, [0xc1; 32], BASELINE);
    checkpoint_world(&world, 0xc2);
    world
        .serving()
        .certification_activate_tier_epoch(world.placement())
        .expect("real tier activation");
    checkpoint_world(&world, 0xc3);
    assert_tier_and_no_release(&selected_checkpoint(world.root()));

    let scope = admitted_blob_scope("c11.recovery.tier-pending-v3.scope");
    let blobs = world.serving().blobs().unwrap();
    let read = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK as u64, read)
        .unwrap();
    ingest.push(&vec![0x37; CHUNK]).unwrap();
    ingest.push(&vec![0x48; CHUNK]).unwrap();
    let published = match ingest.finish() {
        Ok(published) | Err(BlobIngestFailure::PublishedIndexPending { published, .. }) => {
            published
        }
        Err(failure) => panic!("real blob publication before crash: {failure:?}"),
    };
    drop(blobs);
    let publication = world
        .serving()
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("selected publication for released proof");
    assert_eq!(published.object(), object);
    let proof = AdmittedBlobReleaseProof::certification_admit(
        world.serving().store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        publication.record().allocation_epoch(),
        publication.record().ordinal(),
        publication.encoded_digest(),
        [0xc4; 32],
    )
    .unwrap();
    let request = BlobReclaimRequest::released(
        proof,
        world.placement(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(128).unwrap(),
            NonZeroU64::new(32 << 20).unwrap(),
            NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    park_at_descriptor_wal(world.serving(), request, marker, world.root());
}

fn checkpoint_world(world: &PhysicalResidencyStoreWorld, key: u8) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("source checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}

fn park_at_descriptor_wal(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: BlobReclaimRequest,
    marker: &Path,
    root: &Path,
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
            fs::write(&pending, root.to_string_lossy().as_bytes()).unwrap();
            fs::rename(pending, marker).unwrap();
        });
        let outcome = serving
            .blobs()
            .unwrap()
            .reclaim(request)
            .and_then(|handle| handle.wait());
        cancelled.store(true, Ordering::SeqCst);
        panic!("released descriptor finished before OS kill: {outcome:?}");
    });
}

pub(super) struct KilledWorld {
    root: PathBuf,
    _marker: tempfile::TempDir,
}

impl KilledWorld {
    pub(super) fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for KilledWorld {
    fn drop(&mut self) {
        let root = self.root.canonicalize().expect("killed Store root remains");
        assert!(root.starts_with(std::env::temp_dir().canonicalize().unwrap()));
        assert!(root
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("worth-store-c11-tier-pending-v3-"));
        fs::remove_dir_all(root).expect("remove exact child-owned Store root");
    }
}

pub(super) fn kill_producer_after_descriptor_wal() -> KilledWorld {
    let marker_dir = tempfile::tempdir().unwrap();
    let marker = marker_dir.path().join("ready");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, "produce")
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
                "mixed Tier/V3 descriptor WAL seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let root = PathBuf::from(fs::read_to_string(marker).unwrap());
    assert!(root.is_dir());
    KilledWorld {
        root,
        _marker: marker_dir,
    }
}

fn recover_and_open(root: &Path) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        super::certified_release_serving::request(root),
    );
    let handoff = match outcome {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(block) => panic!(
            "mixed Tier/V3 C8 blocked at {:?}: {:?}",
            block.cause(),
            block.evidence().planning_denial
        ),
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
            indeterminate,
        ) => panic!(
            "mixed Tier/V3 Store rejoin indeterminate: handoff={:?} reopen={:?}",
            indeterminate.handoff_failure(),
            indeterminate.reopen_failure()
        ),
        other => panic!("mixed Tier/V3 C8 refused: {other:?}"),
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("Store independently sealed mixed Tier/V3 custody");
    super::certified_release_serving::admit_serving_with_seal(root, seal)
}

fn assert_selected_result(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(8).unwrap())
                .with_payload_limit(RecordByteLimit::new(512 << 10).unwrap()),
        )
        .unwrap();
    let mut scratch = vec![0; 512 << 10];
    let (mut baseline, mut surviving_blob_payload, mut descriptor) = (0, 0, 0);
    let mut selected = BTreeSet::new();
    let mut dropped = None;
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for index in 0..batch.records().len() {
            let record = batch.records()[index].record_id();
            selected.insert(
                worth_store_physical_format::PersistedRecordIdentity::new(
                    record.allocation_epoch(),
                    record.ordinal(),
                )
                .unwrap(),
            );
            let Some(payload) = batch.payload(index) else {
                assert!(!matches!(
                    batch.records()[index].content_class(),
                    SelectedRecordContentClass::Blob(
                        BlobRecordKind::DropSetManifestV3 | BlobRecordKind::ReclaimDescriptorV3
                    )
                ));
                continue;
            };
            baseline += usize::from(payload == BASELINE);
            match decode_blob_record(payload) {
                Ok(BlobRecordV1::Chunk(_) | BlobRecordV1::TreeNode(_)) => {
                    surviving_blob_payload += 1;
                }
                Ok(BlobRecordV1::DropSetManifestV3(manifest)) => {
                    assert!(matches!(
                        manifest.source_basis(),
                        BlobReclaimSourceBasisV1::ReleasedGeneration(_)
                    ));
                    assert!(dropped.replace(manifest.dropped().to_vec()).is_none());
                }
                Ok(BlobRecordV1::ReclaimDescriptorV3(_)) => descriptor += 1,
                _ => {}
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    assert_eq!(baseline, 1, "Tier source root retains its ordinary record");
    assert!(
        surviving_blob_payload > 0,
        "first-batch release must preserve readable, undropped blob payload"
    );
    let dropped = dropped.expect("one selected released-drop manifest");
    assert!(!dropped.is_empty());
    assert!(
        dropped.iter().all(|record| !selected.contains(record)),
        "each manifest-named dropped record must be absent from the selected root"
    );
    assert_eq!(descriptor, 1, "one V3 drop result remains selected");
}
