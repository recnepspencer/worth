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
    let published = after.difference(&before).next().expect(
        "recovery must publish the rewritten segment generation",
    );
    let page = fs::read(
        world
            .writer
            .root
            .join("families")
            .join("records")
            .join("segments")
            .join(published),
    )
    .expect("recovered segment bytes");
    let page_lsn = worth_store_physical_format::decode_data_frame_page_lsn(
        &page,
        worth_store_physical_format::DurableFrameKind::InlinePage,
    )
    .expect("recovered page frame")
    .get();
    let rewrite = wal_rewrite(&world.writer.root).expect("wal rewrite redo");
    assert_eq!(page_lsn, rewrite.page_lsn());
}

#[test]
fn fresh_recovery_applies_a_multi_page_source_rewrite() {
    let parent = tempfile::tempdir().expect("multi-page rewrite parent");
    let root = super::super::super::history::launch_multi_page_rewrite_root(
        parent.path(),
        0xC8_09_00_14,
        0xC8_19_00_14,
    )
    .expect("production writer must leave a killed multi-page rewrite");
    let before = segment_files(&root);
    let report_path = parent.path().join("rewrite-multi-page-runtime-report.bin");
    let (_process_id, output) = run_recovery_with_profile(
        &root,
        &report_path,
        parent.path(),
        "c8-phase2-admission-v1",
    );
    assert_child_succeeded("rewrite-multi-page", &output);
    let report = RecoveryReportEnvelope::decode(&fs::read(&report_path).expect("report bytes"))
        .expect("report decode");
    assert_eq!(
        report.outcome(),
        RecoveryReportOutcome::Recovered,
        "multi-page rewrite recovery denial: {:?}",
        report.denial_cause()
    );
    let published = segment_files(&root).difference(&before).next().expect(
        "recovery must publish the rewritten segment generation",
    ).clone();
    let page = fs::read(
        root.join("families")
            .join("records")
            .join("segments")
            .join(&published),
    )
    .expect("recovered segment bytes");
    let page_lsn = worth_store_physical_format::decode_data_frame_page_lsn(
        &page,
        worth_store_physical_format::DurableFrameKind::InlinePage,
    )
    .expect("recovered page frame")
    .get();
    let rewrite = wal_rewrite(&root).expect("wal rewrite redo");
    assert_ne!(
        rewrite.source_offset(),
        0,
        "multi-page source rewrite must read past the first page"
    );
    assert_eq!(page_lsn, rewrite.page_lsn());
}

fn wal_rewrite(root: &Path) -> Option<worth_store_physical_format::PhysicalRewriteRedo> {
    let domain = worth_store_physical_format::REWRITE_REDO_DOMAIN;
    let mut stack = vec![root.join("families").join("wal")];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Ok(bytes) = fs::read(&path) else {
                continue;
            };
            let Some(index) = bytes.windows(domain.len()).position(|window| window == domain) else {
                continue;
            };
            let start = index.checked_sub(8)?;
            let end = start + 8 + domain.len() + 216;
            let encoded = bytes.get(start..end)?;
            if let Ok(redo) = worth_store_physical_format::PhysicalRewriteRedo::decode(encoded, u64::MAX)
            {
                return Some(redo);
            }
        }
    }
    None
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
