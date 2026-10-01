//! Reconstruct the old v5 SourceCopy wire form from a real durable v6 copy.
//! The only version difference is the v6 `None` semantic field. The resulting
//! member and WAL checksums are rebuilt before the production recovery reads it.
use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    PersistedPhysicalRecoveryBlobSemantic, PersistedPhysicalRecoveryPayload,
    PersistedPhysicalRecoveryProjection, PhysicalExtentCopyRecord,
    PhysicalExtentCopyResolutionKind, PhysicalRecordFormatDeclaration,
    PhysicalRecoveryProjectionDecodeLimits,
};

const HEADER: usize = 116;
const FOOTER: usize = 32;
const FINAL_DOMAIN: &[u8] = b"store.physical.extent-copy-publication.v1";
const V6_DOMAIN: &[u8] = b"store.physical.recovery-projection.v6";
const CANONICAL_REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";

pub(super) fn rewrite_one_durable_copy_as_v5(root: &Path) -> u64 {
    let mut paths = Vec::new();
    collect(&root.join("families/wal"), &mut paths);
    paths.sort();
    let mut copy_lsn = None;
    for path in paths {
        let bytes = fs::read(&path).unwrap();
        if !bytes.starts_with(b"WORTHWAL") {
            continue;
        }
        let mut rewritten = Vec::with_capacity(bytes.len());
        let mut offset = 0;
        while let Some(header) = bytes.get(offset..offset + HEADER) {
            assert_eq!(&header[..8], b"WORTHWAL");
            let length = usize::try_from(number(header, 44)).unwrap();
            let end = offset.checked_add(HEADER + length + FOOTER).unwrap();
            let frame = bytes
                .get(offset..end)
                .expect("durable WAL frame is complete");
            if let Some((replacement, lsn)) = rewrite_copy_frame(frame) {
                assert!(copy_lsn.replace(lsn).is_none(), "one final copy WAL member");
                rewritten.extend_from_slice(&replacement);
            } else {
                rewritten.extend_from_slice(frame);
            }
            offset = end;
        }
        rewritten.extend_from_slice(&bytes[offset..]);
        if rewritten != bytes {
            fs::write(path, rewritten).unwrap();
        }
    }
    copy_lsn.expect("one production SourceCopy publication WAL member")
}

pub(super) fn v6_blob_generation_lsn(root: &Path) -> u64 {
    let mut paths = Vec::new();
    collect(&root.join("families/wal"), &mut paths);
    let mut generation = None;
    for path in paths {
        let bytes = fs::read(path).unwrap();
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
            if frame_has_v6_generation(frame) {
                assert!(generation.replace(number(header, 28)).is_none());
            }
            offset = end;
        }
    }
    generation.expect("one v6 generation publication WAL member")
}

