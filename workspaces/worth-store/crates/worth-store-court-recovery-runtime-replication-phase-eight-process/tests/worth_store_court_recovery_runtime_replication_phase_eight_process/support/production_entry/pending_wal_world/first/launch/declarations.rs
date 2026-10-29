//! The parent side of the first world: start the child, wait for it to park,
//! kill it and adopt the root it left.

pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::CHILD_TEST, std::cell::Cell, std::process::Command, std::time::Duration, std::time::Instant, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::MARKER_ENV, std::num::NonZeroU64, std::path::PathBuf, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::PendingWalWorld, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::ROLE_ENV, std::process::Stdio, crate::production_entry::pending_wal_world::first::Tail, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::WAL_SEGMENT_BYTES_ENV, crate::production_entry::pending_wal_world::first::Workload, crate::production_entry::pending_wal_world::first::World, crate::production_entry::pending_wal_world::first::producer::child, std::fs, crate::production_entry::pending_wal_world::first::objects_path, std::thread};

pub(crate) fn first() -> PendingWalWorld {
    first_with_segment_bytes(None, crate::production_entry::pending_wal_world::first::World::TwoChunks)
}

pub(crate) fn first_with_wal_segment_bytes(bytes: NonZeroU64) -> PendingWalWorld {
    first_with_segment_bytes(Some(bytes), crate::production_entry::pending_wal_world::first::World::TwoChunks)
}

pub(crate) fn first_with_failed_ingest_control() -> PendingWalWorld {
    first_with_segment_bytes(None, crate::production_entry::pending_wal_world::first::World::FailedIngestControl)
}

/// The first object is long enough to have checkpointed a resume frontier.
pub(crate) fn first_with_resume_frontier() -> PendingWalWorld {
    first_with_segment_bytes(None, crate::production_entry::pending_wal_world::first::World::ResumeFrontier)
}

/// Objects published above the baseline checkpoint, killed before the next.
pub(crate) fn published_above_checkpoint(workload: Workload, tail: Tail) -> PendingWalWorld {
    first_with_segment_bytes(None, crate::production_entry::pending_wal_world::first::World::Published(workload, tail))
}

pub(crate) fn first_with_segment_bytes(
    wal_segment_bytes: Option<NonZeroU64>,
    first: World,
) -> PendingWalWorld {
    let marker_dir = tempfile::tempdir().unwrap();
    let marker = marker_dir.path().join("ready");
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, first.role())
        .env(MARKER_ENV, &marker)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if let Some(bytes) = wal_segment_bytes {
        command.env(WAL_SEGMENT_BYTES_ENV, bytes.get().to_string());
    }
    let mut child = command.spawn().unwrap();
    // The 66-chunk world performs many separately bounded operations. Guard
    // stalled progress, rather than treating their sum as one operation.
    let mut deadline = std::time::Instant::now() + std::time::Duration::from_secs(240);
    let mut last_progress = String::new();
    while !marker.is_file() {
        if let Ok(progress) = std::fs::read_to_string(marker.with_extension("progress"))
            && progress != last_progress
        {
            last_progress = progress;
            deadline = std::time::Instant::now() + std::time::Duration::from_secs(240);
        }
        if child.try_wait().unwrap().is_some() || std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "pending V3 descriptor seam not reached after {last_progress:?}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let root = std::path::PathBuf::from(std::fs::read_to_string(&marker).unwrap());
    assert!(root.is_dir());
    let objects = std::fs::read(objects_path(&marker)).unwrap();
    PendingWalWorld {
        root,
        first,
        objects: objects
            .chunks_exact(16)
            .map(|object| object.try_into().unwrap())
            .collect(),
        _marker: marker_dir,
        next_descriptor: std::cell::Cell::new(0),
    }
}