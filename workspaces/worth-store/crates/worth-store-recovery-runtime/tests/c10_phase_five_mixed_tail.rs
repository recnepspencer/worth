#[allow(dead_code)]
mod c10_crash_evidence;
#[allow(dead_code)]
mod c10_phase_five_read;
#[allow(dead_code)]
mod phase_three_support;

use c10_crash_evidence::{
    ordinary_limits, prepare_rewrite, segment_snapshot, wal_contains_rewrite,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use phase_three_support::recovery_request_with_limits;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess,
    PhysicalMutationProvenNoEffectCause, PhysicalMutationRequest,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};
use worth_store_test_support::harness::physical_residency::{
    canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld,
};

const CHILD_MARKER: &str = "C10_PHASE5_MIXED_MARKER";
const CHILD_SEAM: &str = "C10_PHASE5_MIXED_SEAM";

#[test]
fn mixed_append_and_rewrite_tail_publishes_the_rewrite_once() {
    for seam in ["mixed-before-wal", "mixed-after-wal"] {
        let (parent, root) = kill_child(seam);
        let expected: &[u8] = b"phase5-mixed-append";
        let killed = segment_snapshot(&root);
        let rewrite_before = wal_contains_rewrite(&root);
        let (first, first_payload) = recover(&root, parent.path());
        let published = segment_snapshot(&root);
        let (second, second_payload) = recover(&root, parent.path());
        assert_eq!(first, second, "{seam}");
        assert_eq!(first_payload, expected, "{seam}");
        assert_eq!(second_payload, expected, "{seam}");
        assert_eq!(published, segment_snapshot(&root), "{seam}");
        if seam == "mixed-before-wal" {
            assert!(!rewrite_before, "{seam}");
            assert!(!wal_contains_rewrite(&root), "{seam}");
            assert_eq!(killed, published, "{seam}");
        } else {
            assert!(rewrite_before, "{seam}");
            assert_ne!(killed, published, "{seam}");
            assert!(published.len() <= killed.len() + 1, "{seam}");
        }
        drop(parent);
    }
}

#[test]
fn freshly_admitted_rewrite_keeps_the_published_append() {
    for seam in [
        "competing-then-after-wal",
        "competing-then-after-data",
        "competing-then-during-root",
    ] {
        let (parent, root) = kill_child(seam);
        let expected: &[u8] = b"phase5-member-a";
        let killed = segment_snapshot(&root);
        let (first, first_payload) = recover(&root, parent.path());
        let published = segment_snapshot(&root);
        let (second, second_payload) = recover(&root, parent.path());
        assert_eq!(first, second, "{seam}");
        assert_eq!(first_payload, expected, "{seam}");
        assert_eq!(second_payload, expected, "{seam}");
        assert_eq!(published, segment_snapshot(&root), "{seam}");
        assert!(
            published.len() <= killed.len() + 1,
            "{seam} published more than one rewritten generation"
        );
        drop(parent);
    }
}

#[test]
#[ignore = "launched by the phase 5 mixed-tail parent"]
fn c10_mixed_child_parks_at_seam() {
    let marker = PathBuf::from(std::env::var_os(CHILD_MARKER).expect("phase 5 mixed marker"));
    let seam = std::env::var(CHILD_SEAM).expect("phase 5 mixed seam");
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery("c10-phase5-mixed-tail").unwrap();
    std::fs::write(
        marker.join("root-path.txt"),
        world.root().to_string_lossy().as_bytes(),
    )
    .unwrap();
    canonical_physical_mutation_acknowledgment(&world, [0x91; 32], b"phase5-mixed-source");
    checkpoint_source(&world);
    match seam.as_str() {
        "mixed-before-wal" | "mixed-after-wal" => {
            publish_append(&world, &marker, [0x92; 32], b"phase5-mixed-append");
        }
        "competing-then-after-wal" | "competing-then-after-data" | "competing-then-during-root" => {
            finish_append_ahead_of_stale_rewrite(&world, &marker)
        }
        other => panic!("unknown mixed seam {other}"),
    }
    let gate = world
        .serving()
        .certification_pause_physical_mutation_at(rewrite_checkpoint(&seam));
    let rewrite = prepare_rewrite(&world, [0x94; 32]).start();
    assert!(gate.await_arrival(), "rewrite did not reach {seam}");
    std::mem::forget(rewrite);
    std::mem::forget(gate);
    std::fs::write(marker.join("reached"), b"1").unwrap();
    loop {
        thread::park();
    }
}

