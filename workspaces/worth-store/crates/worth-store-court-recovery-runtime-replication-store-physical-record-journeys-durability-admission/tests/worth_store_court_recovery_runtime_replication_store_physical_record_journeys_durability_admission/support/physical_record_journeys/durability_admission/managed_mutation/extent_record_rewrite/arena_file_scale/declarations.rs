pub(crate) use {worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::ServingPhysicalRuntime, worth_signal::facade::TemporalDuration, worth_proof::TransitionOutcome, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare_records, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_record, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request};
pub(crate) use {std::collections::HashSet};
pub(crate) use {std::num::NonZeroU32, std::num::NonZeroU64};
pub(crate) use {std::path::Path};
pub(crate) use {std::process::Command};
pub(crate) use {worth_store::physical_runtime::CheckpointMemoryLimit, worth_store::physical_runtime::ExtentArenaCapacity, worth_store::physical_runtime::GroupCommitDelay, worth_store::physical_runtime::GroupCommitLimit, worth_store::physical_runtime::IdempotencyRetentionGenerations, worth_store::physical_runtime::LiveIdempotencyBindingLimit, worth_store::physical_runtime::PendingUnresolvedMutationLimit, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointPolicy, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalDurabilityDeclaration, worth_store::physical_runtime::PhysicalIdempotencyPolicy, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest, worth_store::physical_runtime::RetainedWalTailLimit, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};

pub(crate) const WRITER: &str = "process::store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_file_scale::scale_writer_child";
pub(crate) const REOPENER: &str = "process::store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_file_scale::scale_reopener_child";
pub(crate) const DIRECTORY: &str = "C11_ARENA_SCALE_DIRECTORY";
pub(crate) const COUNT: &str = "C11_ARENA_SCALE_COUNT";
pub(crate) const CAPACITY: &str = "C11_ARENA_SCALE_CAPACITY";
pub(crate) const LARGE: usize = 256 * 1024;
pub(crate) const SMALL: usize = 20 * 1024;
pub(crate) const LARGE_BATCH: u64 = 16;
pub(crate) const SMALL_BATCH: u64 = 32;

pub(crate) fn run_scale_case(count: u64, capacity: u64) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    run_scale_child(WRITER, parent.path(), count, capacity);
    assert_file_count_and_namespace(&root, count, capacity);
    run_scale_child(REOPENER, parent.path(), count, capacity);
    assert_offline_population(&root, parent.path(), count);
}

pub(crate) fn run_scale_child(test: &str, directory: &Path, count: u64, capacity: u64) {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--ignored", "--nocapture"])
        .env(DIRECTORY, directory)
        .env(COUNT, count.to_string())
        .env(CAPACITY, capacity.to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{test} failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("running 1 test"),
        "{test} must select exactly one child test"
    );
}

pub(crate) fn case_from_env() -> (std::path::PathBuf, u64, u64) {
    let root = std::path::PathBuf::from(std::env::var_os(DIRECTORY).unwrap()).join("store");
    let count = std::env::var(COUNT).unwrap().parse().unwrap();
    let capacity = std::env::var(CAPACITY).unwrap().parse().unwrap();
    (root, count, capacity)
}

