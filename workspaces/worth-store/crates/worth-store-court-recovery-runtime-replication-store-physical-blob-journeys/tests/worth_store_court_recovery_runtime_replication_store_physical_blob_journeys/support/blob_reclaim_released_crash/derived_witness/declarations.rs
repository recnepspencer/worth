//! Read-only independent WAL oracle for the inline retirement that precedes
//! this released-drop crash. C8 must validate the whole page without routing
//! the retired directory identity into its recovered root.

pub(crate) use {std::fs, std::path::Path, std::path::PathBuf};

pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {worth_store_physical_format::decode_canonical_redo_v3, worth_store_physical_format::decode_inline_record, worth_store_physical_format::inspect_inline_page, worth_store_physical_format::BTreeNodeV1, worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DerivedFamilyRootDirectoryV1, worth_store_physical_format::PageGenerationCell, worth_store_physical_format::PersistedPhysicalDataFrameSubject, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::PhysicalRecoveryProjectionDecodeLimits, worth_store_physical_format::SlotGenerationCell};

pub(crate) const HEADER: usize = 116;
pub(crate) const FOOTER: usize = 32;

pub(crate) struct RetiredInlineWitness {
    pub(crate) record: PersistedRecordIdentity,
    pub(crate) page: PageGenerationCell,
    pub(crate) slot: SlotGenerationCell,
}

pub(crate) fn retired_inline_directory_slots(
    root: &Path,
    format: PhysicalRecordFormatDeclaration,
) -> Vec<RetiredInlineWitness> {
    let mut paths = Vec::new();
    collect(&root.join("families/wal"), &mut paths);
    paths.sort();
    let mut observed = Vec::new();
    for path in paths {
        let bytes = std::fs::read(path).unwrap();
        if !bytes.starts_with(b"WORTHWAL") {
            continue;
        }
        let mut offset = 0;
        while offset < bytes.len() {
            let header = bytes
                .get(offset..offset + HEADER)
                .expect("complete WAL header");
            assert_eq!(&header[..8], b"WORTHWAL");
            let length = usize::try_from(number(header, 44)).unwrap();
            let end = offset.checked_add(HEADER + length + FOOTER).unwrap();
            let frame = bytes.get(offset..end).expect("complete WAL member");
            if let Some(record) = inspect_member(frame, format) {
                observed.push(record);
            }
            offset = end;
        }
    }
    assert!(
        !observed.is_empty(),
        "real five-slot/four-live derived retirement"
    );
    observed
}

pub(crate) fn inspect_member(
    frame: &[u8],
    format: PhysicalRecordFormatDeclaration,
) -> Option<RetiredInlineWitness> {
    let mut payload = frame.get(HEADER..frame.len().checked_sub(FOOTER)?)?;
    if &frame[84..HEADER] != sha2::Sha256::digest(payload).as_slice()
        || &frame[frame.len() - FOOTER..]
            != sha2::Sha256::digest(&frame[..frame.len() - FOOTER]).as_slice()
    {
        return None;
    }
    let _binding = field(&mut payload)?;
    let redo = field(&mut payload)?;
    if !payload.is_empty() {
        return None;
    }
    let start = number(frame, 28);
    let (_, projection) = decode_canonical_redo_v3(
        redo,
        start,
        start.checked_add(1)?,
        128,
        None,
        PhysicalRecoveryProjectionDecodeLimits {
            frames: 128,
            record_identities: 128,
            placements: 128,
            segment_updates: 128,
            manifests: 128,
            total_entries: 512,
            inline_allocations: 128,
        },
        format,
    )
    .ok()?;
    let worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
        retirement: Some(retirement),
        ..
    } = projection.operation()
    else {
        return None;
    };
    for page_frame in projection.frames()? {
        let worth_store_physical_format::PersistedPhysicalDataFrameSubject::InlinePage(page) = page_frame.subject() else {
            continue;
        };
        let geometry = inspect_inline_page(format, page_frame.bytes()).ok()?;
        if geometry.slot_count() != 5 || geometry.page_cell() != page {
            continue;
        }
        let on_page = projection
            .placements()
            .iter()
            .filter_map(|placement| match placement {
                worth_store_physical_format::CurrentPhysicalRecordPlacement::Inline(value) if value.page_cell() == page => {
                    Some(value.record())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let retired = on_page
            .iter()
            .copied()
            .filter(|record| retirement.dropped_records().binary_search(record).is_ok())
            .collect::<Vec<_>>();
        if on_page.len() == 5 && retired.len() == 1 {
            assert_eq!(
                on_page
                    .iter()
                    .filter(|record| **record != retired[0])
                    .count(),
                4,
                "five physical slots, exactly four final-live routes"
            );
            let worth_store_physical_format::CurrentPhysicalRecordPlacement::Inline(witness) = projection
                .placements()
                .iter()
                .find(|placement| placement.record() == retired[0])?
            else {
                return None;
            };
            let (range, _) =
                decode_inline_record(page_frame.bytes(), retired[0], page, witness.slot_cell())
                    .ok()?;
            let content = &page_frame.bytes()[range.range()];
            if worth_store_physical_format::DerivedFamilyRootDirectoryV1::decode(content).is_err()
                && worth_store_physical_format::BTreeNodeV1::decode(content).is_err()
            {
                return None;
            }
            return Some(RetiredInlineWitness {
                record: retired[0],
                page,
                slot: witness.slot_cell(),
            });
        }
    }
    None
}

pub(crate) fn collect(directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, paths);
        } else {
            paths.push(path);
        }
    }
}

pub(crate) fn number(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

pub(crate) fn field<'a>(bytes: &mut &'a [u8]) -> Option<&'a [u8]> {
    let length = usize::try_from(u64::from_le_bytes(bytes.get(..8)?.try_into().ok()?)).ok()?;
    *bytes = bytes.get(8..)?;
    let (head, tail) = bytes.split_at_checked(length)?;
    *bytes = tail;
    Some(head)
}