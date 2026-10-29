pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalRetirementDenial, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::manifest_fixture::current_extent_route, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation, crate::durability_admission::managed_mutation::extent_record_rewrite::independent_arena_observer, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request};
pub(crate) use {std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};
pub(crate) use {worth_store::physical_runtime::ArenaEvacuationThreshold, worth_store::physical_runtime::ExtentArenaCapacity, worth_store::physical_runtime::PhysicalArenaEvacuationPreparationOutcome, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordPlacementPolicy};
pub(crate) use {worth_store_physical_format::RecordArtifactFile};

pub(crate) const CHILD: &str = "durability_admission::managed_mutation::extent_record_rewrite::arena_retirement_crash::arena_retirement_child";
pub(crate) const MARKER: &str = "C11_ARENA_RETIRE_MARKER";
pub(crate) const SEAM: &str = "C11_ARENA_RETIRE_SEAM";
pub(crate) const SMALL_WAL: &str = "C11_ARENA_RETIRE_SMALL_WAL";

pub(crate) fn small_wal_policy() -> worth_store::physical_runtime::PhysicalWalPolicy {
    use {std::num::NonZeroU32, std::num::NonZeroU64};
    use {worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};
    PhysicalWalPolicy::segmented(
        WalSegmentByteLimit::new(NonZeroU64::new(128 << 10).unwrap()),
        WalSegmentInventoryLimit::new(NonZeroU32::new(1024).unwrap()),
    )
}

pub(crate) fn observe_arena_report(
    root: &Path,
    directory: &Path,
    seam: u8,
    stage: &str,
) -> serde_json::Value {
    crate::durability_admission::managed_mutation::extent_record_rewrite::independent_arena_observer::observe_arena_in_separate_process(
        root,
        directory,
        "arena-retire",
        &format!("{seam}-{stage}"),
    )
}

pub(crate) fn assert_offline_arena_integrity(root: &Path, directory: &Path, seam: u8, stage: &str) {
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

pub(crate) fn kill_at_seam(directory: &Path, seam: u8) {
    kill_at_seam_with_policy(directory, seam, false);
}

pub(crate) fn kill_at_seam_with_policy(directory: &Path, seam: u8, small_wal: bool) {
    let mut child = std::process::Command::new(std::env::current_exe().unwrap());
    child
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .env(MARKER, directory)
        .env(SEAM, seam.to_string())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if small_wal {
        child.env(SMALL_WAL, "1");
    }
    let mut child = child.spawn().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !directory.join("reached").exists() {
        if child.try_wait().unwrap().is_some() || std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "seam {seam} not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
}

pub(crate) fn arena_file(root: &Path) -> PathBuf {
    root.join("families/records/arenas")
        .join(worth_store_physical_format::RecordArtifactFile::ExtentArena { arena: 1 }.file_name())
}