//! Worlds killed while Serving moved selected media: an inline rewrite and a
//! final extent copy, each killed once its WAL was durable and before its
//! root published. Completing either reads the moved bytes again.

use super::*;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use worth_store::physical_runtime::{
    certification::CertificationPhysicalMutationCheckpoint, PhysicalExtentCopyPhase,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationPreparationSuccess,
    PhysicalMutationRequest, ServingPhysicalRuntime,
};

#[allow(dead_code)]
#[path = "../../../c11_arena_crash_support/mod.rs"]
mod arena;

const ROLE_ENV: &str = "WORTH_C11_SWEEP_MOVED_MEDIA_ROLE";
const MARKER_ENV: &str = "WORTH_C11_SWEEP_MOVED_MEDIA_MARKER";
const CHILD_TEST: &str = "published_above_checkpoint::limit_sweep::moved_media::moved_media_child";

/// A Store root the child left as the kill found it, under a directory the
/// world owns.
pub(super) struct KilledMove {
    root: PathBuf,
    _directory: tempfile::TempDir,
}

impl Killed for KilledMove {
    fn root(&self) -> &Path {
        &self.root
    }
}

/// An inline segment rewrite of one checkpointed record.
pub(super) fn killed_rewrite() -> KilledMove {
    kill("rewrite")
}

/// A selected extent copied to another arena.
pub(super) fn killed_copy() -> KilledMove {
    kill("copy")
}

fn kill(role: &str) -> KilledMove {
    let directory = tempfile::tempdir().unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture"])
        .env(ROLE_ENV, role)
        .env(MARKER_ENV, directory.path())
        .env("TMP", directory.path())
        .env("TEMP", directory.path())
        .env("TMPDIR", directory.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let reached = directory.path().join("reached");
    let deadline = Instant::now() + Duration::from_secs(120);
    while !reached.exists() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "{role}: WAL durability not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let root = PathBuf::from(fs::read_to_string(directory.path().join("root-path.txt")).unwrap());
    KilledMove {
        root,
        _directory: directory,
    }
}

#[test]
fn moved_media_child() {
    let Some(marker) = std::env::var_os(MARKER_ENV) else {
        return;
    };
    let marker = PathBuf::from(marker);
    match std::env::var(ROLE_ENV).as_deref() {
        Ok("rewrite") => park_rewrite(&marker),
        Ok("copy") => park_copy(&marker),
        role => panic!("unknown moved-media role {role:?}"),
    }
}

fn park_rewrite(marker: &Path) {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("c11-sweep-rewrite").unwrap();
    fs::write(
        marker.join("root-path.txt"),
        world.root().to_string_lossy().as_bytes(),
    )
    .unwrap();
    canonical_physical_mutation_acknowledgment(&world, [0x91; 32], b"sweep-rewrite-source");
    checkpoint(world.serving(), [0x92; 32]);
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x93; 32]))
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .rewrite_selected_inline_segment(world.placement(), durable(key))
            .into_raw()
    else {
        panic!("rewrite preparation must succeed")
    };
    park_at_wal(world.serving(), marker, || prepared.start());
}

fn park_copy(marker: &Path) {
    let root = marker.join("store");
    fs::write(
        marker.join("root-path.txt"),
        root.to_string_lossy().as_bytes(),
    )
    .unwrap();
    let serving = arena::initialize(&root);
    let (_, placement, _) = arena::configuration();
    arena::append(&serving, placement, [0x94; 32], b"seed");
    let extent = vec![0x5d; arena::EXTENT_BYTES];
    let record = arena::append(&serving, placement, [0x95; 32], &extent).persisted_records()[0];
    checkpoint(&serving, [0x96; 32]);
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x97; 32]))
        .unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, durable(key), record)
        .unwrap()
    else {
        panic!("selected extent copy did not admit")
    };
    for _ in 0..128 {
        if progress.phase == PhysicalExtentCopyPhase::ReadyForAdoption {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert_eq!(progress.phase, PhysicalExtentCopyPhase::ReadyForAdoption);
    let prepared = submission.prepare_completed_extent_copy().unwrap();
    park_at_wal(&serving, marker, || prepared.start());
}

fn durable(
    key: worth_store::physical_runtime::PhysicalMutationIdempotencyKey,
) -> PhysicalMutationRequest {
    PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
}

/// Starts the mutation, waits until its WAL is durable, and parks there for
/// the parent's kill.
fn park_at_wal<T>(serving: &ServingPhysicalRuntime, marker: &Path, start: impl FnOnce() -> T) {
    let gate = serving.certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let mutation = start();
    assert!(
        gate.await_arrival(),
        "the move did not reach WAL durability"
    );
    std::mem::forget(mutation);
    std::mem::forget(gate);
    fs::write(marker.join("reached"), b"wal-durable").unwrap();
    loop {
        thread::park();
    }
}

fn checkpoint(serving: &ServingPhysicalRuntime, key: [u8; 32]) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("source checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}
