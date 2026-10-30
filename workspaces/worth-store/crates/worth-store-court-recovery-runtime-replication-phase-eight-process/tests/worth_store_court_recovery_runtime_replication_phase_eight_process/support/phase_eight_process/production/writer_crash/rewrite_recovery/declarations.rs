pub(crate) use std::fs;
pub(crate) use std::path::Path;

pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {
    worth_store_physical_format::DurableFrameKind,
    worth_store_physical_format::PhysicalRewriteRedo,
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::REWRITE_REDO_DOMAIN,
};
pub(crate) use {
    worth_store_recovery_runtime::RecoveryReportEnvelope,
    worth_store_recovery_runtime::RecoveryReportOutcome,
};

pub(crate) use {
    crate::history::schedule::in_flight_mutation::MutationCrashWorkload,
    crate::phase_eight_process::production::harness::process::assert_child_succeeded,
    crate::phase_eight_process::production::harness::process::run_recovery_with_profile,
    crate::phase_eight_process::production::harness::ProcessWorld,
};

pub(crate) fn wal_rewrite(root: &Path) -> Option<PhysicalRewriteRedo> {
    let domain = REWRITE_REDO_DOMAIN;
    let mut stack = vec![root.join("families").join("wal")];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Ok(bytes) = std::fs::read(&path) else {
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
                if let Ok(redo) =
                    worth_store_physical_format::PhysicalRewriteRedo::decode(encoded, u64::MAX)
                {
                    return Some(redo);
                }
            }
        }
    }
    None
}

pub(crate) fn source_segment(root: &Path, rewrite: PhysicalRewriteRedo) -> u64 {
    let directory = root.join("families").join("records").join("segments");
    let start = usize::try_from(rewrite.source_offset()).expect("source offset fits host");
    let end = start + rewrite.source_length() as usize;
    std::fs::read_dir(directory)
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
            let bytes = std::fs::read(path).ok()?;
            (sha2::Sha256::digest(bytes.get(start..end)?)[..] == rewrite.source_digest())
                .then_some(segment)
        })
        .expect("one source segment must match WAL rewrite generation and digest")
}

pub(crate) fn destination_page_lsn(root: &Path, segment: u64, rewrite: PhysicalRewriteRedo) -> u64 {
    const PAGE_BYTES: usize = 16 * 1024;
    let path = root.join("families/records/segments").join(
        worth_store_physical_format::RecordArtifactFile::Segment {
            segment,
            generation: rewrite.destination_generation(),
        }
        .file_name(),
    );
    let bytes = std::fs::read(path).expect("recovered destination segment");
    let start =
        usize::try_from(rewrite.destination_offset()).expect("destination offset fits host");
    let length = rewrite.destination_length() as usize;
    assert!(length >= PAGE_BYTES && length.is_multiple_of(PAGE_BYTES));
    let final_page = bytes
        .get(start + length - PAGE_BYTES..start + length)
        .expect("rewritten destination range must exist");
    worth_store_physical_format::decode_data_frame_page_lsn(
        final_page,
        worth_store_physical_format::DurableFrameKind::InlinePage,
    )
    .expect("recovered destination page frame")
    .get()
}
