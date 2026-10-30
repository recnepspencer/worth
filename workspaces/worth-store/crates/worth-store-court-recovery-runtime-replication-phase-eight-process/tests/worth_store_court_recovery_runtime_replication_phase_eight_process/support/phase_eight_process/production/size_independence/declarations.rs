pub(crate) use std::collections::BTreeMap;
pub(crate) use std::fs;
pub(crate) use std::path::Path;

pub(crate) use worth_store_offline_verifier::RecoveryObserverReport;
pub(crate) use {
    worth_store_recovery_runtime::RecoveryReportCounters,
    worth_store_recovery_runtime::RecoveryReportOutcome,
};

pub(crate) use crate::history;
pub(crate) use {
    crate::phase_eight_process::production::harness::ProcessWorld,
    crate::phase_eight_process::production::harness::RuntimeProcess,
    crate::production::harness::markers::RecoveryFateMarker,
};

pub(crate) const MINIMUM_STORE_SIZE_MULTIPLIER: u64 = 2;
pub(crate) const MAX_UNRELATED_RESIDUE_FILES: usize = 64;
pub(crate) const UNRELATED_RESIDUE_CHUNK_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SelectedRecoveryBasis {
    pub(crate) generation_links: (u64, [u8; 32]),
    pub(crate) selectors: (u64, u64, u64, Option<[u8; 16]>, Option<u64>, [u8; 32]),
    pub(crate) checkpoint: (
        u64,
        u64,
        Option<u64>,
        Option<u64>,
        Option<u64>,
        Option<u64>,
        [u8; 32],
    ),
    pub(crate) wal: (u64, u64, u64, u64, Option<u64>, Option<u64>, [u8; 32]),
    pub(crate) pages: (u64, Option<u64>, Option<u64>, [u8; 32]),
    pub(crate) manifests: (u64, u64, [u8; 32]),
}

pub(crate) fn selected_recovery_basis(report: &RecoveryObserverReport) -> SelectedRecoveryBasis {
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
pub(crate) struct RecoveryWorkEvidence {
    pub(crate) outcome: RecoveryReportOutcome,
    pub(crate) store: Option<[u8; 16]>,
    pub(crate) root_generation: Option<u64>,
    pub(crate) counters: RecoveryReportCounters,
    pub(crate) fate_counts: RecoveryFateMarker,
    pub(crate) indexed_fates: BTreeMap<[u8; 32], String>,
}

pub(crate) fn recovery_work(runtime: &RuntimeProcess) -> RecoveryWorkEvidence {
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

pub(crate) fn copy_persisted_store_root(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("create paired Store root");
    for entry in std::fs::read_dir(source).expect("read source Store root") {
        let entry = entry.expect("read source Store entry");
        let target = destination.join(entry.file_name());
        let file_type = entry.file_type().expect("read source Store entry type");
        if file_type.is_dir() {
            copy_persisted_store_root(&entry.path(), &target);
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), target).expect("copy persisted Store artifact");
        }
    }
}

pub(crate) fn add_unrelated_residue_bytes(root: &Path, baseline_bytes: u64) -> u64 {
    let residue = root.join("families").join("records").join("roots");
    std::fs::create_dir_all(&residue).expect("create unrelated persisted residue");
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
        std::fs::write(&target, &template).expect("write unrelated persisted Store bytes");
        added_bytes = added_bytes.saturating_add(template.len() as u64);
    }
    assert!(added_bytes >= baseline_bytes);
    added_bytes
}

pub(crate) fn persisted_store_bytes(path: &Path) -> u64 {
    let file_type = std::fs::symlink_metadata(path)
        .expect("read persisted Store metadata")
        .file_type();
    if file_type.is_file() {
        return if is_runtime_lock(path) {
            0
        } else {
            std::fs::metadata(path)
                .expect("read persisted Store file metadata")
                .len()
        };
    }
    if !file_type.is_dir() {
        return 0;
    }
    std::fs::read_dir(path)
        .expect("read persisted Store directory")
        .map(|entry| persisted_store_bytes(&entry.expect("read persisted Store entry").path()))
        .sum()
}

pub(crate) fn is_runtime_lock(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == "mutation.lock")
}
