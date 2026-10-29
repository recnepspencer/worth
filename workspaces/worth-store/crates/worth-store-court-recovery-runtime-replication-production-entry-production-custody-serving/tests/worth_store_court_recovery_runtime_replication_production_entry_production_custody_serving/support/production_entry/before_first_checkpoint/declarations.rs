//! A store killed before its first checkpoint recovers from the
//! generation-zero basis: the whole WAL from the canonical origin.

pub(crate) use {std::fs, std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};

pub(crate) use {worth_store::physical_runtime::ExternalPhysicalRecordLocator, worth_store::physical_runtime::PhysicalCheckpointStartDenial, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff, worth_store_recovery_runtime::WorthStoreRecovery};

pub(crate) use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store_blob_chunks::BlobChunkSize, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, std::num::NonZeroU64, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving};



pub(crate) const ROLE_ENV: &str = "WORTH_GEN0_CHILD_ROLE";
pub(crate) const MARKER_ENV: &str = "WORTH_GEN0_CHILD_MARKER";
pub(crate) const CHILD_TEST: &str = "before_first_checkpoint::before_first_checkpoint_child";
pub(crate) const RECORD: &[u8] = b"written before the first checkpoint";
pub(crate) const CHUNK: usize = 64 << 10;
pub(crate) const SCOPE: &str = "c11.recovery.generation-zero.scope";

/// What the killed child wrote after initialization, with no checkpoint.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FirstWrite {
    Nothing,
    Record,
    TwoChunkBlob,
}

impl FirstWrite {
    pub(crate) const ALL: [Self; 3] = [Self::Nothing, Self::Record, Self::TwoChunkBlob];

    pub(crate) const fn role(self) -> &'static str {
        match self {
            Self::Nothing => "nothing",
            Self::Record => "record",
            Self::TwoChunkBlob => "two-chunk-blob",
        }
    }
}

/// The root a killed child left, and what it wrote there.
pub(crate) struct KilledWorld {
    pub(crate) root: PathBuf,
    pub(crate) write: FirstWrite,
    /// The record's external locator or the blob's object identity.
    pub(crate) written: Vec<u8>,
    pub(crate) _marker: tempfile::TempDir,
}

impl KilledWorld {
    pub(crate) fn launch(write: FirstWrite) -> Self {
        let marker_dir = tempfile::tempdir().unwrap();
        let marker = marker_dir.path().join("ready");
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
            .env(ROLE_ENV, write.role())
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
                    "generation-zero child did not park: {} {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        child.kill().unwrap();
        child.wait().unwrap();
        let root = std::path::PathBuf::from(std::fs::read_to_string(&marker).unwrap());
        assert!(
            !root.join("families/checkpoint.current").exists(),
            "the child must die before its first checkpoint"
        );
        Self {
            root,
            write,
            written: std::fs::read(marker.with_extension("written")).unwrap(),
            _marker: marker_dir,
        }
    }

