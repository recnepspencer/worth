use super::*;
use std::collections::HashSet;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;
use std::process::Command;
use worth_store::physical_runtime::{
    CheckpointMemoryLimit, ExtentArenaCapacity, GroupCommitDelay, GroupCommitLimit,
    IdempotencyRetentionGenerations, LiveIdempotencyBindingLimit, PendingUnresolvedMutationLimit,
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointPolicy, PhysicalCheckpointRequest, PhysicalDurabilityDeclaration,
    PhysicalIdempotencyPolicy, PhysicalRecordInitialization, PhysicalRecordOpen,
    PhysicalRecordPlacementPolicy, PhysicalWalPolicy, RecordCountLimit, RecordScanOutcome,
    RecordScanRequest, RetainedWalTailLimit, WalSegmentByteLimit, WalSegmentInventoryLimit,
};

const WRITER: &str = "durability_admission::managed_mutation::extent_record_rewrite::arena_file_scale::scale_writer_child";
const REOPENER: &str = "durability_admission::managed_mutation::extent_record_rewrite::arena_file_scale::scale_reopener_child";
const DIRECTORY: &str = "C11_ARENA_SCALE_DIRECTORY";
const COUNT: &str = "C11_ARENA_SCALE_COUNT";
const CAPACITY: &str = "C11_ARENA_SCALE_CAPACITY";
const OBSERVER: &str = "WORTH_C9_OBSERVER_EXECUTABLE";
const LARGE: usize = 256 * 1024;
const SMALL: usize = 20 * 1024;
const LARGE_BATCH: u64 = 16;
const SMALL_BATCH: u64 = 32;

/// Exact C11 file-count population: 32,768 records and over 4 GiB of logical
/// payload, requiring multiple GiB of disk and a long certification window.
#[test]
#[ignore = "32,768 real records and >4 GiB payload; run in the C11 capacity lane"]
fn thirty_two_thousand_extents_use_arena_files_proportional_to_bytes() {
    run_scale_case(16_384, 1 << 30);
}

#[test]
fn small_population_reopens_in_another_process_and_agrees_with_offline_media() {
    run_scale_case(33, 64 << 20);
}

fn run_scale_case(count: u64, capacity: u64) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    run_scale_child(WRITER, parent.path(), count, capacity);
    assert_file_count_and_namespace(&root, count, capacity);
    run_scale_child(REOPENER, parent.path(), count, capacity);
    assert_offline_population(&root, parent.path(), count);
}

fn run_scale_child(test: &str, directory: &Path, count: u64, capacity: u64) {
    let output = Command::new(std::env::current_exe().unwrap())
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

fn case_from_env() -> (std::path::PathBuf, u64, u64) {
    let root = std::path::PathBuf::from(std::env::var_os(DIRECTORY).unwrap()).join("store");
    let count = std::env::var(COUNT).unwrap().parse().unwrap();
    let capacity = std::env::var(CAPACITY).unwrap().parse().unwrap();
    (root, count, capacity)
}

#[test]
#[ignore = "the parent starts this writer in a distinct process"]
fn scale_writer_child() {
    let (root, count, capacity) = case_from_env();
    let (format, _, access) = configuration();
    let placement = PhysicalRecordPlacementPolicy::builder()
        .arena_capacity(ExtentArenaCapacity::bytes(capacity).unwrap())
        // Each batch reserves its arena ranges before root publication. The
        // default 64 KiB index has 15 slots shared by free ranges and claims,
        // with admission headroom reserved.
        .arena_index_bytes(RecordByteLimit::new(256 * 1024).unwrap())
        .admit(format)
        .unwrap();
    let media = crate::media(&root);
    let durability = large_population_durability(&media);
    let serving = crate::success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    );
    let mut published_records = 0_u64;
    let mut batch_number = 0_u64;
    let mut checkpoints = 0_u64;
    let mut identities = HashSet::new();
    for (phase, bytes, batch_width) in [(0_u64, LARGE, LARGE_BATCH), (1, SMALL, SMALL_BATCH)] {
        let payload = vec![phase as u8 + 47; bytes];
        for start in (0..count).step_by(batch_width as usize) {
            let batch_size = (count - start).min(batch_width);
            batch_number += 1;
            let mut key = [43; 32];
            key[..8].copy_from_slice(&batch_number.to_le_bytes());
            let records = vec![payload.as_slice(); batch_size as usize];
            let completed =
                completed(prepare_records(&serving, placement, key, &records).execute());
            assert_eq!(completed.persisted_records().len(), batch_size as usize);
            for record in completed.persisted_records() {
                assert!(
                    identities.insert(*record),
                    "batch reused a physical record identity"
                );
            }
            let previous = published_records;
            published_records += batch_size;
            for checkpoint in (previous / 256 + 1)..=(published_records / 256) {
                checkpoint_and_bound_wal(&serving, checkpoint);
                checkpoints += 1;
            }
        }
    }
    assert_eq!(published_records, count * 2);
    assert_eq!(identities.len() as u64, published_records);
    assert_eq!(checkpoints, published_records / 256);
    serving.close();
}