fn finish_append_ahead_of_stale_rewrite(world: &PhysicalResidencyStoreWorld, marker: &Path) {
    let gate = world.serving().certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let append = prepare_append(world, [0x93; 32], b"phase5-member-a").start();
    assert!(gate.await_arrival(), "append did not reach WAL durability");
    match prepare_rewrite(world, [0x95; 32]).execute() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                PhysicalMutationProvenNoEffectCause::ScopeConflict
            );
        }
        PhysicalMutationOutcome::Completed(_) => {
            panic!("stale rewrite must not pass the pending append")
        }
        PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("stale rewrite reached an effect")
        }
    }
    gate.release();
    match append.wait() {
        PhysicalMutationOutcome::Completed(done) => {
            c10_phase_five_read::store_identity(marker, done.persisted_records()[0]);
        }
        PhysicalMutationOutcome::ProvenNoEffect(_) | PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("released append must publish")
        }
    }
}

fn rewrite_checkpoint(seam: &str) -> CertificationPhysicalMutationCheckpoint {
    match seam {
        "mixed-before-wal" => CertificationPhysicalMutationCheckpoint::BeforeWalAppend,
        "mixed-after-wal" | "competing-then-after-wal" => {
            CertificationPhysicalMutationCheckpoint::AfterWalDurability
        }
        "competing-then-after-data" => CertificationPhysicalMutationCheckpoint::AfterDataSettlement,
        "competing-then-during-root" => {
            CertificationPhysicalMutationCheckpoint::DuringRootPublication
        }
        other => panic!("unknown mixed seam {other}"),
    }
}

fn publish_append(
    world: &PhysicalResidencyStoreWorld,
    marker: &Path,
    material: [u8; 32],
    payload: &'static [u8],
) {
    match prepare_append(world, material, payload).execute() {
        PhysicalMutationOutcome::Completed(done) => {
            c10_phase_five_read::store_identity(marker, done.persisted_records()[0]);
        }
        PhysicalMutationOutcome::ProvenNoEffect(_) | PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("mixed append must publish")
        }
    }
}

fn checkpoint_source(world: &PhysicalResidencyStoreWorld) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x96; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match world.serving().checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("source checkpoint was not admitted"),
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(_) => {}
        _ => panic!("source checkpoint must complete"),
    }
}

fn prepare_append(
    world: &PhysicalResidencyStoreWorld,
    material: [u8; 32],
    payload: &'static [u8],
) -> worth_store::physical_runtime::PreparedPhysicalMutation {
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload]).unwrap(),
            world.placement(),
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(1_000).unwrap(),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("append preparation must succeed"),
    }
}

fn kill_child(seam: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "c10_mixed_child_parks_at_seam",
            "--ignored",
            "--nocapture",
        ])
        .env(CHILD_MARKER, parent.path())
        .env(CHILD_SEAM, seam)
        .env("TMP", parent.path())
        .env("TEMP", parent.path())
        .env("TMPDIR", parent.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch phase 5 mixed child");
    let reached = parent.path().join("reached");
    let deadline = Instant::now() + Duration::from_secs(60);
    while !reached.exists() {
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("mixed child did not reach {seam}");
        }
        if let Some(status) = child.try_wait().unwrap() {
            let output = child.wait_with_output().unwrap();
            panic!(
                "mixed child exited early: {status}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(20));
    }
    child.kill().unwrap();
    let _ = child.wait();
    let root = PathBuf::from(std::fs::read_to_string(parent.path().join("root-path.txt")).unwrap());
    (parent, root)
}

fn recover(root: &Path, marker: &Path) -> (Vec<(u64, u64, u64, u64)>, Vec<u8>) {
    let outcome =
        WorthStoreRecovery::recover(recovery_request_with_limits(root, ordinary_limits()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        if let PhysicalRecoveryOutcome::Blocked(blocked) = &outcome {
            panic!(
                "mixed tail blocked {:?} artifact {:?} denial {:?} lsn {:?}",
                blocked.kind,
                blocked.evidence().artifact,
                blocked.evidence().planning_denial,
                blocked.evidence().lsn
            );
        }
        panic!("mixed tail must recover");
    };
    let payload = c10_phase_five_read::selected_payload(
        root,
        &handoff,
        c10_phase_five_read::load_identity(marker),
    );
    let retirements = handoff
        .freshness_sample()
        .retirements()
        .iter()
        .map(|retirement| {
            (
                retirement.source_root(),
                retirement
                    .artifact()
                    .segment()
                    .expect("an inline rewrite retires a segment generation"),
                retirement.artifact().generation(),
                retirement.bytes(),
            )
        })
        .collect();
    drop(handoff);
    (retirements, payload)
}
