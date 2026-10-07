//! A same-object V3 successor survives a selected idle accumulator-only checkpoint.

use std::{
    fs,
    num::{NonZeroU16, NonZeroU64},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};
use worth_proof::{AdmittedBlobReleaseProof, TransitionOutcome};
use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, AdmittedPhysicalRecordFormat, BlobReclaimLimits,
    BlobReclaimRequest, ManifestEntryCapacity, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalMutationDeadline, PhysicalRecordFormatDeclaration, PhysicalRecordPlacementPolicy,
};
use worth_store_physical_format::{
    decode_blob_record, release_checkpoint_batch_records_digest_v1, BlobRecordV1,
    ReleasedDropCumulativeEvidenceV1,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

#[path = "accumulator_only_pending_wal/selected_media.rs"]
mod selected_media;
use selected_media::{selected_blob_control, selected_certificates, selected_released_source};

const CHILD_TEST: &str = "accumulator_only_pending_wal::successor_child";
const ROLE_ENV: &str = "WORTH_C11_ACCUMULATOR_SUCCESSOR_ROOT";
const MARKER_ENV: &str = "WORTH_C11_ACCUMULATOR_SUCCESSOR_MARKER";

#[test]
fn same_object_pending_v3_follows_idle_accumulator_tip() {
    let world = super::pending_wal_world::first();
    let root = world.root();
    let serving = recover_serving(root);
    serving
        .retire_displaced_segment()
        .expect("first recovered V3 extent retires through C1 checkpoint");
    let (first_batches, first_accumulator) = selected_certificates(root);
    assert_eq!(first_batches.len(), 1, "C1 folds the real first V3 WAL");
    assert_eq!(first_accumulator.base().batch_count(), 1);
    assert_eq!(
        first_accumulator.base().tip(),
        first_batches[0].tip_provenance().unwrap()
    );
    assert!(
        !first_accumulator.base().terminal(),
        "first drop remains nonterminal"
    );
    let first_heads =
        super::release_reopen::selected_head_oracle::selected_heads(root, first_accumulator);
    assert_eq!(first_heads.len(), 1);
    assert!(!first_heads[0].terminal());

    checkpoint(&serving, [0xc2; 32]);
    let (idle_batches, idle_accumulator) = selected_certificates(root);
    assert!(idle_batches.is_empty(), "C2 has no new released Batch");
    assert_eq!(idle_accumulator.base().batch_count(), 0);
    assert_eq!(
        idle_accumulator.base().tip(),
        first_accumulator.base().tip()
    );
    assert_eq!(
        idle_accumulator.base().cumulative_dropped(),
        first_accumulator.base().cumulative_dropped()
    );
    assert_eq!(
        idle_accumulator.base().cumulative_digest(),
        first_accumulator.base().cumulative_digest()
    );
    assert!(!idle_accumulator.base().terminal());
    assert_eq!(
        idle_accumulator.prior_head_count(),
        first_accumulator.head_count()
    );
    assert_eq!(
        idle_accumulator.prior_head_roster_digest(),
        first_accumulator.head_roster_digest()
    );
    let idle_heads =
        super::release_reopen::selected_head_oracle::selected_heads(root, idle_accumulator);
    assert_eq!(idle_heads, first_heads);
    serving.close();

    kill_successor_after_descriptor_wal(root);
    let serving = recover_serving(root);
    checkpoint(&serving, [0xc3; 32]);
    serving.close();
    let (successor_batches, successor_accumulator) = selected_certificates(root);
    assert_eq!(successor_batches.len(), 1, "C3 folds the pending V3");
    assert_eq!(successor_accumulator.base().batch_count(), 1);
    let predecessor = successor_batches[0]
        .predecessor()
        .expect("same-object successor names the first descriptor");
    assert_eq!(
        predecessor.descriptor_record(),
        idle_heads[0].descriptor_record()
    );
    assert_eq!(
        predecessor.descriptor_frame_sha256(),
        idle_heads[0].descriptor_frame_sha256()
    );
    assert_eq!(
        successor_accumulator.base().tip(),
        successor_batches[0].tip_provenance().unwrap()
    );
    assert!(
        successor_accumulator.base().cumulative_dropped()
            > idle_accumulator.base().cumulative_dropped()
    );
    assert_eq!(
        successor_accumulator.base().prior_cumulative_dropped(),
        idle_accumulator.base().cumulative_dropped()
    );
    assert_eq!(
        successor_accumulator.base().prior_cumulative_digest(),
        idle_accumulator.base().cumulative_digest()
    );
    assert_eq!(
        successor_accumulator.base().batch_records_digest(),
        release_checkpoint_batch_records_digest_v1(&successor_batches).unwrap()
    );
    let successor_heads =
        super::release_reopen::selected_head_oracle::selected_heads(root, successor_accumulator);
    assert_eq!(successor_heads.len(), 1);
    assert_eq!(successor_heads[0].key(), idle_heads[0].key());
    assert_eq!(
        successor_heads[0].descriptor_record(),
        successor_batches[0].descriptor_record()
    );
    assert!(successor_heads[0].cumulative_dropped() > idle_heads[0].cumulative_dropped());

    let fresh = recover_serving(root);
    let descriptor_bytes = selected_blob_control(&fresh, successor_batches[0].descriptor_record());
    let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
        decode_blob_record(&descriptor_bytes).unwrap()
    else {
        panic!("C3 selected descriptor must be the genuine V3 control")
    };
    let manifest_bytes = selected_blob_control(&fresh, descriptor.base().manifest_record());
    let BlobRecordV1::DropSetManifestV3(manifest) = decode_blob_record(&manifest_bytes).unwrap()
    else {
        panic!("C3 selected manifest must be the genuine V3 control")
    };
    let batch = successor_batches[0];
    assert_eq!(
        <[u8; 32]>::from(Sha256::digest(descriptor.encode())),
        batch.descriptor_frame_sha256()
    );
    assert_eq!(descriptor.custody_digest(), batch.custody_digest());
    assert_eq!(manifest.count(), descriptor.base().manifest_count());
    let evidence = ReleasedDropCumulativeEvidenceV1::new(
        batch.descriptor_record(),
        batch.descriptor_frame_sha256(),
        descriptor.custody_digest(),
        batch.reservation_record(),
        batch.reservation_frame_sha256(),
        batch.fate(),
        batch.candidate_root_generation(),
        batch.candidate_root_sha256(),
        batch.predecessor(),
        manifest.count(),
        descriptor.base().terminal(),
    )
    .unwrap();
    let (expected_count, expected_digest) = evidence
        .advance(
            idle_accumulator.base().cumulative_dropped(),
            idle_accumulator.base().cumulative_digest(),
        )
        .unwrap();
    assert_eq!(batch.cumulative_dropped(), expected_count);
    assert_eq!(batch.cumulative_digest(), expected_digest);
    assert_eq!(
        successor_accumulator.base().cumulative_dropped(),
        expected_count
    );
    assert_eq!(
        successor_accumulator.base().cumulative_digest(),
        expected_digest
    );
    fresh.close();
}

fn recover_serving(root: &Path) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let outcome = WorthStoreRecovery::recover(super::certified_release_serving::request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("C8 must recover genuine accumulator continuation: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("C8 selected custody seal");
    super::certified_release_serving::admit_serving_with_seal(root, seal)
}

fn checkpoint(serving: &worth_store::physical_runtime::ServingPhysicalRuntime, key: [u8; 32]) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("release checkpoint must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "release checkpoint must complete: {outcome:?}"
    );
}

fn kill_successor_after_descriptor_wal(root: &Path) {
    let markers = tempfile::tempdir().unwrap();
    let marker = markers.path().join("descriptor-durable");
    let checkpoint_before = fs::read(root.join("families/checkpoint.current")).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, root)
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
                "same-object descriptor seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    assert_eq!(fs::read(&marker).unwrap(), b"successor-descriptor-wal");
    assert_eq!(
        fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint_before,
        "successor remains pending beyond C2"
    );
}