    pub(crate) fn recover(&self, stage: &str) -> RecoveredPhysicalRuntimeHandoff {
        match worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(&self.root)) {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => *handoff,
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(block) => panic!(
                "{stage} blocked: kind={:?}; artifact={:?}; sources={:?}; cause={:?}; effects={}",
                block.cause(),
                block.evidence().artifact.as_deref(),
                block.evidence().source_denials,
                block.evidence().planning_denial,
                block.recovery_effects()
            ),
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
                "{stage} indeterminate: handoff={:?}; reopen={:?}",
                failure.handoff_failure(),
                failure.reopen_failure()
            ),
            _ => panic!("{stage} did not recover"),
        }
    }

    pub(crate) fn serve(&self, stage: &str) -> ServingPhysicalRuntime {
        let seal = self
            .recover(stage)
            .into_core()
            .certification_release_media_into_checkpoint_custody()
            .expect("generation-zero custody seal");
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(&self.root, seal)
    }

    /// Requires what the child wrote to read back byte-exact.
    pub(crate) fn assert_reads_back(&self, serving: &ServingPhysicalRuntime) {
        match self.write {
            crate::production_entry::before_first_checkpoint::FirstWrite::Nothing => assert!(self.written.is_empty()),
            crate::production_entry::before_first_checkpoint::FirstWrite::Record => {
                let locator =
                    worth_store::physical_runtime::ExternalPhysicalRecordLocator::decode(self.written.clone().try_into().unwrap())
                        .unwrap();
                let mut record = serving
                    .records()
                    .expect("read protection admission")
                    .open_external(
                        locator,
                        worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(1 << 16).unwrap()),
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
            crate::production_entry::before_first_checkpoint::FirstWrite::TwoChunkBlob => {
                let scope = admitted_blob_scope(SCOPE);
                let limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
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
pub(crate) fn assert_recovers_reads_back_and_reopens(write: FirstWrite, key: u8) {
    let world = crate::production_entry::before_first_checkpoint::KilledWorld::launch(write);
    let serving = world.serve("first recovery");
    world.assert_reads_back(&serving);
    serving.close();
    let serving = world.serve("second recovery before any checkpoint");
    world.assert_reads_back(&serving);
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let wrote = write != crate::production_entry::before_first_checkpoint::FirstWrite::Nothing;
    match serving.checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) if wrote => assert!(matches!(
            handle.wait(),
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        )),
        worth_proof::TransitionOutcome::Denied(worth_store::physical_runtime::PhysicalCheckpointStartDenial::NoDurableWalSource) if !wrote => {}
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
        .certification_release_media_into_checkpoint_custody()
        .expect("reopened custody seal");
    let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(&world.root, seal);
    world.assert_reads_back(&serving);
    serving.close();
}

pub(crate) fn blob_bytes() -> Vec<u8> {
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
pub(crate) fn child(marker: &Path, write: FirstWrite) -> ! {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("c11-generation-zero")
        .expect("generation-zero store");
    let written = match write {
        crate::production_entry::before_first_checkpoint::FirstWrite::Nothing => Vec::new(),
        crate::production_entry::before_first_checkpoint::FirstWrite::Record => write_record(&world),
        crate::production_entry::before_first_checkpoint::FirstWrite::TwoChunkBlob => publish_blob(&world),
    };
    std::fs::write(marker.with_extension("written"), written).unwrap();
    let pending = marker.with_extension("pending");
    std::fs::write(&pending, world.root().to_string_lossy().as_bytes()).unwrap();
    std::fs::rename(pending, marker).unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

pub(crate) fn write_record(world: &PhysicalResidencyStoreWorld) -> Vec<u8> {
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0xd0; 32]))
        .unwrap();
    let worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([RECORD]).unwrap(),
                world.placement(),
                worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                    key,
                    worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("the first record must prepare")
    };
    let worth_store::physical_runtime::PhysicalMutationOutcome::Completed(completed) = prepared.execute() else {
        panic!("the first record must complete")
    };
    let record = completed.persisted_records()[0];
    let mut locator = world.serving().store_identity().bytes().to_vec();
    locator.extend(record.allocation_epoch());
    locator.extend(record.ordinal().to_le_bytes());
    locator
}

pub(crate) fn publish_blob(world: &PhysicalResidencyStoreWorld) -> Vec<u8> {
    let scope = admitted_blob_scope(SCOPE);
    let blobs = world.serving().blobs().unwrap();
    let read = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read).unwrap();
    let bytes = blob_bytes();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        bytes.len() as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK as u64, read)
        .unwrap();
    for chunk in bytes.chunks(CHUNK) {
        ingest.push(chunk).unwrap();
    }
    match ingest.finish() {
        Ok(_) | Err(worth_store::physical_runtime::BlobIngestFailure::PublishedIndexPending { .. }) => {}
        Err(failure) => panic!("publication before the kill: {failure:?}"),
    }
    object.bytes().to_vec()
}