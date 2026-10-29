//! Worlds killed while Serving moved selected media: an inline rewrite and a
//! final extent copy, each killed once its WAL was durable and before its
//! root published. Completing either reads the moved bytes again.

pub use {crate::production_entry::published_above_checkpoint::limit_sweep::worlds::Killed, std::path::Path, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, crate::production_entry::published_above_checkpoint::limit_sweep::reached};
pub use {std::fs, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};
pub use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::ServingPhysicalRuntime};



pub const ROLE_ENV: &str = "WORTH_C11_SWEEP_MOVED_MEDIA_ROLE";
pub const MARKER_ENV: &str = "WORTH_C11_SWEEP_MOVED_MEDIA_MARKER";
pub const CHILD_TEST: &str = "published_above_checkpoint::limit_sweep::moved_media::moved_media_child";

/// A Store root the child left as the kill found it, under a directory the
/// world owns.
pub struct KilledMove {
    pub root: PathBuf,
    pub _directory: tempfile::TempDir,
}

impl Killed for KilledMove {
    fn root(&self) -> &Path {
        &self.root
    }
}

/// An inline segment rewrite of one checkpointed record.
pub fn killed_rewrite() -> KilledMove {
    kill("rewrite")
}

/// A selected extent copied to another arena.
pub fn killed_copy() -> KilledMove {
    kill("copy")
}

pub fn kill(role: &str) -> KilledMove {
    let directory = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture"])
        .env(ROLE_ENV, role)
        .env(MARKER_ENV, directory.path())
        .env("TMP", directory.path())
        .env("TEMP", directory.path())
        .env("TMPDIR", directory.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let reached = directory.path().join("reached");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while !reached.exists() {
        if child.try_wait().unwrap().is_some() || std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "{role}: WAL durability not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let root = std::path::PathBuf::from(std::fs::read_to_string(directory.path().join("root-path.txt")).unwrap());
    KilledMove {
        root,
        _directory: directory,
    }
}

pub fn park_rewrite(marker: &Path) {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("c11-sweep-rewrite").unwrap();
    std::fs::write(
        marker.join("root-path.txt"),
        world.root().to_string_lossy().as_bytes(),
    )
    .unwrap();
    canonical_physical_mutation_acknowledgment(&world, [0x91; 32], b"sweep-rewrite-source");
    checkpoint(world.serving(), [0x92; 32]);
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0x93; 32]))
        .unwrap();
    let worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .rewrite_selected_inline_segment(world.placement(), durable(key))
            .into_raw()
    else {
        panic!("rewrite preparation must succeed")
    };
    park_at_wal(world.serving(), marker, || prepared.start());
}

pub fn park_copy(marker: &Path) {
    let root = marker.join("store");
    std::fs::write(
        marker.join("root-path.txt"),
        root.to_string_lossy().as_bytes(),
    )
    .unwrap();
    let serving = crate::published_above_checkpoint::limit_sweep::moved_media::arena::initialize(&root);
    let (_, placement, _) = crate::published_above_checkpoint::limit_sweep::moved_media::arena::configuration();
    crate::published_above_checkpoint::limit_sweep::moved_media::arena::append(&serving, placement, [0x94; 32], b"seed");
    let extent = vec![0x5d; crate::published_above_checkpoint::limit_sweep::moved_media::arena::EXTENT_BYTES];
    let record = crate::published_above_checkpoint::limit_sweep::moved_media::arena::append(&serving, placement, [0x95; 32], &extent).persisted_records()[0];
    checkpoint(&serving, [0x96; 32]);
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0x97; 32]))
        .unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, durable(key), record)
        .unwrap()
    else {
        panic!("selected extent copy did not admit")
    };
    for _ in 0..128 {
        if progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert_eq!(progress.phase, worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption);
    let prepared = submission.prepare_completed_extent_copy().unwrap();
    park_at_wal(&serving, marker, || prepared.start());
}

pub fn durable(
    key: worth_store::physical_runtime::PhysicalMutationIdempotencyKey,
) -> PhysicalMutationRequest {
    worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
        key,
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
}

/// Starts the mutation, waits until its WAL is durable, and parks there for
/// the parent's kill.
pub fn park_at_wal<T>(serving: &ServingPhysicalRuntime, marker: &Path, start: impl FnOnce() -> T) {
    let gate = serving.certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let mutation = start();
    assert!(
        gate.await_arrival(),
        "the move did not reach WAL durability"
    );
    std::mem::forget(mutation);
    std::mem::forget(gate);
    std::fs::write(marker.join("reached"), b"wal-durable").unwrap();
    loop {
        std::thread::park();
    }
}

pub fn checkpoint(serving: &ServingPhysicalRuntime, key: [u8; 32]) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("source checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
}