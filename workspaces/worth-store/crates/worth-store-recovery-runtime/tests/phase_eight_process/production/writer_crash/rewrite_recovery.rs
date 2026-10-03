use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    DurableFrameKind, PhysicalRewriteRedo, RecordArtifactFile, REWRITE_REDO_DOMAIN,
};
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
    let report = RecoveryReportEnvelope::decode(&fs::read(&report_path).expect("report bytes"))
        .expect("report decode");
    assert_eq!(
        report.outcome(),
        RecoveryReportOutcome::Recovered,
        "rewrite recovery denial: {:?}",
        report.denial_cause()
    );
    let page_lsn = destination_page_lsn(&world.writer.root, segment, rewrite);
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
    let rewrite = wal_rewrite(&root).expect("wal rewrite redo");
    let segment = source_segment(&root, rewrite);
    let report_path = parent.path().join("rewrite-multi-page-runtime-report.bin");
    let (_process_id, output) =
        run_recovery_with_profile(&root, &report_path, parent.path(), "c10-extent-rewrite-v1");
    assert_child_succeeded("rewrite-multi-page", &output);
    let report = RecoveryReportEnvelope::decode(&fs::read(&report_path).expect("report bytes"))
        .expect("report decode");
    assert_eq!(
        report.outcome(),
        RecoveryReportOutcome::Recovered,
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

fn wal_rewrite(root: &Path) -> Option<PhysicalRewriteRedo> {
    let domain = REWRITE_REDO_DOMAIN;
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
            for (index, _) in bytes
                .windows(domain.len())
                .enumerate()
                .filter(|(_, window)| *window == domain)
            {
                let Some(start) = index.checked_sub(8) else {
                    continue;
                };
                // The separately versioned v2 redo has a 280-byte fixed body.
                let Some(encoded) = bytes.get(start..start + 8 + domain.len() + 280) else {
                    continue;
                };
                if let Ok(redo) = PhysicalRewriteRedo::decode(encoded, u64::MAX) {
                    return Some(redo);
                }
            }
        }
    }
    None
}

fn source_segment(root: &Path, rewrite: PhysicalRewriteRedo) -> u64 {
    let directory = root.join("families").join("records").join("segments");
    let start = usize::try_from(rewrite.source_offset()).expect("source offset fits host");
    let end = start + rewrite.source_length() as usize;
    fs::read_dir(directory)
        .expect("source segment directory")
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let stem = name.strip_prefix("segment-")?.strip_suffix(".pages")?;
            let (segment, generation) = stem.split_once('-')?;
            if u64::from_str_radix(generation, 16).ok()? != rewrite.source_generation() {
                return None;
            }
            Some((u64::from_str_radix(segment, 16).ok()?, entry.path()))
        })
        .find_map(|(segment, path)| {
            let bytes = fs::read(path).ok()?;
            (Sha256::digest(bytes.get(start..end)?)[..] == rewrite.source_digest())
                .then_some(segment)
        })
        .expect("one source segment must match WAL rewrite generation and digest")
}

fn destination_page_lsn(root: &Path, segment: u64, rewrite: PhysicalRewriteRedo) -> u64 {
    const PAGE_BYTES: usize = 16 * 1024;
    let path = root.join("families/records/segments").join(
        RecordArtifactFile::Segment {
            segment,
            generation: rewrite.destination_generation(),
        }
        .file_name(),
    );
    let bytes = fs::read(path).expect("recovered destination segment");
    let start =
        usize::try_from(rewrite.destination_offset()).expect("destination offset fits host");
    let length = rewrite.destination_length() as usize;
    assert!(length >= PAGE_BYTES && length.is_multiple_of(PAGE_BYTES));
    let final_page = bytes
        .get(start + length - PAGE_BYTES..start + length)
        .expect("rewritten destination range must exist");
    worth_store_physical_format::decode_data_frame_page_lsn(
        final_page,
        DurableFrameKind::InlinePage,
    )
    .expect("recovered destination page frame")
    .get()
}
