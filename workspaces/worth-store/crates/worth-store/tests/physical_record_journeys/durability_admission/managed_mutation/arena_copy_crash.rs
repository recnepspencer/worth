use super::*;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use worth_store::physical_runtime::PhysicalExtentCopyPhase;

const CHILD: &str = "durability_admission::managed_mutation::extent_record_rewrite::arena_copy_crash::copy_bytes_before_final_child";
const DIRECTORY: &str = "C11_COPY_CRASH_DIRECTORY";
const SEAM: &str = "C11_COPY_CRASH_SEAM";

#[test]
fn partial_copy_bytes_before_final_kill_preserves_private_destination_claim() {
    verify_killed_copy("partial");
}

#[test]
fn fully_copied_unpublished_destination_reopens_with_private_claim() {
    verify_killed_copy("ready");
}

fn verify_killed_copy(seam: &str) {
    use super::super::super::independent_wal_oracle::produced_copy_intents;
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    kill_after_copy_bytes(directory.path(), seam);
    assert_eq!(
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
        std::fs::read(directory.path().join("baseline-catalog")).unwrap(),
        "copy destination has no published selector/root at this seam"
    );
    let intents = produced_copy_intents(&root);
    assert_eq!(
        intents.len(),
        1,
        "one authentic copy intent survives process kill"
    );
    let intent = intents[0];
    assert_ne!(intent.source[0], intent.destination[0]);
    assert_eq!(
        intent.destination[1], 0,
        "new arena is an unpublished reservation"
    );
    assert!(
        root.join(format!(
            "families/records/arenas/arena-{:016x}.data",
            intent.destination[0]
        ))
        .exists(),
        "copied bytes exist without a published destination route"
    );
    let report = observe_after_kill(&root, directory.path());
    assert_eq!(report["completeness"], "complete", "{report:#?}");
    let artifacts = report["artifacts"].as_array().unwrap();
    let stage = artifacts
        .iter()
        .find(|artifact| {
            artifact["identity"]
                .as_str()
                .unwrap()
                .starts_with("copy-staging:")
        })
        .expect("observer classifies exact WAL-held destination staging range");
    assert_eq!(stage["range"]["offset"], intent.destination[1]);
    assert_eq!(stage["range"]["length"], intent.destination[2]);
    assert_eq!(stage["outcome"]["posture"], "unknown");
    assert!(
        artifacts.iter().any(|artifact| artifact["path"]
            .as_str()
            .unwrap()
            .contains("arena-0000000000000001.data")
            && artifact["family"] == "extent_manifest"
            && artifact["identity"]
                .as_str()
                .unwrap()
                .starts_with("arena:0000000000000001:extent:")
            && !artifact["identity"].as_str().unwrap().contains(":chunk:")
            && artifact["outcome"]["posture"] == "intact"),
        "source remains independently readable at the crash boundary"
    );
    let serving = crate::serving_from_open(&root);
    assert!(
        serving.certification_holds_recovered_copy_destination(
            intent.destination[0],
            intent.destination[1],
            intent.destination[2]
        ),
        "fresh reopen retains the exact WAL-destined private range before allocator exposure"
    );
    assert!(
        serving.observed_non_authoritative_residue(),
        "unpublished staged bytes still require the separate C8 recovery pass"
    );
    assert!(
        !serving.physical_recovery_evidence_damaged(),
        "a valid WAL-held staging range is not physical damage"
    );
    serving.close();
}

fn observe_after_kill(root: &Path, directory: &Path) -> serde_json::Value {
    super::independent_arena_observer::observe_arena_in_separate_process(
        root,
        directory,
        "copy-crash",
        "killed",
    )
}

#[test]
#[ignore = "parent terminates this process after a durable copy intent and copied frame"]
fn copy_bytes_before_final_child() {
    let directory = PathBuf::from(std::env::var_os(DIRECTORY).unwrap());
    let seam = std::env::var(SEAM).unwrap();
    let root = directory.join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let payload = vec![73; 40_000];
    let append = completed(prepare(&serving, placement, [241; 32], &payload).execute());
    let identity = append.persisted_records()[0];
    std::fs::write(
        directory.join("baseline-catalog"),
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
    )
    .unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([242; 32]))
        .unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request(key), identity)
        .unwrap()
    else {
        panic!("selected copy rejected")
    };
    for _ in 0..128 {
        progress = submission.advance_extent_copy().unwrap();
        if seam == "partial"
            && progress.phase == PhysicalExtentCopyPhase::Copying
            && progress.completed_payload_bytes > 0
        {
            break;
        }
        if seam == "ready" && progress.phase == PhysicalExtentCopyPhase::ReadyForAdoption {
            break;
        }
    }
    match seam.as_str() {
        "partial" => assert!(
            progress.phase == PhysicalExtentCopyPhase::Copying
                && progress.completed_payload_bytes > 0
                && progress.completed_payload_bytes < payload.len() as u64
        ),
        "ready" => assert_eq!(progress.phase, PhysicalExtentCopyPhase::ReadyForAdoption),
        _ => panic!("unknown crash seam"),
    }
    std::fs::write(directory.join("reached"), b"durable-intent-and-bytes").unwrap();
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn kill_after_copy_bytes(directory: &Path, seam: &str) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .env(DIRECTORY, directory)
        .env(SEAM, seam)
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
                "copy child failed to reach crash seam: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
}
