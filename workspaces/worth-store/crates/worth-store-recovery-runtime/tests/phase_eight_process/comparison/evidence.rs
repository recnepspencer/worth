use worth_store_offline_verifier::RecoveryObserverReport;

pub(super) fn same_physical_evidence(
    expected: &RecoveryObserverReport,
    observed: &RecoveryObserverReport,
) -> bool {
    physical_evidence_key(expected) == physical_evidence_key(observed)
}

fn physical_evidence_key(
    report: &RecoveryObserverReport,
) -> (
    (u64, u64, u64, [u8; 32], u64),
    (u64, u64, u64, Option<[u8; 16]>, Option<u64>, [u8; 32]),
    (
        u64,
        u64,
        Option<u64>,
        Option<u64>,
        Option<u64>,
        Option<u64>,
        [u8; 32],
    ),
    (u64, u64, u64, u64, Option<u64>, Option<u64>, [u8; 32]),
    (u64, Option<u64>, Option<u64>, [u8; 32]),
    (u64, u64, [u8; 32], u64, u64, [u8; 32]),
) {
    (
        (
            report.artifact_count(),
            report.bytes_read(),
            report.generation_link_count(),
            report.generation_link_digest(),
            report.artifact_identity_count(),
        ),
        (
            report.durable_selector_count(),
            report.linked_selector_count(),
            report.unpaired_selector_link_count(),
            report.selector_store_identity(),
            report.current_root_generation(),
            report.durable_selector_digest(),
        ),
        (
            report.checkpoint_count(),
            report.checkpoint_page_count(),
            report.checkpoint_covered_lsn_start(),
            report.checkpoint_covered_lsn_end(),
            report.checkpoint_redo_lsn(),
            report.durable_checkpoint_lsn(),
            report.checkpoint_coverage_digest(),
        ),
        (
            report.wal_segment_count(),
            report.valid_wal_prefix_bytes(),
            report.observed_wal_bytes(),
            report.wal_frame_count(),
            report.wal_first_lsn(),
            report.wal_last_lsn(),
            report.valid_wal_prefix_digest(),
        ),
        (
            report.page_lsn_count(),
            report.page_lsn_minimum(),
            report.page_lsn_maximum(),
            report.page_lsn_digest(),
        ),
        (
            report.manifest_count(),
            report.manifest_member_count(),
            report.manifest_membership_digest(),
            report.residue_artifact_count(),
            report.residue_bytes(),
            report.residue_digest(),
        ),
    )
}