fn assert_file_count_and_namespace(root: &Path, count: u64, capacity: u64) {
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
    assert_no_per_extent_artifacts(&root);
}

#[test]
#[ignore = "the parent starts this reopener in a distinct process"]
fn scale_reopener_child() {
    let (root, count, _) = case_from_env();
    let (format, _, access) = configuration();
    let media = crate::media(&root);
    let durability = large_population_durability(&media);
    let serving = crate::success(
        media.open_record_store(PhysicalRecordOpen::new(format, access, durability)),
    );
    let mut scan = serving
        .records()
        .unwrap()
        .scan(RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(32).unwrap()))
        .unwrap();
    let mut scratch = vec![0_u8; 64 * 1024];
    let mut large = 0_u64;
    let mut small = 0_u64;
    let mut identities = HashSet::new();
    loop {
        match scan.read_next_into(&mut scratch).unwrap() {
            RecordScanOutcome::Batch(batch) => {
                for record in batch.records() {
                    assert!(
                        identities.insert(record.record_id()),
                        "fresh-process scan repeated a physical record identity"
                    );
                    let (seen, expected, value) = match record.declared_payload_bytes() {
                        n if n == LARGE as u64 => (&mut large, LARGE, 47),
                        n if n == SMALL as u64 => (&mut small, SMALL, 48),
                        n => panic!("unexpected reopened record length {n}"),
                    };
                    *seen += 1;
                    if *seen == 1 || *seen == count / 2 || *seen == count {
                        let reader = serving.records().unwrap();
                        let session = reader
                            .open(
                                record.record_id(),
                                RecordReadLimits::new(
                                    RecordByteLimit::new(expected as u32).unwrap(),
                                ),
                            )
                            .unwrap();
                        assert_eq!(
                            crate::read_record(session, expected).0,
                            vec![value; expected]
                        );
                    }
                }
            }
            RecordScanOutcome::Completed(_) => break,
        }
    }
    assert_eq!((large, small), (count, count));
    assert_eq!(identities.len() as u64, count * 2);
    drop(scan);
    serving.close();
}

fn assert_offline_population(root: &Path, directory: &Path, count: u64) {
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

fn observe_offline_population(root: &Path, directory: &Path, count: u64) -> serde_json::Value {
    let logical_bytes = count * (LARGE as u64 + SMALL as u64);
    // Current and previous roots each read the selected frames; WAL and
    // manifest admission need additional room beyond those two payload walks.
    let observed_byte_budget = logical_bytes * 4 + (4_u64 << 30);
    let executable = std::env::var_os(OBSERVER)
        .unwrap_or_else(|| panic!("{OBSERVER} must name the Cargo-built independent observer"));
    let report_path = directory.join("arena-scale-observation.json");
    let output = Command::new(executable)
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

fn assert_no_per_extent_artifacts(root: &std::path::Path) {
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

fn checkpoint_and_bound_wal(serving: &ServingPhysicalRuntime, batch: u64) {
    let mut key = [219; 32];
    key[..8].copy_from_slice(&batch.to_le_bytes());
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(30_000).unwrap(),
        ),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("capacity-lane checkpoint {batch} was not admitted");
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
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

fn large_population_durability(
    media: &worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
) -> worth_store::physical_runtime::AdmittedPhysicalDurabilityPolicy {
    let admitted = PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(16 << 20).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(128).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1024).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(65_536).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(64 << 20).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(512_u64 << 20).unwrap()),
        ))
        .admit(media.physical_durability_admission_basis().unwrap())
        .into_raw();
    let TransitionOutcome::Success(policy) = admitted else {
        panic!("the capacity lane durability declaration must admit");
    };
    policy
}
