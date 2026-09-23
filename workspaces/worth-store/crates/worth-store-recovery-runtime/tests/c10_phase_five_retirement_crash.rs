#[allow(dead_code)]
mod c10_crash_evidence;
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
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationProvenNoEffectCause,
    PhysicalMutationRequest,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};
use worth_store_test_support::harness::physical_residency::{
    canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld,
};

const CHILD_MARKER: &str = "C10_PHASE5_RETIREMENT_MARKER";
const CHILD_SEAM: &str = "C10_PHASE5_RETIREMENT_SEAM";

#[test]
fn killed_retirement_intent_survives_a_fresh_recovery_and_second_reopen() {
    let (parent, root) = kill_child("before-delete");
    let present = segment_names(&root);
    let first = recovered_retirements(&root);
    let second = recovered_retirements(&root);
    assert_eq!(first, second);
    assert_eq!(first.len(), 1);
    assert!(first[0].3 > 0);
    let name = segment_name(first[0].1, first[0].2);
    assert!(present.iter().any(|file| file == &name));
    assert!(segment_names(&root).iter().any(|file| file == &name));
    drop(parent);
}

#[test]
fn killed_retirement_after_unlink_keeps_the_obligation_until_completion() {
    let (parent, root) = kill_child("after-unlink");
    let present = segment_names(&root);
    let first = recovered_retirements(&root);
    let second = recovered_retirements(&root);
    assert_eq!(first, second);
    assert_eq!(first.len(), 1);
    assert!(first[0].3 > 0);
    let name = segment_name(first[0].1, first[0].2);
    assert!(present.iter().all(|file| file != &name));
    assert!(segment_names(&root).iter().all(|file| file != &name));
    drop(parent);
}

#[test]
fn killed_retirement_after_completion_does_not_resurrect_the_obligation() {
    let obligation = kill_and_recover("after-completion");
    assert!(
        obligation.is_empty(),
        "a durable completion must not come back as unfinished cleanup"
    );
}

#[test]
fn killed_rewrite_behind_a_durable_append_leaves_no_rewrite_effect() {
    let (parent, root) = kill_child("competing-append");
    let killed = segment_snapshot(&root);
    assert!(
        !wal_contains_rewrite(&root),
        "a blocked rewrite must not append redo before the kill"
    );
    let first = recovered_retirements(&root);
    let published = segment_snapshot(&root);
    let second = recovered_retirements(&root);
    assert_eq!(first, second);
    assert!(
        first.is_empty(),
        "the blocked rewrite must not retire a generation"
    );
    assert_ne!(
        killed, published,
        "recovery must publish the append that was already WAL-durable"
    );
    assert!(
        !wal_contains_rewrite(&root),
        "recovery must not invent rewrite redo for the blocked writer"
    );
    assert_eq!(published, segment_snapshot(&root));
    drop(parent);
}

#[test]
#[ignore = "launched by the phase 5 retirement parent"]
fn c10_retirement_child_parks_at_seam() {
    let marker = PathBuf::from(std::env::var_os(CHILD_MARKER).expect("phase 5 marker"));
    let seam = std::env::var(CHILD_SEAM).expect("phase 5 seam");
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery("c10-phase5-retirement-kill").unwrap();
    std::fs::write(
        marker.join("root-path.txt"),
        world.root().to_string_lossy().as_bytes(),
    )
    .unwrap();
    canonical_physical_mutation_acknowledgment(&world, [0x71; 32], b"phase5-retirement-source");
    if seam != "competing-append" {
        publish_rewrite(&world);
    }
    match seam.as_str() {
        "before-delete" => park_retirement_until_killed(&world, &marker, 1),
        "after-unlink" => park_retirement_until_killed(&world, &marker, 2),
        "after-completion" => {
            world
                .serving()
                .retire_displaced_segment()
                .expect("retirement reaches a durable completion");
        }
        "competing-append" => park_behind_durable_append(&world),
        other => panic!("unknown seam {other}"),
    }
    std::fs::write(marker.join("reached"), b"1").unwrap();
    loop {
        thread::park();
    }
}

