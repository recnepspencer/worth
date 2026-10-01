#[path = "c11_arena_crash_support.rs"]
mod support;

use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use support::*;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    certification::CertificationPhysicalMutationCheckpoint, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalExtentCopyPhase, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationRequest,
};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, PersistedRecordIdentity,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryOutcome, RecoveredPhysicalRuntimeHandoff, WorthStoreRecovery,
};
#[path = "c11_arena_copy_crash/observation.rs"]
mod observation;
use observation::observe_copy_media;
#[path = "c11_arena_copy_crash/mixed_history/copy_frame.rs"]
mod copy_frame;
#[path = "c11_arena_copy_crash/historical_displacement.rs"]
mod historical_displacement;
#[path = "c11_arena_copy_crash/mixed_history.rs"]
mod mixed_history;
#[path = "c11_arena_copy_crash/pre_final.rs"]
mod pre_final;

const CHILD: &str = "final_copy_wal_child";
const MARKER: &str = "C11_FINAL_COPY_CRASH_MARKER";

#[test]
fn killed_final_copy_wal_replays_one_destination_before_serving() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    kill_after_final_wal(directory.path());
    let baseline = std::fs::read(directory.path().join("baseline-catalog")).unwrap();
    assert_eq!(
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
        baseline,
        "the durable final WAL has not published a root"
    );
    observe_copy_media(&root, directory.path(), "killed");
    let record = load_identity(directory.path());
    let first = recover(&root);
    let source = selected_extent(&first, record);
    assert_eq!(
        generation(&baseline),
        first
            .selected_sources()
            .root()
            .selected()
            .selector()
            .root_generation()
    );
    drop(first);
    let second = recover(&root);
    let destination = selected_extent(&second, record);
    assert_eq!(destination.extent(), source.extent());
    assert_eq!(
        destination.extent_generation(),
        source.extent_generation() + 1
    );
    assert_ne!(
        destination.arena_range().arena(),
        source.arena_range().arena(),
        "copy evacuation publishes into another arena"
    );
    drop(second);
    let third = recover(&root);
    assert_eq!(
        selected_extent(&third, record),
        destination,
        "repeated recovery cannot publish another copy generation"
    );
    drop(third);
    let serving = open(&root);
    let rows = scan(&serving);
    assert_eq!(rows.len(), 2, "seed and selected extent each appear once");
    assert_eq!(rows.iter().filter(|(_, bytes)| bytes == b"seed").count(), 1);
    assert_eq!(
        rows.iter()
            .filter(|(_, bytes)| bytes.len() == EXTENT_BYTES && bytes.iter().all(|&byte| byte == 93))
            .count(),
        1
    );
    serving.close();
    observe_copy_media(&root, directory.path(), "recovered");
}

#[test]
#[ignore = "parent kills after final copy WAL durability and before root publication"]
fn final_copy_wal_child() {
    let directory = PathBuf::from(std::env::var_os(MARKER).unwrap());
    let root = directory.join("store");
    let serving = initialize(&root);
    let (_, placement, _) = configuration();
    append(&serving, placement, [201; 32], b"seed");
    let completed = append(&serving, placement, [202; 32], &vec![93; EXTENT_BYTES]);
    let identity = completed.persisted_records()[0];
    save_identity(&directory, identity);
    checkpoint_source(&serving);
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([203; 32]))
        .unwrap();
    let request = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    );
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request, identity)
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
    let original = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    std::fs::write(directory.join("baseline-catalog"), &original).unwrap();
    let gate = serving.certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let mutation = prepared.start();
    assert!(
        gate.await_arrival(),
        "final copy did not reach durable-WAL boundary"
    );
    assert_eq!(
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
        original
    );
    std::mem::forget(mutation);
    std::mem::forget(gate);
    std::fs::write(directory.join("reached"), b"final-copy-wal-durable").unwrap();
    loop {
        thread::park();
    }
}

fn checkpoint_source(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([204; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("source checkpoint did not admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}

fn kill_after_final_wal(directory: &Path) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .env(MARKER, directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    while !directory.join("reached").exists() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "final WAL seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
}

fn recover(root: &Path) -> RecoveredPhysicalRuntimeHandoff {
    match WorthStoreRecovery::recover(recovery_request(root)) {
        PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
        outcome => panic!("final copy recovery blocked: {outcome:?}"),
    }
}

fn selected_extent(
    handoff: &RecoveredPhysicalRuntimeHandoff,
    record: PersistedRecordIdentity,
) -> DurableExtentRecordPlacement {
    handoff
        .selected_sources()
        .page_facts()
        .placements()
        .iter()
        .find_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Extent(extent) if extent.record() == record => {
                Some(*extent)
            }
            _ => None,
        })
        .expect("selected root must route the copied extent")
}

fn save_identity(directory: &Path, record: PersistedRecordIdentity) {
    let mut bytes = record.allocation_epoch().to_vec();
    bytes.extend_from_slice(&record.ordinal().to_le_bytes());
    std::fs::write(directory.join("record-id.bin"), bytes).unwrap();
}

fn load_identity(directory: &Path) -> PersistedRecordIdentity {
    let bytes = std::fs::read(directory.join("record-id.bin")).unwrap();
    let epoch = bytes[..16].try_into().unwrap();
    let ordinal = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
    PersistedRecordIdentity::new(epoch, ordinal).unwrap()
}
