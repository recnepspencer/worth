use std::fs;
use std::path::Path;

use worth_store_recovery_runtime::{RecoveryReportEnvelope, RecoveryReportOutcome};

use super::super::harness::{
    assert_child_succeeded, run_recovery_with_profile, MutationCrashWorkload, ProcessWorld,
};

#[test]
fn fresh_recovery_applies_a_wal_durable_segment_rewrite() {
    let world = ProcessWorld::start_mutation_crash(
        "after-wal-durability",
        MutationCrashWorkload::SelectedSegmentRewrite,
        0xC8_09_00_13,
        0xC8_19_00_13,
    );
    let before = segment_files(&world.writer.root);
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
    let report = RecoveryReportEnvelope::decode(&fs::read(&report_path).expect("report bytes"))
        .expect("report decode");
    assert_eq!(
        report.outcome(),
        RecoveryReportOutcome::Recovered,
        "rewrite recovery denial: {:?}",
        report.denial_cause()
    );
    let after = segment_files(&world.writer.root);
    assert!(
        after.difference(&before).next().is_some(),
        "recovery must publish the rewritten segment generation"
    );
}

fn segment_files(root: &Path) -> std::collections::BTreeSet<String> {
    let directory = root.join("families").join("records").join("segments");
    let mut names = std::collections::BTreeSet::new();
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                names.insert(name.to_owned());
            }
        }
    }
    names
}
