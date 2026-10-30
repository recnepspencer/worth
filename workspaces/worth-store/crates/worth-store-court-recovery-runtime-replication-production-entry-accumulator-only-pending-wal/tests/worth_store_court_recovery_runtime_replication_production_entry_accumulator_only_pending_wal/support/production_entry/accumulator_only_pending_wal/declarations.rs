//! A same-object V3 successor survives a selected idle accumulator-only checkpoint.

pub(crate) use {
    std::fs, std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, std::path::PathBuf,
    std::process::Command, std::process::Stdio, std::sync::atomic::AtomicBool,
    std::sync::atomic::Ordering, std::thread, std::time::Duration, std::time::Instant,
};

pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome};
pub(crate) use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record,
    worth_store_physical_format::release_checkpoint_batch_records_digest_v1,
    worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::ReleasedDropCumulativeEvidenceV1,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub(crate) use {
    crate::production_entry::accumulator_only_pending_wal::selected_media::selected_blob_control,
    crate::production_entry::accumulator_only_pending_wal::selected_media::selected_certificates,
    crate::production_entry::accumulator_only_pending_wal::selected_media::selected_released_source,
};

pub(crate) const CHILD_TEST: &str =
    "process::recovery_runtime::production_entry::accumulator_only_pending_wal::successor_child";
pub(crate) const ROLE_ENV: &str = "WORTH_C11_ACCUMULATOR_SUCCESSOR_ROOT";
pub(crate) const MARKER_ENV: &str = "WORTH_C11_ACCUMULATOR_SUCCESSOR_MARKER";

pub(crate) fn recover_serving(
    root: &Path,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("C8 must recover genuine accumulator continuation: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("C8 selected custody seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(root, seal)
}

pub(crate) fn checkpoint(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    key: [u8; 32],
) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("release checkpoint must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(
            outcome,
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
        "release checkpoint must complete: {outcome:?}"
    );
}

pub(crate) fn kill_successor_after_descriptor_wal(root: &Path) {
    let markers = tempfile::tempdir().unwrap();
    let marker = markers.path().join("descriptor-durable");
    let checkpoint_before = std::fs::read(root.join("families/checkpoint.current")).unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, root)
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
                "same-object descriptor seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    assert_eq!(std::fs::read(&marker).unwrap(), b"successor-descriptor-wal");
    assert_eq!(
        std::fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint_before,
        "successor remains pending beyond C2"
    );
}

pub(crate) fn park_at_descriptor_wal(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: BlobReclaimRequest,
    marker: &Path,
) {
    let first = serving.pause_physical_mutation_at(
        worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability,
    );
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
            std::fs::write(&pending, b"successor-descriptor-wal").unwrap();
            std::fs::rename(pending, marker).unwrap();
        });
        let result = serving
            .blobs()
            .unwrap()
            .reclaim(request)
            .and_then(|handle| handle.wait());
        cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
        panic!("successor descriptor ended before durable WAL pause: {result:?}");
    });
}