pub(super) fn assert_no_published_copy_resolution(root: &Path) {
    let mut paths = Vec::new();
    collect(&root.join("families/wal"), &mut paths);
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let mut published = 0;
    for path in paths {
        let bytes = fs::read(path).unwrap();
        if !bytes.starts_with(b"WORTHWAL") {
            continue;
        }
        let mut offset = 0;
        while let Some(header) = bytes.get(offset..offset + HEADER) {
            assert_eq!(&header[..8], b"WORTHWAL");
            let length = usize::try_from(number(header, 44)).unwrap();
            let end = offset.checked_add(HEADER + length + FOOTER).unwrap();
            let frame = bytes.get(offset..end).expect("complete retained WAL frame");
            if let Ok(PhysicalExtentCopyRecord::Resolved(resolution)) =
                PhysicalExtentCopyRecord::decode(&frame[HEADER..frame.len() - FOOTER], format)
            {
                published += u64::from(matches!(
                    resolution.kind(),
                    PhysicalExtentCopyResolutionKind::Published { .. }
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

fn frame_has_v6_generation(frame: &[u8]) -> bool {
    let mut payload = &frame[HEADER..frame.len() - FOOTER];
    if payload.len() < 8 || number(payload, 0) > payload.len() as u64 {
        return false; // extent-copy intent and resolution frames are raw records
    }
    field(&mut payload); // attempt binding
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
    let projection = field(&mut redo);
    assert!(redo.is_empty());
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let projection =
        PersistedPhysicalRecoveryProjection::decode(projection, limits(projection), format)
            .expect("production generation projection is decodable");
    matches!(
        projection.blob_semantic(),
        PersistedPhysicalRecoveryBlobSemantic::GenerationPublished(_)
    )
}

fn rewrite_copy_frame(frame: &[u8]) -> Option<(Vec<u8>, u64)> {
    let payload = &frame[HEADER..frame.len() - FOOTER];
    if payload.len() < 8 || number(payload, 0) > payload.len() as u64 {
        return None;
    }
    let mut outer = payload;
    let binding = field(&mut outer);
    let redo = field(&mut outer);
    assert!(outer.is_empty());
    let mut tail = redo;
    if field(&mut tail) != FINAL_DOMAIN {
        return None;
    }
    let publication_lsn = take_u64(&mut tail);
    let projection = field(&mut tail);
    assert!(tail.is_empty());
    assert_eq!(number(frame, 28), publication_lsn);
    let legacy_projection = v5_projection(projection);
    let mut legacy_redo = Vec::new();
    write_field(&mut legacy_redo, FINAL_DOMAIN);
    legacy_redo.extend_from_slice(&publication_lsn.to_le_bytes());
    write_field(&mut legacy_redo, &legacy_projection);
    let mut legacy_binding = binding.to_vec();
    let digest_field = legacy_binding.len() - 40;
    assert_eq!(number(&legacy_binding, digest_field), 32);
    legacy_binding[digest_field + 8..].copy_from_slice(&Sha256::digest(&legacy_redo));
    let mut legacy_payload = Vec::new();
    write_field(&mut legacy_payload, &legacy_binding);
    write_field(&mut legacy_payload, &legacy_redo);
    let mut legacy_frame = frame[..HEADER].to_vec();
    legacy_frame[44..52].copy_from_slice(&(legacy_payload.len() as u64).to_le_bytes());
    legacy_frame[84..116].copy_from_slice(&Sha256::digest(&legacy_payload));
    legacy_frame.extend_from_slice(&legacy_payload);
    let footer = Sha256::digest(&legacy_frame);
    legacy_frame.extend_from_slice(&footer);
    Some((legacy_frame, publication_lsn))
}

fn v5_projection(projection: &[u8]) -> Vec<u8> {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    assert!(matches!(
        PersistedPhysicalRecoveryProjection::decode(projection, limits(projection), format)
            .unwrap()
            .payload(),
        PersistedPhysicalRecoveryPayload::SourceCopy(_)
    ));
    let mut tail = projection;
    assert_eq!(field(&mut tail), V6_DOMAIN);
    take(&mut tail, 8); // source root
    field(&mut tail); // root state
    assert_eq!(take_u64(&mut tail), 1);
    field(&mut tail); // selected record identity
    assert_eq!(take(&mut tail, 1), [1]); // SourceCopy
    field(&mut tail); // exact source-copy intent
    take(&mut tail, 8 + 32); // intent LSN and digest
    let semantic_start = projection.len() - tail.len();
    assert_eq!(field(&mut tail), [0]);
    let semantic_end = projection.len() - tail.len();
    let mut legacy = projection.to_vec();
    legacy[8 + V6_DOMAIN.len() - 1] = b'5';
    legacy.drain(semantic_start..semantic_end);
    let decoded = PersistedPhysicalRecoveryProjection::decode(&legacy, limits(&legacy), format)
        .expect("legacy v5 copy must be production-decodable");
    assert!(matches!(
        decoded.payload(),
        PersistedPhysicalRecoveryPayload::SourceCopy(_)
    ));
    assert_eq!(decoded.encode(), legacy, "legacy bytes must be canonical");
    legacy
}

fn limits(bytes: &[u8]) -> PhysicalRecoveryProjectionDecodeLimits {
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

fn number(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
fn take<'a>(bytes: &mut &'a [u8], length: usize) -> &'a [u8] {
    let (head, tail) = bytes
        .split_at_checked(length)
        .expect("complete WAL member field");
    *bytes = tail;
    head
}
fn take_u64(bytes: &mut &[u8]) -> u64 {
    u64::from_le_bytes(take(bytes, 8).try_into().unwrap())
}
fn field<'a>(bytes: &mut &'a [u8]) -> &'a [u8] {
    let length = usize::try_from(take_u64(bytes)).unwrap();
    take(bytes, length)
}
fn write_field(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_le_bytes());
    target.extend_from_slice(value);
}
fn collect(directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, paths);
        } else {
            paths.push(path);
        }
    }
}