pub(crate) fn assert_file_count_and_namespace(root: &Path, count: u64, capacity: u64) {
    let arena_dir = root.join("families/records/arenas");
    let arena_files = std::fs::read_dir(&arena_dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    assert!(
        arena_files
            .iter()
            .all(|name| name.starts_with("arena-") && name.ends_with(".data")),
        "the arena directory contains only canonical packed data files"
    );
    let logical_bytes = count * (LARGE as u64 + SMALL as u64);
    let ceiling = logical_bytes.div_ceil(capacity) + 1;
    assert!(
        arena_files.len() as u64 <= ceiling,
        "{} arena files exceed the C11 byte-proportional bound {ceiling}",
        arena_files.len()
    );
    assert!(
        !root.join("families/records/extents").exists(),
        "no per-extent data or manifest family is created"
    );
    assert_no_per_extent_artifacts(root);
}

pub(crate) fn assert_offline_population(root: &Path, directory: &Path, count: u64) {
    let report = observe_offline_population(root, directory, count);
    assert_eq!(report["role"], "offline-root-observer");
    assert_eq!(report["completeness"], "complete");
    let artifacts = report["artifacts"].as_array().unwrap();
    let manifests = artifacts
        .iter()
        .filter(|artifact| {
            artifact["family"] == "extent_manifest"
                && artifact["identity"].as_str().is_some_and(|identity| {
                    identity.starts_with("arena:")
                        && identity.contains(":extent:")
                        && !identity.contains(":chunk:")
                })
        })
        .collect::<Vec<_>>();
    assert_eq!(manifests.len() as u64, count * 2);
    assert!(
        manifests
            .iter()
            .all(|artifact| artifact["outcome"]["posture"] == "intact"),
        "every reopened extent manifest is independently intact"
    );
    let accounting = artifacts
        .iter()
        .filter(|artifact| {
            artifact["family"] == "extent_arena_frame"
                && artifact["identity"]
                    .as_str()
                    .is_some_and(|identity| identity.starts_with("arena-accounting:"))
        })
        .collect::<Vec<_>>();
    assert!(!accounting.is_empty());
    assert!(
        accounting
            .iter()
            .all(|artifact| artifact["outcome"]["posture"] == "intact"),
        "reopened arena ranges have no offline overlap or accounting damage"
    );
}

pub(crate) fn observe_offline_population(
    root: &Path,
    directory: &Path,
    count: u64,
) -> serde_json::Value {
    let logical_bytes = count * (LARGE as u64 + SMALL as u64);
    // Current and previous roots each read the selected frames; WAL and
    // manifest admission need additional room beyond those two payload walks.
    let observed_byte_budget = logical_bytes * 4 + (4_u64 << 30);
    let executable = worth_store_test_support::OFFLINE_ROOT_OBSERVER
        .path()
        .expect("runner must prebuild the independent executable");
    let report_path = directory.join("arena-scale-observation.json");
    let output = std::process::Command::new(executable)
        .args(["observe", "--store-root"])
        .arg(root)
        .arg("--report")
        .arg(&report_path)
        .args(["--max-entries", "1000000", "--max-bytes"])
        .arg(observed_byte_budget.to_string())
        .args([
            "--max-open-files",
            "8",
            "--max-depth",
            "12",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            "1800000",
            "--max-report-bytes",
            "536870912",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "independent observer failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&std::fs::read(report_path).unwrap())
        .expect("independent observer report is valid JSON")
}

pub(crate) fn assert_no_per_extent_artifacts(root: &std::path::Path) {
    let mut directories = vec![root.to_owned()];
    let mut inspected = 0_u64;
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            inspected += 1;
            assert!(
                inspected <= 500_000,
                "capacity-lane namespace census is bounded"
            );
            let name = entry.file_name().into_string().unwrap();
            assert!(
                name != "extents"
                    && name != "extent-manifests"
                    && !name.starts_with("extent-")
                    && !name.starts_with("extent-manifest-"),
                "per-extent artifact survives outside packed arena family: {}",
                entry.path().display()
            );
            if entry.file_type().unwrap().is_dir() {
                directories.push(entry.path());
            }
        }
    }
}

pub(crate) fn checkpoint_and_bound_wal(serving: &ServingPhysicalRuntime, batch: u64) {
    let mut key = [219; 32];
    key[..8].copy_from_slice(&batch.to_le_bytes());
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(30_000).unwrap(),
        ),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("capacity-lane checkpoint {batch} was not admitted");
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    let wal = serving
        .certification_record_submission()
        .wal_observation()
        .unwrap();
    assert!(
        wal.active_segment_count() <= 40,
        "capacity-lane WAL must stay below 640 MiB after checkpoint {batch}"
    );
}

pub(crate) fn large_population_durability(
    media: &worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
) -> worth_store::physical_runtime::AdmittedPhysicalDurabilityPolicy {
    let admitted = worth_store::physical_runtime::PhysicalDurabilityDeclaration::builder()
        .group_commit(
            worth_store::physical_runtime::GroupCommitLimit::new(
                std::num::NonZeroU32::new(32).unwrap(),
            ),
            worth_store::physical_runtime::GroupCommitDelay::new(
                std::num::NonZeroU64::new(1).unwrap(),
            ),
        )
        .wal(worth_store::physical_runtime::PhysicalWalPolicy::segmented(
            worth_store::physical_runtime::WalSegmentByteLimit::new(
                std::num::NonZeroU64::new(16 << 20).unwrap(),
            ),
            worth_store::physical_runtime::WalSegmentInventoryLimit::new(
                std::num::NonZeroU32::new(128).unwrap(),
            ),
        ))
        .idempotency(
            worth_store::physical_runtime::PhysicalIdempotencyPolicy::new(
                worth_store::physical_runtime::IdempotencyRetentionGenerations::new(
                    std::num::NonZeroU64::new(4).unwrap(),
                ),
                worth_store::physical_runtime::PendingUnresolvedMutationLimit::new(
                    std::num::NonZeroU32::new(1024).unwrap(),
                ),
                worth_store::physical_runtime::LiveIdempotencyBindingLimit::new(
                    std::num::NonZeroU32::new(65_536).unwrap(),
                ),
            ),
        )
        .checkpoint(
            worth_store::physical_runtime::PhysicalCheckpointPolicy::fuzzy(
                worth_store::physical_runtime::CheckpointMemoryLimit::new(
                    std::num::NonZeroU64::new(64 << 20).unwrap(),
                ),
                worth_store::physical_runtime::RetainedWalTailLimit::new(
                    std::num::NonZeroU64::new(512_u64 << 20).unwrap(),
                ),
            ),
        )
        .admit(media.physical_durability_admission_basis().unwrap())
        .into_raw();
    let worth_proof::TransitionOutcome::Success(policy) = admitted else {
        panic!("the capacity lane durability declaration must admit");
    };
    policy
}