#[test]
fn successor_child() {
    let Some(root) = std::env::var_os(ROLE_ENV) else {
        return;
    };
    let marker = PathBuf::from(std::env::var_os(MARKER_ENV).expect("successor marker"));
    let serving = recover_serving(Path::new(&root));
    let source = selected_released_source(&serving);
    let proof = AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        source.object(),
        source.generation(),
        source.publication_record().allocation_epoch(),
        source.publication_record().ordinal(),
        source.publication_frame_sha256(),
        source.issuer_evidence_sha256(),
    )
    .unwrap();
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        ))
        .unwrap();
    let request = BlobReclaimRequest::released(
        proof,
        placement,
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(128).unwrap(),
            NonZeroU64::new(32 << 20).unwrap(),
            NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    park_at_descriptor_wal(&serving, request, &marker);
}

fn park_at_descriptor_wal(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: BlobReclaimRequest,
    marker: &Path,
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
            fs::write(&pending, b"successor-descriptor-wal").unwrap();
            fs::rename(pending, marker).unwrap();
        });
        let result = serving
            .blobs()
            .unwrap()
            .reclaim(request)
            .and_then(|handle| handle.wait());
        cancelled.store(true, Ordering::SeqCst);
        panic!("successor descriptor ended before durable WAL pause: {result:?}");
    });
}
