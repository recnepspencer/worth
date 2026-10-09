use super::*;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use worth_store::physical_runtime::{
    ArenaEvacuationThreshold, ExtentArenaCapacity, PhysicalArenaEvacuationPreparationOutcome,
    PhysicalExtentCopyPhase, PhysicalRecordInitialization, PhysicalRecordPlacementPolicy,
};
use worth_store_physical_format::RecordArtifactFile;

const CHILD: &str = "durability_admission::managed_mutation::extent_record_rewrite::arena_retirement_crash::arena_retirement_child";
const MARKER: &str = "C11_ARENA_RETIRE_MARKER";
const SEAM: &str = "C11_ARENA_RETIRE_SEAM";
const SMALL_WAL: &str = "C11_ARENA_RETIRE_SMALL_WAL";

pub(super) fn small_wal_policy() -> worth_store::physical_runtime::PhysicalWalPolicy {
    use std::num::{NonZeroU32, NonZeroU64};
    use worth_store::physical_runtime::{
        PhysicalWalPolicy, WalSegmentByteLimit, WalSegmentInventoryLimit,
    };
    PhysicalWalPolicy::segmented(
        WalSegmentByteLimit::new(NonZeroU64::new(128 << 10).unwrap()),
        WalSegmentInventoryLimit::new(NonZeroU32::new(1024).unwrap()),
    )
}

#[test]
fn process_kills_converge_whole_arena_retirement() {
    use super::super::super::independent_wal_oracle::{
        produced_retirement_payloads, IndependentRetiredKind, IndependentRetirementAction,
    };
    for seam in [1_u8, 2, 3, 4] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("store");
        kill_at_seam(directory.path(), seam);
        assert_offline_arena_integrity(&root, directory.path(), seam, "killed");
        let arena = arena_file(&root);
        assert_eq!(arena.exists(), seam != 2, "seam {seam} file state");
        let records = produced_retirement_payloads(&root);
        let arena_intents = records
            .iter()
            .filter(|record| {
                record.kind == IndependentRetiredKind::Arena
                    && record.action == IndependentRetirementAction::Intent
            })
            .count();
        assert_eq!(
            arena_intents, 1,
            "seam {seam} keeps one durable original intent"
        );
        let serving = super::arena_retirement::reopen_with_copy_policy(&root);
        serving
            .retire_displaced_segment()
            .unwrap_or_else(|denial| panic!("seam {seam} retirement resume failed: {denial:?}"));
        assert!(!arena.exists(), "seam {seam} resumes exact arena deletion");
        assert!(serving.records().is_ok());
        serving.close();
        let records = produced_retirement_payloads(&root);
        assert_eq!(
            records
                .iter()
                .filter(|record| record.kind == IndependentRetiredKind::Arena
                    && record.action == IndependentRetirementAction::Intent)
                .count(),
            1
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.kind == IndependentRetiredKind::Arena
                    && record.action == IndependentRetirementAction::Completion)
                .count(),
            1
        );
        let reopened = super::arena_retirement::reopen_with_copy_policy(&root);
        assert_eq!(
            reopened.retire_displaced_segment(),
            Err(PhysicalRetirementDenial::Absent)
        );
        reopened.close();
        assert_offline_arena_integrity(&root, directory.path(), seam, "recovered");
    }
}

pub(super) fn observe_arena_report(
    root: &Path,
    directory: &Path,
    seam: u8,
    stage: &str,
) -> serde_json::Value {
    super::independent_arena_observer::observe_arena_in_separate_process(
        root,
        directory,
        "arena-retire",
        &format!("{seam}-{stage}"),
    )
}

