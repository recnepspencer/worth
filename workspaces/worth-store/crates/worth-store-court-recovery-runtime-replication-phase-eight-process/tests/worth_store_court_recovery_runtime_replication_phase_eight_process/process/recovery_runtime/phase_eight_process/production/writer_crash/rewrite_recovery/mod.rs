#[allow(unused_imports)]
use crate::phase_eight_process::production::writer_crash::rewrite_recovery::*;

use {
    crate::process::support::production_harness::assert_child_succeeded,
    crate::process::support::production_harness::run_recovery_with_profile,
};

#[test]
fn fresh_recovery_applies_a_wal_durable_segment_rewrite() {
    let world = crate::phase_eight_process::production::harness::ProcessWorld::start_mutation_crash(
        "after-wal-durability",
        crate::history::schedule::in_flight_mutation::MutationCrashWorkload::SelectedSegmentRewrite,
        0xC8_09_00_13,
        0xC8_19_00_13,
    );
    let rewrite = wal_rewrite(&world.writer.root).expect("wal rewrite redo");
    let segment = source_segment(&world.writer.root, rewrite);
    let temporary = world
        .writer
        .root
        .parent()
        .expect("writer root parent")
        .to_path_buf();
    let report_path = temporary.join("rewrite-after-wal-runtime-report.bin");
    let (_process_id, output) = run_recovery_with_profile(
        &world.writer.root,
        &report_path,
        &temporary,
        "c8-phase2-admission-v1",
    );
    assert_child_succeeded("rewrite-after-wal", &output);
    let report = worth_store_recovery_runtime::RecoveryReportEnvelope::decode(
        &std::fs::read(&report_path).expect("report bytes"),
    )
    .expect("report decode");
    assert_eq!(
        report.outcome(),
        worth_store_recovery_runtime::RecoveryReportOutcome::Recovered,
        "rewrite recovery denial: {:?}",
        report.denial_cause()
    );
    let page_lsn = destination_page_lsn(&world.writer.root, segment, rewrite);
    assert_eq!(page_lsn, rewrite.page_lsn());
}

#[test]
fn fresh_recovery_applies_a_multi_page_source_rewrite() {
    let parent = tempfile::tempdir().expect("multi-page rewrite parent");
    let root = crate::process::support::history::mutation_writer::launch_multi_page_rewrite_root(
        parent.path(),
        0xC8_09_00_14,
        0xC8_19_00_14,
    )
    .expect("production writer must leave a killed multi-page rewrite");
    let rewrite = wal_rewrite(&root).expect("wal rewrite redo");
    let segment = source_segment(&root, rewrite);
    let report_path = parent.path().join("rewrite-multi-page-runtime-report.bin");
    let (_process_id, output) =
        run_recovery_with_profile(&root, &report_path, parent.path(), "c10-extent-rewrite-v1");
    assert_child_succeeded("rewrite-multi-page", &output);
    let report = worth_store_recovery_runtime::RecoveryReportEnvelope::decode(
        &std::fs::read(&report_path).expect("report bytes"),
    )
    .expect("report decode");
    assert_eq!(
        report.outcome(),
        worth_store_recovery_runtime::RecoveryReportOutcome::Recovered,
        "multi-page rewrite recovery denial: {:?}",
        report.denial_cause()
    );
    let page_lsn = destination_page_lsn(&root, segment, rewrite);
    assert_ne!(
        rewrite.source_offset(),
        0,
        "multi-page source rewrite must read past the first page"
    );
    assert_eq!(page_lsn, rewrite.page_lsn());
}