fn park_retirement_until_killed(world: &PhysicalResidencyStoreWorld, marker: &Path, seam: u8) {
    let arrived = world.serving().certification_arm_retirement_kill(seam);
    let marker = marker.to_path_buf();
    thread::spawn(move || {
        while !arrived.load(std::sync::atomic::Ordering::Acquire) {
            thread::sleep(Duration::from_millis(1));
        }
        std::fs::write(marker.join("reached"), b"1").unwrap();
        loop {
            thread::park();
        }
    });
    match world.serving().retire_displaced_segment() {
        returned => panic!("armed retirement seam {seam} returned before the kill: {returned:?}"),
    }
}

fn segment_name(segment: u64, generation: u64) -> String {
    format!("segment-{segment:016x}-{generation:016x}.pages")
}

fn segment_names(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root.join("families/records/segments")) {
        for entry in entries.flatten() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names
}

fn park_behind_durable_append(world: &PhysicalResidencyStoreWorld) {
    checkpoint_source(world);
    let gate = world.serving().certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let append = prepare_append(world, [0x73; 32], b"wal-durable-append").start();
    assert!(gate.await_arrival(), "append did not reach WAL durability");
    match prepare_rewrite(world, [0x74; 32]).execute() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                PhysicalMutationProvenNoEffectCause::ScopeConflict
            );
        }
        PhysicalMutationOutcome::Completed(_) => panic!("rewrite passed a pending append"),
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("rewrite reached an effect: {:?}", fate.stage())
        }
    }
    std::mem::forget(append);
    std::mem::forget(gate);
}

fn checkpoint_source(world: &PhysicalResidencyStoreWorld) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x75; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let handle = match world.serving().checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("source checkpoint was not admitted"),
    };
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_) => {}
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

fn kill_and_recover(seam: &str) -> Vec<(u64, u64, u64, u64)> {
    let (parent, root) = kill_child(seam);
    let first = recovered_retirements(&root);
    let second = recovered_retirements(&root);
    assert_eq!(first, second, "{seam} second reopen changed the obligation");
    drop(parent);
    first
}

fn kill_child(seam: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "c10_retirement_child_parks_at_seam",
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
        .expect("launch phase 5 retirement child");
    let reached = parent.path().join("reached");
    let deadline = Instant::now() + Duration::from_secs(60);
    while !reached.exists() {
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("child did not reach the {seam} seam");
        }
        if let Some(status) = child.try_wait().unwrap() {
            let output = child.wait_with_output().unwrap();
            panic!(
                "child exited before {seam}: {status}\nstdout:\n{}\nstderr:\n{}",
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

fn publish_rewrite(world: &PhysicalResidencyStoreWorld) {
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x72; 32]))
        .unwrap();
    let prepared = match submission
        .rewrite_selected_inline_segment(
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
        TransitionOutcome::Success(_) => panic!("rewrite preparation must return a mutation"),
        TransitionOutcome::Denied(_) => panic!("rewrite preparation was denied"),
        TransitionOutcome::Deferred(_) => panic!("rewrite preparation was deferred"),
        TransitionOutcome::Stale(_) => panic!("rewrite preparation was stale"),
        TransitionOutcome::RebindRequired(_) => panic!("rewrite preparation required rebind"),
        TransitionOutcome::Failed(_) => panic!("rewrite preparation failed"),
    };
    match prepared.execute() {
        PhysicalMutationOutcome::Completed(_) => {}
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("rewrite must publish: {:?}", fate.cause())
        }
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("rewrite must settle: {:?}", fate.stage())
        }
    }
}

fn recovered_retirements(root: &Path) -> Vec<(u64, u64, u64, u64)> {
    let outcome =
        WorthStoreRecovery::recover(recovery_request_with_limits(root, ordinary_limits()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("killed retirement must recover: {outcome:?}")
    };
    handoff
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
        .collect()
}
