pub(crate) use {worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, crate::durability_admission::managed_mutation::extent_record_rewrite::independent_arena_observer, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
pub(crate) use {std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};
pub(crate) use {worth_store::physical_runtime::PhysicalExtentCopyPhase};

pub(crate) const CHILD: &str = "process::store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_copy_crash::copy_bytes_before_final_child";
pub(crate) const DIRECTORY: &str = "C11_COPY_CRASH_DIRECTORY";
pub(crate) const SEAM: &str = "C11_COPY_CRASH_SEAM";

pub(crate) fn verify_killed_copy(seam: &str) {
    use worth_store_testkit_store::durability_admission::independent_wal_oracle::copy_intent::produced_copy_intents;
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
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
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

pub(crate) fn observe_after_kill(root: &Path, directory: &Path) -> serde_json::Value {
    crate::durability_admission::managed_mutation::extent_record_rewrite::independent_arena_observer::observe_arena_in_separate_process(
        root,
        directory,
        "copy-crash",
        "killed",
    )
}

pub(crate) fn kill_after_copy_bytes(directory: &Path, seam: &str) {
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .env(DIRECTORY, directory)
        .env(SEAM, seam)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !directory.join("reached").exists() {
        if child.try_wait().unwrap().is_some() || std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "copy child failed to reach crash seam: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
}
