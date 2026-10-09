//! The parent side of the first world: start the child, wait for it to park,
//! kill it and adopt the root it left.

use super::*;

pub(crate) fn first() -> PendingWalWorld {
    first_with_segment_bytes(None, World::TwoChunks)
}

pub(crate) fn first_with_wal_segment_bytes(bytes: NonZeroU64) -> PendingWalWorld {
    first_with_segment_bytes(Some(bytes), World::TwoChunks)
}

pub(crate) fn first_with_failed_ingest_control() -> PendingWalWorld {
    first_with_segment_bytes(None, World::FailedIngestControl)
}

/// The first object is long enough to have checkpointed a resume frontier.
pub(crate) fn first_with_resume_frontier() -> PendingWalWorld {
    first_with_segment_bytes(None, World::ResumeFrontier)
}

/// Objects published above the baseline checkpoint, killed before the next.
pub(crate) fn published_above_checkpoint(workload: Workload, tail: Tail) -> PendingWalWorld {
    first_with_segment_bytes(None, World::Published(workload, tail))
}

fn first_with_segment_bytes(
    wal_segment_bytes: Option<NonZeroU64>,
    first: World,
) -> PendingWalWorld {
    let marker_dir = tempfile::tempdir().unwrap();
    let marker = marker_dir.path().join("ready");
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, first.role())
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
    let objects = fs::read(objects_path(&marker)).unwrap();
    PendingWalWorld {
        root,
        first,
        objects: objects
            .chunks_exact(16)
            .map(|object| object.try_into().unwrap())
            .collect(),
        _marker: marker_dir,
        next_descriptor: Cell::new(0),
    }
}
