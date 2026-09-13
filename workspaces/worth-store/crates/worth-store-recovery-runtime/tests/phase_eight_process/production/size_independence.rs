use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use worth_store_offline_verifier::RecoveryObserverReport;
use worth_store_recovery_runtime::{RecoveryReportCounters, RecoveryReportOutcome};

use super::super::history;
use super::harness::{ProcessWorld, RecoveryFateMarker, RuntimeProcess};

const MINIMUM_STORE_SIZE_MULTIPLIER: u64 = 2;
const MAX_UNRELATED_RESIDUE_FILES: usize = 64;
const UNRELATED_RESIDUE_CHUNK_BYTES: usize = 64 * 1024;

#[test]
fn recovery_work_is_independent_of_unrelated_persisted_store_size() {
    let world = ProcessWorld::start("candidate-publication", 0, 1);
    let small_root = world.writer.root.clone();
    let large_root = world.parent_path().join("size-independence-large-root");
    copy_persisted_store_root(&small_root, &large_root);

    let small_bytes = persisted_store_bytes(&small_root);
    let small_observer = world.observe_root(&small_root, "size-small-before-recovery");
    let small_fates = history::classify_persisted_fates(&world.writer.expected, &small_root)
        .expect("small persisted fate oracle");

    let added_bytes = add_unrelated_residue_bytes(&large_root, small_bytes);
    let large_bytes = persisted_store_bytes(&large_root);
    assert!(
        large_bytes >= small_bytes.saturating_mul(MINIMUM_STORE_SIZE_MULTIPLIER),
        "paired fixture must substantially increase unrelated Store bytes: small={small_bytes}, large={large_bytes}, added={added_bytes}"
    );

    let large_observer = world.observe_root(&large_root, "size-large-before-recovery");
    let large_fates = history::classify_persisted_fates(&world.writer.expected, &large_root)
        .expect("large persisted fate oracle");
    assert_eq!(
        small_fates, large_fates,
        "checkpoint/tail/damage scope changed"
    );
    assert_eq!(
        large_observer.report.residue_bytes(),
        small_observer
            .report
            .residue_bytes()
            .saturating_add(added_bytes),
        "independent observer did not see the planted persisted bytes as unrelated residue"
    );
    assert!(large_observer.report.bytes_read() > small_observer.report.bytes_read());
    assert_eq!(
        selected_recovery_basis(&small_observer.report),
        selected_recovery_basis(&large_observer.report),
        "selected checkpoint/tail basis changed with unrelated Store bytes"
    );

    let small_runtime = world.recover_root(&small_root, "size-small-recovery");
    let large_runtime = world.recover_root(&large_root, "size-large-recovery");
    assert_ne!(
        small_runtime.process_id, large_runtime.process_id,
        "paired recoveries must run in distinct physical_store_recover processes"
    );
    assert_ne!(small_runtime.marker.runtime, large_runtime.marker.runtime);
    assert_eq!(small_runtime.marker.store, large_runtime.marker.store);
    assert_eq!(
        small_runtime.marker.root_generation,
        large_runtime.marker.root_generation
    );
    assert_eq!(
        recovery_work(&small_runtime),
        recovery_work(&large_runtime),
        "recovery work evidence scaled with unrelated persisted Store bytes"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SelectedRecoveryBasis {
    generation_links: (u64, [u8; 32]),
    selectors: (u64, u64, u64, Option<[u8; 16]>, Option<u64>, [u8; 32]),
    checkpoint: (
        u64,
        u64,
        Option<u64>,
        Option<u64>,
        Option<u64>,
        Option<u64>,
        [u8; 32],
    ),
    wal: (u64, u64, u64, u64, Option<u64>, Option<u64>, [u8; 32]),
    pages: (u64, Option<u64>, Option<u64>, [u8; 32]),
    manifests: (u64, u64, [u8; 32]),
}

fn selected_recovery_basis(report: &RecoveryObserverReport) -> SelectedRecoveryBasis {
    SelectedRecoveryBasis {
        generation_links: (
            report.generation_link_count(),
            report.generation_link_digest(),
        ),
        selectors: (
            report.durable_selector_count(),
            report.linked_selector_count(),
            report.unpaired_selector_link_count(),
            report.selector_store_identity(),
            report.current_root_generation(),
            report.durable_selector_digest(),
        ),
        checkpoint: (
            report.checkpoint_count(),
            report.checkpoint_page_count(),
            report.checkpoint_covered_lsn_start(),
            report.checkpoint_covered_lsn_end(),
            report.checkpoint_redo_lsn(),
            report.durable_checkpoint_lsn(),
            report.checkpoint_coverage_digest(),
        ),
        wal: (
            report.wal_segment_count(),
            report.valid_wal_prefix_bytes(),
            report.observed_wal_bytes(),
            report.wal_frame_count(),
            report.wal_first_lsn(),
            report.wal_last_lsn(),
            report.valid_wal_prefix_digest(),
        ),
        pages: (
            report.page_lsn_count(),
            report.page_lsn_minimum(),
            report.page_lsn_maximum(),
            report.page_lsn_digest(),
        ),
        manifests: (
            report.manifest_count(),
            report.manifest_member_count(),
            report.manifest_membership_digest(),
        ),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct RecoveryWorkEvidence {
    outcome: RecoveryReportOutcome,
    store: Option<[u8; 16]>,
    root_generation: Option<u64>,
    counters: RecoveryReportCounters,
    fate_counts: RecoveryFateMarker,
    indexed_fates: BTreeMap<[u8; 32], String>,
}

fn recovery_work(runtime: &RuntimeProcess) -> RecoveryWorkEvidence {
    RecoveryWorkEvidence {
        outcome: runtime.report.outcome(),
        store: runtime.report.store_identity(),
        root_generation: runtime.report.root_generation(),
        counters: runtime.report.counters(),
        fate_counts: runtime.fates,
        indexed_fates: runtime
            .indexed_fates
            .iter()
            .map(|fate| (fate.idempotency, fate.fate.clone()))
            .collect(),
    }
}

fn copy_persisted_store_root(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create paired Store root");
    for entry in fs::read_dir(source).expect("read source Store root") {
        let entry = entry.expect("read source Store entry");
        let target = destination.join(entry.file_name());
        let file_type = entry.file_type().expect("read source Store entry type");
        if file_type.is_dir() {
            copy_persisted_store_root(&entry.path(), &target);
        } else if file_type.is_file() {
            fs::copy(entry.path(), target).expect("copy persisted Store artifact");
        }
    }
}

fn add_unrelated_residue_bytes(root: &Path, baseline_bytes: u64) -> u64 {
    let residue = root.join("families").join("records").join("roots");
    fs::create_dir_all(&residue).expect("create unrelated persisted residue");
    let template = (0..UNRELATED_RESIDUE_CHUNK_BYTES)
        .map(|index| (index as u8).wrapping_mul(31).wrapping_add(7))
        .collect::<Vec<_>>();
    let mut added_bytes = 0_u64;
    let mut ordinal = 1_u64;
    while baseline_bytes.saturating_add(added_bytes)
        < baseline_bytes.saturating_mul(MINIMUM_STORE_SIZE_MULTIPLIER)
    {
        assert!(
            ordinal <= MAX_UNRELATED_RESIDUE_FILES as u64,
            "bounded residue fixture could not build the larger Store twin"
        );
        let target = residue.join(format!("unrelated-{ordinal:016x}.residue"));
        ordinal = ordinal.saturating_add(1);
        fs::write(&target, &template).expect("write unrelated persisted Store bytes");
        added_bytes = added_bytes.saturating_add(template.len() as u64);
    }
    assert!(added_bytes >= baseline_bytes);
    added_bytes
}

fn persisted_store_bytes(path: &Path) -> u64 {
    let file_type = fs::symlink_metadata(path)
        .expect("read persisted Store metadata")
        .file_type();
    if file_type.is_file() {
        return if is_runtime_lock(path) {
            0
        } else {
            fs::metadata(path)
                .expect("read persisted Store file metadata")
                .len()
        };
    }
    if !file_type.is_dir() {
        return 0;
    }
    fs::read_dir(path)
        .expect("read persisted Store directory")
        .map(|entry| persisted_store_bytes(&entry.expect("read persisted Store entry").path()))
        .sum()
}

fn is_runtime_lock(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == "mutation.lock")
}
