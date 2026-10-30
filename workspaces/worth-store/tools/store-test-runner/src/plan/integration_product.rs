//! Court selections generated from the applied court plan. Categories are prefixes.
use super::TestExecutionUnit;
use std::path::Path;

pub(super) fn scenario(shard: Option<(usize, usize)>, root: &Path) -> Vec<TestExecutionUnit> {
    units(
        "scenario",
        &["ordinary", "process", "determinism"],
        shard,
        root,
    )
}
pub(super) fn ui(shard: Option<(usize, usize)>, root: &Path) -> Vec<TestExecutionUnit> {
    units("ui", &["compile_fail"], shard, root)
}
pub(super) fn formal(shard: Option<(usize, usize)>, root: &Path) -> Vec<TestExecutionUnit> {
    units("formal", &["oracle"], shard, root)
}
fn units(
    lane: &str,
    categories: &[&str],
    shard: Option<(usize, usize)>,
    root: &Path,
) -> Vec<TestExecutionUnit> {
    let mut args: Vec<String> = [
        "nextest",
        "run",
        "--workspace",
        "--tests",
        "--no-fail-fast",
        "--no-tests=fail",
        "--filterset",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let mut filters = Vec::new();
    for &(package, binary, present) in COURTS {
        for &category in categories {
            if !present.contains(&category) {
                continue;
            }
            let prefix = match category {
                "ordinary" => "not (test(/^process::/) + test(/^oracle::/) + test(/^cost::/) + test(/^determinism::/) + test(/^compile_fail::/))".to_owned(),
                "process" | "oracle" | "cost" | "determinism" | "compile_fail" => format!("test(/^{category}::/)"),
                other => panic!("unregistered category {other}"),
            };
            filters.push(format!("(binary_id(={package}::{binary}) & ({prefix}))"));
        }
    }
    args.push(filters.join(" + "));
    if let Some((index, count)) = shard {
        args.extend([
            "--partition".into(),
            format!("hash:{}/{}", index + 1, count),
        ]);
    }
    vec![
        TestExecutionUnit::cargo(
            format!("{lane}::executables"),
            root,
            vec![
                "build".into(),
                "--bins".into(),
                "-p".into(),
                "store-test-runner".into(),
                "-p".into(),
                "worth-store".into(),
                "-p".into(),
                "worth-store-offline-integrity-observer".into(),
                "-p".into(),
                "worth-store-offline-verifier".into(),
                "-p".into(),
                "worth-store-recovery-runtime".into(),
            ],
        ),
        TestExecutionUnit::cargo(format!("{lane}::nextest"), root, args),
    ]
}
const COURTS: &[(&str, &str, &[&str])] = &[
    ("worth-store-court-source", "worth_store_court_source", &["process"]),
    ("worth-store-court-store", "worth_store_court_store", &["compile_fail", "ordinary", "process"]),
    ("worth-store-court-store-replication-physical-record-journeys-durability-admission", "worth_store_court_store_replication_physical_record_journeys_durability_admission", &["ordinary", "process"]),
    ("worth-store-court-store-replication-physical-record-journeys-physical-work", "worth_store_court_store_replication_physical_record_journeys_physical_work", &["compile_fail", "cost", "process"]),
    ("worth-store-court-store-replication-physical-blob-journeys", "worth_store_court_store_replication_physical_blob_journeys", &["compile_fail", "process"]),
    ("worth-store-court-store-replication", "worth_store_court_store_replication", &["process"]),
    ("worth-store-court-aspect-native", "worth_store_court_aspect_native", &["ordinary"]),
    ("worth-store-court-branch-deltas", "worth_store_court_branch_deltas", &["ordinary"]),
    ("worth-store-court-buffer-pool", "worth_store_court_buffer_pool", &["ordinary"]),
    ("worth-store-court-layout-indexes", "worth_store_court_layout_indexes", &["ordinary"]),
    ("worth-store-court-layout-indexes-replication", "worth_store_court_layout_indexes_replication", &["ordinary", "process"]),
    ("worth-store-court-maintenance-recovery-runtime-replication", "worth_store_court_maintenance_recovery_runtime_replication", &["compile_fail", "ordinary"]),
    ("worth-store-court-physical-format", "worth_store_court_physical_format", &["ordinary", "process"]),
    ("worth-store-court-physical-integrity", "worth_store_court_physical_integrity", &["ordinary"]),
    ("worth-store-court-physical-isolation", "worth_store_court_physical_isolation", &["ordinary"]),
    ("worth-store-court-recovery-runtime-replication", "worth_store_court_recovery_runtime_replication", &["ordinary", "process"]),
    ("worth-store-court-recovery-runtime-replication-phase-eight-process", "worth_store_court_recovery_runtime_replication_phase_eight_process", &["oracle", "ordinary", "process"]),
    ("worth-store-court-recovery-runtime-replication-production-entry-certified-release-serving", "worth_store_court_recovery_runtime_replication_production_entry_certified_release_serving", &["cost", "ordinary", "process"]),
    ("worth-store-court-recovery-runtime-replication-production-entry-accumulator-only-pending-wal", "worth_store_court_recovery_runtime_replication_production_entry_accumulator_only_pending_wal", &["ordinary", "process"]),
    ("worth-store-court-recovery-runtime-replication-production-entry-production-custody-serving", "worth_store_court_recovery_runtime_replication_production_entry_production_custody_serving", &["compile_fail", "oracle", "ordinary", "process"]),
    ("worth-store-court-recovery-runtime-replication-store-physical-blob-journeys", "worth_store_court_recovery_runtime_replication_store_physical_blob_journeys", &["process"]),
    ("worth-store-court-recovery-runtime-replication-store-physical-record-journeys-durability-admission", "worth_store_court_recovery_runtime_replication_store_physical_record_journeys_durability_admission", &["process"]),
    ("worth-store-court-recovery-runtime-replication-store-physical-record-journeys-record-chunk-views", "worth_store_court_recovery_runtime_replication_store_physical_record_journeys_record_chunk_views", &["process"]),
    ("worth-store-court-relational", "worth_store_court_relational", &["process"]),
    ("worth-store-court-snapshots", "worth_store_court_snapshots", &["ordinary"]),
];