pub(super) fn assert_offline_arena_integrity(root: &Path, directory: &Path, seam: u8, stage: &str) {
    let report = observe_arena_report(root, directory, seam, stage);
    let artifacts = report["artifacts"].as_array().unwrap();
    let non_intact = artifacts
        .iter()
        .filter(|artifact| artifact["outcome"]["posture"] != "intact")
        .map(|artifact| (&artifact["path"], &artifact["outcome"]))
        .collect::<Vec<_>>();
    assert_eq!(
        report["completeness"], "complete",
        "seam {seam} {stage} offline observation was incomplete: {non_intact:?}"
    );
    let arenas = artifacts
        .iter()
        .filter(|artifact| artifact["path"].as_str().unwrap().contains("/arenas/"))
        .collect::<Vec<_>>();
    assert!(
        !arenas.is_empty(),
        "seam {seam} {stage} did not observe the surviving destination arena"
    );
    assert!(
        arenas
            .iter()
            .all(|artifact| artifact["outcome"]["posture"] == "intact"),
        "seam {seam} {stage} damaged arena observation: {arenas:#?}"
    );
}

#[test]
#[ignore = "spawned and killed at a real arena-retirement boundary"]
fn arena_retirement_child() {
    let directory = PathBuf::from(std::env::var_os(MARKER).unwrap());
    let seam: u8 = std::env::var(SEAM).unwrap().parse().unwrap();
    let root = directory.join("store");
    let (format, _, access) = configuration();
    let placement = PhysicalRecordPlacementPolicy::builder()
        .arena_capacity(ExtentArenaCapacity::bytes(64 << 20).unwrap())
        .arena_evacuation(ArenaEvacuationThreshold::percent(99).unwrap())
        .admit(format)
        .unwrap();
    let media = crate::media(&root);
    let durability = if std::env::var_os(SMALL_WAL).is_some() {
        crate::durability_with_wal_policy(&media, small_wal_policy())
    } else {
        super::evacuation::fill_durability(&media)
    };
    let serving = crate::success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    );
    let payload = vec![61; 40_000];
    let append = completed(prepare(&serving, placement, [31; 32], &payload).execute());
    let identity = append.persisted_records()[0];
    let _record = append.into_acknowledgment().record_ids().next().unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([32; 32]))
        .unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request(key), identity)
        .unwrap()
    else {
        panic!("selected copy was not prepared");
    };
    for _ in 0..1000 {
        if progress.phase == PhysicalExtentCopyPhase::ReadyForAdoption {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert_eq!(progress.phase, PhysicalExtentCopyPhase::ReadyForAdoption);
    completed(
        submission
            .prepare_completed_extent_copy()
            .unwrap()
            .execute(),
    );
    for ordinal in [1, 2] {
        let checkpoint = super::evacuation::copy_checkpoint(&serving, ordinal);
        submission.finalize_extent_copy(&checkpoint).unwrap();
    }
    serving.retire_displaced_segment().unwrap();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([33; 32]))
        .unwrap();
    assert!(matches!(
        submission
            .prepare_arena_evacuation(placement, request(key))
            .unwrap(),
        PhysicalArenaEvacuationPreparationOutcome::EmptyAwaitingRetirement { arena: 1, .. }
    ));
    assert_eq!(current_extent_route(&root, identity).arena, 2);
    let arrived = serving.certification_arm_retirement_kill(seam);
    thread::spawn(move || {
        while !arrived.load(std::sync::atomic::Ordering::Acquire) {
            thread::sleep(Duration::from_millis(1));
        }
        std::fs::write(directory.join("reached"), b"1").unwrap();
    });
    let result = serving.retire_displaced_segment();
    panic!("armed arena retirement returned: {result:?}");
}

pub(super) fn kill_at_seam(directory: &Path, seam: u8) {
    kill_at_seam_with_policy(directory, seam, false);
}

pub(super) fn kill_at_seam_with_policy(directory: &Path, seam: u8, small_wal: bool) {
    let mut child = Command::new(std::env::current_exe().unwrap());
    child
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .env(MARKER, directory)
        .env(SEAM, seam.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if small_wal {
        child.env(SMALL_WAL, "1");
    }
    let mut child = child.spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    while !directory.join("reached").exists() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "seam {seam} not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
}

fn arena_file(root: &Path) -> PathBuf {
    root.join("families/records/arenas")
        .join(RecordArtifactFile::ExtentArena { arena: 1 }.file_name())
}
