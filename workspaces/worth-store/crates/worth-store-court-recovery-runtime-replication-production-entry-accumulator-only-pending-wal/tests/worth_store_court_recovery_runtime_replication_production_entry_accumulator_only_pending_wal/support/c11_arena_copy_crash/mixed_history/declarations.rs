pub(crate) use {std::process::Command, std::time::Duration, worth_store_testkit_recovery_runtime::support::EXTENT_BYTES, std::time::Instant, std::path::Path, std::path::PathBuf, worth_store_physical_format::PersistedRecordIdentity, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff, worth_store::physical_runtime::ServingPhysicalRuntime, std::process::Stdio, worth_store_testkit_recovery_runtime::support::configuration, worth_store_testkit_recovery_runtime::support::generation, crate::c11_arena_copy_crash::kill_after_final_wal, crate::c11_arena_copy_crash::load_identity, worth_store_testkit_recovery_runtime::support::observe_arena_in_separate_process, crate::c11_arena_copy_crash::observation::observe_copy_media, worth_store_testkit_recovery_runtime::support::open, worth_store_testkit_recovery_runtime::support::read_record, crate::c11_arena_copy_crash::recover, worth_store_testkit_recovery_runtime::support::scan, crate::c11_arena_copy_crash::selected_extent, std::thread};
pub(crate) use {std::num::NonZeroU64};
pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReadOpenFailure, worth_store::physical_runtime::LayoutRebuildLimits, worth_store::physical_runtime::PhysicalLayoutDenial, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_contracts::DurableArtifactFamilyId};


pub(crate) use {crate::copy_frame};

pub(crate) const BLOB_CHILD: &str = "mixed_history::blob_child";
pub(crate) const BLOB_MARKER: &str = "mixed-current-copy-blob-ready";
pub(crate) const CHUNK_BYTES: usize = 64 * 1024;
pub(crate) const SOURCE_WINDOW_BYTES: u64 = 32 * 1024;

pub(crate) fn kill_after_blob_generation_wal(directory: &Path) -> ([u8; 16], [u8; 16]) {
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", BLOB_CHILD, "--ignored", "--nocapture"])
        .env(BLOB_MARKER, directory)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let marker = directory.join(BLOB_MARKER);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while !marker.is_file() {
        if child.try_wait().unwrap().is_some() || std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "generation seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let identity = std::fs::read(marker).unwrap();
    assert_eq!(identity.len(), 32);
    child.kill().unwrap();
    child.wait().unwrap();
    (
        identity[..16].try_into().unwrap(),
        identity[16..32].try_into().unwrap(),
    )
}

pub(crate) fn selected_wal_contains(handoff: &RecoveredPhysicalRuntimeHandoff, lsn: u64) -> bool {
    handoff
        .selected_sources()
        .wal_tail()
        .frame_facts()
        .any(|frame| {
            frame.lsn_range().start().get() <= lsn && lsn < frame.lsn_range().end_exclusive().get()
        })
}

pub(crate) fn read_copied_record(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    record: PersistedRecordIdentity,
) -> Vec<u8> {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(1).unwrap()))
        .unwrap();
    let mut scratch = vec![0; 128_000];
    let mut selected = None;
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for row in batch.records() {
            let id = row.record_id();
            if id.allocation_epoch() == record.allocation_epoch()
                && id.ordinal() == record.ordinal()
            {
                assert!(selected.replace(id).is_none(), "copied record appears once");
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    drop(scan);
    read_record(serving, selected.expect("copied record remains readable"))
}