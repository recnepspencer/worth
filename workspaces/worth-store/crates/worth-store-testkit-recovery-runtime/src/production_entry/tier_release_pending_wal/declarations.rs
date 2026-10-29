//! A selected Tier checkpoint and process-killed V3 drop share one recovery suffix.

pub use {worth_proof::AdmittedBlobReleaseProof, worth_store::physical_runtime::BlobCheckpointLimit, worth_store_blob_chunks::BlobChunkSize, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimRequest, std::num::NonZeroU16, std::num::NonZeroU64, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, crate::production_entry::blob_scope_admission::admitted_blob_scope, worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, crate::production_entry::certified_release_serving};
pub use {std::collections::BTreeSet, std::fs, std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::sync::atomic::AtomicBool, std::sync::atomic::Ordering, std::thread, std::time::Duration, std::time::Instant};
pub use {worth_store::physical_runtime::production::PhysicalMutationCheckpoint, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest};
pub use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobReclaimSourceBasisV1, worth_store_physical_format::BlobRecordKind, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::SelectedRecordContentClass};


pub use {worth_proof::AdmittedBlobReleaseProof, worth_store::physical_runtime::BlobCheckpointLimit, worth_store_blob_chunks::BlobChunkSize, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimRequest, std::num::NonZeroU16, std::num::NonZeroU64, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, crate::production_entry::blob_scope_admission::admitted_blob_scope, crate::production_entry::tier_release_pending_wal::checkpoint::assert_tier_and_batch, crate::production_entry::tier_release_pending_wal::checkpoint::assert_tier_and_no_release, worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, crate::production_entry::certified_release_serving, crate::production_entry::tier_release_pending_wal::checkpoint::selected_checkpoint};

pub const ROLE_ENV: &str = "WORTH_C11_TIER_RELEASE_PENDING_CHILD";
pub const MARKER_ENV: &str = "WORTH_C11_TIER_RELEASE_PENDING_MARKER";
pub const CHILD_TEST: &str = "tier_release_pending_wal::mixed_tier_pending_child";
pub const BASELINE: &[u8] = b"c11-tier-release-surviving-record";
pub const CHUNK: usize = 64 << 10;

pub fn run_tier_checkpoint_then_killed_released_wal() {
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

pub fn produce_pending_world(marker: &Path) {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("c11-tier-pending-v3")
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
    let read = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK as u64, read)
        .unwrap();
    ingest.push(&vec![0x37; CHUNK]).unwrap();
    ingest.push(&vec![0x48; CHUNK]).unwrap();
    let published = match ingest.finish() {
        Ok(published) | Err(worth_store::physical_runtime::BlobIngestFailure::PublishedIndexPending { published, .. }) => {
            published
        }
        Err(failure) => panic!("real blob publication before crash: {failure:?}"),
    };
    let publication = world
        .serving()
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("selected publication for released proof");
    assert_eq!(published.object(), object);
    let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
        world.serving().store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        publication.record().allocation_epoch(),
        publication.record().ordinal(),
        publication.encoded_digest(),
        [0xc4; 32],
    )
    .unwrap();
    let request = worth_store::physical_runtime::BlobReclaimRequest::released(
        proof,
        world.placement(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(128).unwrap(),
            std::num::NonZeroU64::new(32 << 20).unwrap(),
            std::num::NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    park_at_descriptor_wal(world.serving(), request, marker, world.root());
}

pub fn checkpoint_world(world: &PhysicalResidencyStoreWorld, key: u8) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("source checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
}

pub fn park_at_descriptor_wal(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: BlobReclaimRequest,
    marker: &Path,
    root: &Path,
) {
    let first = serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability);
    let cancelled = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
            while !first.await_arrival() {
                if cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                    return;
                }
                assert!(std::time::Instant::now() < deadline, "manifest WAL durability");
            }
            let second =
                serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability);
            first.release();
            while !second.await_arrival() {
                if cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                    return;
                }
                assert!(std::time::Instant::now() < deadline, "reservation WAL durability");
            }
            let third =
                serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability);
            second.release();
            while !third.await_arrival() {
                if cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                    return;
                }
                assert!(std::time::Instant::now() < deadline, "descriptor WAL durability");
            }
            let pending = marker.with_extension("pending");
            std::fs::write(&pending, root.to_string_lossy().as_bytes()).unwrap();
            std::fs::rename(pending, marker).unwrap();
        });
        let outcome = serving
            .blobs()
            .unwrap()
            .reclaim(request)
            .and_then(|handle| handle.wait());
        cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
        panic!("released descriptor finished before OS kill: {outcome:?}");
    });
}

pub struct KilledWorld {
    pub root: PathBuf,
    pub _marker: tempfile::TempDir,
}

impl KilledWorld {
    pub fn root(&self) -> &Path {
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
        std::fs::remove_dir_all(root).expect("remove exact child-owned Store root");
    }
}

pub fn kill_producer_after_descriptor_wal() -> KilledWorld {
    let marker_dir = tempfile::tempdir().unwrap();
    let marker = marker_dir.path().join("ready");
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, "produce")
        .env(MARKER_ENV, &marker)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(240);
    while !marker.is_file() {
        if child.try_wait().unwrap().is_some() || std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "mixed Tier/V3 descriptor WAL seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let root = std::path::PathBuf::from(std::fs::read_to_string(marker).unwrap());
    assert!(root.is_dir());
    KilledWorld {
        root,
        _marker: marker_dir,
    }
}

pub fn recover_and_open(root: &Path) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        crate::production_entry::certified_release_serving::recovery_request::request(root),
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
        .certification_release_media_into_checkpoint_custody()
        .expect("Store independently sealed mixed Tier/V3 custody");
    crate::production_entry::certified_release_serving::admit_serving_with_seal(root, seal)
}

pub fn assert_selected_result(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(8).unwrap())
                .with_payload_limit(worth_store::physical_runtime::RecordByteLimit::new(512 << 10).unwrap()),
        )
        .unwrap();
    let mut scratch = vec![0; 512 << 10];
    let (mut baseline, mut surviving_blob_payload, mut descriptor) = (0, 0, 0);
    let mut selected = std::collections::BTreeSet::new();
    let mut dropped = None;
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
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
                    worth_store_physical_format::SelectedRecordContentClass::Blob(
                        worth_store_physical_format::BlobRecordKind::DropSetManifestV3 | worth_store_physical_format::BlobRecordKind::ReclaimDescriptorV3
                    )
                ));
                continue;
            };
            baseline += usize::from(payload == BASELINE);
            match decode_blob_record(payload) {
                Ok(worth_store_physical_format::BlobRecordV1::Chunk(_) | worth_store_physical_format::BlobRecordV1::TreeNode(_)) => {
                    surviving_blob_payload += 1;
                }
                Ok(worth_store_physical_format::BlobRecordV1::DropSetManifestV3(manifest)) => {
                    assert!(matches!(
                        manifest.source_basis(),
                        worth_store_physical_format::BlobReclaimSourceBasisV1::ReleasedGeneration(_)
                    ));
                    assert!(dropped.replace(manifest.dropped().to_vec()).is_none());
                }
                Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptorV3(_)) => descriptor += 1,
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