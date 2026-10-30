//! Inspect genuine current-format copy and blob WAL members without rewriting media.
pub(crate) use {std::fs, std::path::Path, std::path::PathBuf};
pub(crate) use {
    worth_store_physical_format::PersistedPhysicalRecoveryOperation,
    worth_store_physical_format::PersistedPhysicalRecoveryPayload,
    worth_store_physical_format::PersistedPhysicalRecoveryProjection,
    worth_store_physical_format::PhysicalExtentCopyRecord,
    worth_store_physical_format::PhysicalExtentCopyResolutionKind,
    worth_store_physical_format::PhysicalRecordFormatDeclaration,
    worth_store_physical_format::PhysicalRecoveryProjectionDecodeLimits,
};

pub(crate) const HEADER: usize = 116;
pub(crate) const FOOTER: usize = 32;
pub(crate) const FINAL_DOMAIN: &[u8] = b"store.physical.extent-copy-publication.v1";
pub(crate) const CANONICAL_REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";

pub(crate) fn current_copy_publication_lsn(root: &Path) -> u64 {
    let mut paths = Vec::new();
    collect(&root.join("families/wal"), &mut paths);
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let mut copy_lsn = None;
    for path in paths {
        let bytes = std::fs::read(path).unwrap();
        if !bytes.starts_with(b"WORTHWAL") {
            continue;
        }
        let mut offset = 0;
        while let Some(header) = bytes.get(offset..offset + HEADER) {
            let length = usize::try_from(number(header, 44)).unwrap();
            let end = offset.checked_add(HEADER + length + FOOTER).unwrap();
            let frame = bytes.get(offset..end).expect("complete selected WAL frame");
            let mut payload = &frame[HEADER..frame.len() - FOOTER];
            if payload.len() < 8 || number(payload, 0) > payload.len() as u64 {
                offset = end;
                continue;
            }
            let _binding = field(&mut payload);
            let mut redo = field(&mut payload);
            assert!(payload.is_empty());
            if field(&mut redo) == FINAL_DOMAIN {
                let lsn = take_u64(&mut redo);
                let encoded = field(&mut redo);
                assert!(redo.is_empty());
                let projection =
                    worth_store_physical_format::PersistedPhysicalRecoveryProjection::decode(
                        encoded,
                        limits(encoded),
                        format,
                    )
                    .expect("current copy projection decodes");
                if matches!(
                    projection.payload(),
                    worth_store_physical_format::PersistedPhysicalRecoveryPayload::SourceCopy(_)
                ) {
                    assert_eq!(
                        projection.operation(),
                        &worth_store_physical_format::PersistedPhysicalRecoveryOperation::None
                    );
                    assert_eq!(projection.encode(), encoded);
                    assert_eq!(number(header, 28), lsn);
                    assert!(copy_lsn.replace(lsn).is_none(), "one copy publication");
                }
            }
            offset = end;
        }
    }
    copy_lsn.expect("one current copy publication WAL member")
}

pub(crate) fn current_blob_generation_lsn(root: &Path) -> u64 {
    let mut paths = Vec::new();
    collect(&root.join("families/wal"), &mut paths);
    let mut generation = None;
    for path in paths {
        let bytes = std::fs::read(path).unwrap();
        if !bytes.starts_with(b"WORTHWAL") {
            continue;
        }
        let mut offset = 0;
        while let Some(header) = bytes.get(offset..offset + HEADER) {
            let length = usize::try_from(number(header, 44)).unwrap();
            let end = offset.checked_add(HEADER + length + FOOTER).unwrap();
            let frame = bytes
                .get(offset..end)
                .expect("complete generation WAL frame");
            if frame_has_generation(frame) {
                assert!(generation.replace(number(header, 28)).is_none());
            }
            offset = end;
        }
    }
    generation.expect("one current generation publication WAL member")
}

pub(crate) fn assert_no_published_copy_resolution(root: &Path) {
    let mut paths = Vec::new();
    collect(&root.join("families/wal"), &mut paths);
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let mut published = 0;
    for path in paths {
        let bytes = std::fs::read(path).unwrap();
        if !bytes.starts_with(b"WORTHWAL") {
            continue;
        }
        let mut offset = 0;
        while let Some(header) = bytes.get(offset..offset + HEADER) {
            let length = usize::try_from(number(header, 44)).unwrap();
            let end = offset.checked_add(HEADER + length + FOOTER).unwrap();
            let frame = bytes.get(offset..end).expect("complete retained WAL frame");
            if let Ok(worth_store_physical_format::PhysicalExtentCopyRecord::Resolved(resolution)) =
                worth_store_physical_format::PhysicalExtentCopyRecord::decode(
                    &frame[HEADER..frame.len() - FOOTER],
                    format,
                )
            {
                published += u64::from(matches!(
                    resolution.kind(),
                    worth_store_physical_format::PhysicalExtentCopyResolutionKind::Published { .. }
                ));
            }
            offset = end;
        }
    }
    assert_eq!(
        published, 0,
        "copy remains unresolved before later blob roots"
    );
}

pub(crate) fn frame_has_generation(frame: &[u8]) -> bool {
    let mut payload = &frame[HEADER..frame.len() - FOOTER];
    if payload.len() < 8 || number(payload, 0) > payload.len() as u64 {
        return false;
    }
    field(&mut payload);
    let mut redo = field(&mut payload);
    assert!(payload.is_empty());
    if field(&mut redo) != CANONICAL_REDO_DOMAIN {
        return false;
    }
    let count = take_u64(&mut redo);
    for _ in 0..count {
        take(&mut redo, 4 + 8);
        let claims = take_u64(&mut redo);
        for _ in 0..claims {
            field(&mut redo);
            take(&mut redo, 32);
        }
        field(&mut redo);
    }
    let encoded = field(&mut redo);
    assert!(redo.is_empty());
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let projection = worth_store_physical_format::PersistedPhysicalRecoveryProjection::decode(
        encoded,
        limits(encoded),
        format,
    )
    .expect("current generation projection decodes");
    matches!(
        projection.operation(),
        worth_store_physical_format::PersistedPhysicalRecoveryOperation::GenerationPublished(_)
    )
}

pub(crate) fn limits(bytes: &[u8]) -> PhysicalRecoveryProjectionDecodeLimits {
    let bound = bytes.len() as u64;
    PhysicalRecoveryProjectionDecodeLimits {
        frames: bound,
        record_identities: bound,
        placements: bound,
        segment_updates: bound,
        manifests: bound,
        total_entries: bound,
        inline_allocations: bound,
    }
}
pub(crate) fn number(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
pub(crate) fn take<'a>(bytes: &mut &'a [u8], length: usize) -> &'a [u8] {
    let (head, tail) = bytes
        .split_at_checked(length)
        .expect("complete WAL member field");
    *bytes = tail;
    head
}
pub(crate) fn take_u64(bytes: &mut &[u8]) -> u64 {
    u64::from_le_bytes(take(bytes, 8).try_into().unwrap())
}
pub(crate) fn field<'a>(bytes: &mut &'a [u8]) -> &'a [u8] {
    let length = usize::try_from(take_u64(bytes)).unwrap();
    take(bytes, length)
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
