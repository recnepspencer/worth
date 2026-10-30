pub(crate) use std::path::Path;

pub(crate) struct OfflineCompletion {
    pub(crate) root_generation: u64,
    pub(crate) records: usize,
    pub(crate) payload_digest: String,
}

pub(crate) struct WriterCompletion {
    pub(crate) root_generation: u64,
    pub(crate) positioned_writes: u64,
    pub(crate) file_barriers: u64,
    pub(crate) catalog_replacements: u64,
    pub(crate) directory_barriers: u64,
}

pub(crate) fn writer_completion(stdout: &str) -> WriterCompletion {
    let fields = completion_fields(stdout, "C5_COURTROOM_WRITER ");
    assert_eq!(fields.len(), 6);
    WriterCompletion {
        root_generation: fields[0].parse().unwrap(),
        positioned_writes: fields[2].parse().unwrap(),
        file_barriers: fields[3].parse().unwrap(),
        catalog_replacements: fields[4].parse().unwrap(),
        directory_barriers: fields[5].parse().unwrap(),
    }
}

/// Catalog replacements, non-journal directory barriers, and recovery-journal
/// directory barriers of each publication, in workload order.
pub(crate) fn publication_barriers(stdout: &str) -> Vec<(u64, u64, u64)> {
    stdout
        .lines()
        .filter_map(|line| line.strip_prefix("C5_COURTROOM_PUBLICATION "))
        .map(|fields| {
            let mut fields = fields
                .split_whitespace()
                .map(|field| field.parse().unwrap());
            let mut next = || fields.next().unwrap();
            (next(), next(), next())
        })
        .collect()
}

pub(crate) struct ReopenerCompletion {
    pub(crate) records: usize,
    pub(crate) deferred_records: usize,
    pub(crate) root_generation: u64,
    pub(crate) point_digest: String,
    pub(crate) scan_digest: String,
}

pub(crate) fn reopener_completion(stdout: &str) -> ReopenerCompletion {
    let fields = completion_fields(stdout, "C5_COURTROOM_REOPEN ");
    assert_eq!(fields.len(), 8);
    ReopenerCompletion {
        records: fields[0].parse().unwrap(),
        deferred_records: fields[1].parse().unwrap(),
        root_generation: fields[2].parse().unwrap(),
        point_digest: fields[3].into(),
        scan_digest: fields[4].into(),
    }
}

pub(crate) fn offline_completion(stdout: &str) -> OfflineCompletion {
    let fields = completion_fields(stdout, "C5_OFFLINE ");
    assert_eq!(fields.len(), 10);
    OfflineCompletion {
        root_generation: fields[1].parse().unwrap(),
        records: fields[2].parse().unwrap(),
        payload_digest: fields[9].to_owned(),
    }
}

pub(crate) fn completion_fields<'output>(stdout: &'output str, prefix: &str) -> Vec<&'output str> {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .unwrap_or_else(|| panic!("child output omitted `{prefix}` completion"))
        .split_whitespace()
        .collect()
}

pub(crate) fn locator_identities(path: &Path) -> Vec<String> {
    let mut identities = std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|locator| locator[32..].to_owned())
        .collect::<Vec<_>>();
    identities.sort();
    identities
}

pub(crate) fn placement_identities(
    walk: &worth_store_offline_verifier::OfflineDurableManifestWalk,
) -> Vec<String> {
    let mut identities = walk
        .placements()
        .iter()
        .map(|placement| {
            let record = placement.record();
            format!(
                "{}{}",
                hex(&record.allocation_epoch()),
                hex(&record.ordinal().to_le_bytes())
            )
        })
        .collect::<Vec<_>>();
    identities.sort();
    identities
}

#[derive(Clone, Copy)]
pub(crate) struct InlinePlacement {
    pub(crate) page: u64,
    pub(crate) page_generation: u64,
    pub(crate) slot: u16,
}

pub(crate) fn inline_placement(
    walk: &worth_store_offline_verifier::OfflineDurableManifestWalk,
    identity: ([u8; 16], u64),
) -> InlinePlacement {
    let placement = walk
        .placements()
        .iter()
        .find(|placement| {
            placement.record().allocation_epoch() == identity.0
                && placement.record().ordinal() == identity.1
        })
        .unwrap();
    match placement {
        worth_store_offline_verifier::OfflineRecordPlacement::Inline {
            page,
            page_generation,
            slot,
            ..
        } => InlinePlacement {
            page: *page,
            page_generation: *page_generation,
            slot: *slot,
        },
        worth_store_offline_verifier::OfflineRecordPlacement::Extent { .. } => {
            panic!("first courtroom record must remain inline")
        }
    }
}

pub(crate) fn segment_files(root: &Path) -> usize {
    std::fs::read_dir(root.join("families/records/segments"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|value| value == "pages")
        })
        .count()
}

pub(crate) fn segment_page_bytes(root: &Path) -> u64 {
    std::fs::read_dir(root.join("families/records/segments"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|value| value == "pages")
        })
        .map(|entry| entry.metadata().unwrap().len())
        .sum()
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
