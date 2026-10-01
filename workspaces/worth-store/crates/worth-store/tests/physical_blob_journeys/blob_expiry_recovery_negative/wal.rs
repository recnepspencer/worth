//! Reseal one real WAL-durable terminal member, preserving every identity and
//! coordinate except the expiry witness. This deliberately exercises C8's
//! semantic checkpoint check rather than malformed-frame admission.

use std::{fs, num::NonZeroU64, path::Path};

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, decode_data_frame_page_lsn, decode_extent_chunk,
    encode_data_frame_page_lsn, encode_extent_chunk, BlobAbandonmentReasonV1, BlobRecordV1,
    BlobSessionAbandonedV1, DurableFrameKind, DurablePhysicalRootManifest, DurableRootSelector,
    PersistedBlobSemanticRecordBinding, PersistedPhysicalDataFrameSubject,
    PersistedPhysicalRecoveryBlobSemantic, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryProjection, PhysicalRecordFormatDeclaration,
    PhysicalRecoveryProjectionDecodeLimits, RecordArtifactFile,
};
use worth_store_recovery_physics::decode_physical_redo_records;
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

const WAL_HEADER: usize = 116;
const WAL_FOOTER: usize = 32;
const REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";

pub(super) fn reseal_one_expiry_witness_after_selected_checkpoint(root: &Path) -> u64 {
    let format = selected_format(root);
    let mut paths = Vec::new();
    collect(&root.join("families/wal"), &mut paths);
    paths.sort();
    let mut original_witness = None;
    for path in paths {
        let bytes = fs::read(&path).unwrap();
        if !bytes.starts_with(b"WORTHWAL") {
            continue;
        }
        let mut rewritten = Vec::with_capacity(bytes.len());
        let mut offset = 0;
        while let Some(header) = bytes.get(offset..offset + WAL_HEADER) {
            assert_eq!(&header[..8], b"WORTHWAL");
            let length = usize::try_from(number(header, 44)).unwrap();
            let end = offset
                .checked_add(WAL_HEADER + length + WAL_FOOTER)
                .unwrap();
            let frame = bytes.get(offset..end).expect("complete retained WAL frame");
            if let Some((replacement, witness)) = rewrite_expiry_frame(frame, format) {
                assert!(
                    original_witness.replace(witness).is_none(),
                    "one expiry member"
                );
                rewritten.extend_from_slice(&replacement);
            } else {
                rewritten.extend_from_slice(frame);
            }
            offset = end;
        }
        assert_eq!(
            offset,
            bytes.len(),
            "retained WAL has no trailing partial frame"
        );
        if rewritten != bytes {
            assert_eq!(
                rewritten.len(),
                bytes.len(),
                "witness does not change WAL length"
            );
            fs::write(path, rewritten).unwrap();
        }
    }
    original_witness.expect("one real WAL-durable checkpoint-expiry terminal")
}

fn rewrite_expiry_frame(
    frame: &[u8],
    format: PhysicalRecordFormatDeclaration,
) -> Option<(Vec<u8>, u64)> {
    let mut payload = frame.get(WAL_HEADER..frame.len().checked_sub(WAL_FOOTER)?)?;
    assert_eq!(&frame[84..116], Sha256::digest(payload).as_slice());
    assert_eq!(
        &frame[frame.len() - WAL_FOOTER..],
        Sha256::digest(&frame[..frame.len() - WAL_FOOTER]).as_slice()
    );
    let binding = field(&mut payload)?;
    let redo = field(&mut payload)?;
    if !payload.is_empty() {
        return None;
    }
    let mut cursor = redo;
    if field(&mut cursor)? != REDO_DOMAIN || read_u64(&mut cursor)? != 1 {
        return None;
    }
    let record_header = take(&mut cursor, 12)?;
    if u32::from_le_bytes(record_header[..4].try_into().unwrap()) != 0
        || read_u64(&mut cursor)? != 1
    {
        return None;
    }
    let target = field(&mut cursor)?;
    let old_target_digest = take(&mut cursor, 32)?;
    let old_control = field(&mut cursor)?;
    let encoded_projection = field(&mut cursor)?;
    if !cursor.is_empty() {
        return None;
    }
    let projection = PersistedPhysicalRecoveryProjection::decode_frames(
        encoded_projection,
        PhysicalRecoveryProjectionDecodeLimits {
            frames: 16,
            record_identities: 16,
            placements: 16,
            segment_updates: 16,
            manifests: 16,
            total_entries: 64,
            inline_allocations: 16,
        },
    )
    .ok()?;
    let PersistedPhysicalRecoveryBlobSemantic::SessionAbandoned(semantic) =
        projection.blob_semantic()
    else {
        return None;
    };
    let BlobRecordV1::SessionAbandoned(old) = decode_blob_record(old_control).ok()? else {
        return None;
    };
    let BlobAbandonmentReasonV1::CheckpointExpired {
        checkpoint_sequence,
    } = old.reason()
    else {
        return None;
    };
    let next = NonZeroU64::new(checkpoint_sequence.get().checked_add(1)?)?;
    let new_control = BlobSessionAbandonedV1::new(
        old.store(),
        old.session(),
        old.declaration_record(),
        old.declaration_digest(),
        BlobAbandonmentReasonV1::CheckpointExpired {
            checkpoint_sequence: next,
        },
    )
    .ok()?
    .encode();
    assert_eq!(new_control.len(), old_control.len());
    let old_frames = projection.frames()?;
    let [old_frame] = old_frames else {
        panic!("terminal publication writes one exact extent frame");
    };
    let PersistedPhysicalDataFrameSubject::ExtentChunk(coordinate) = old_frame.subject() else {
        panic!("Store blob controls are extent-resident");
    };
    let (old_payload, format) = decode_extent_chunk(old_frame.bytes(), coordinate).unwrap();
    assert_eq!(
        old_payload, old_control,
        "projection carries exact terminal bytes"
    );
    assert_eq!(
        old_target_digest,
        Sha256::digest(old_frame.bytes()).as_slice()
    );
    let lsn = decode_data_frame_page_lsn(old_frame.bytes(), DurableFrameKind::Extent).unwrap();
    let mut frame_bytes = encode_extent_chunk(format, coordinate, &new_control).unwrap();
    encode_data_frame_page_lsn(&mut frame_bytes, DurableFrameKind::Extent, lsn).unwrap();
    assert_eq!(frame_bytes.len(), old_frame.bytes().len());
    let replacement_frame = PersistedPhysicalRecoveryFrame::new(
        old_frame.subject(),
        old_frame.coordinate(),
        &frame_bytes,
    )
    .unwrap();
    let replacement_binding = PersistedBlobSemanticRecordBinding::new(
        semantic.record(),
        Sha256::digest(&new_control).into(),
        semantic.candidate_root_generation(),
    )
    .unwrap();
    let replacement_projection = PersistedPhysicalRecoveryProjection::new_with_blob_semantic(
        projection.source_root_generation(),
        projection.root_state().clone(),
        projection.record_identities().to_vec(),
        vec![replacement_frame],
        projection.placements().to_vec(),
        projection.segment_updates().to_vec(),
        projection.manifests().to_vec(),
        PersistedPhysicalRecoveryBlobSemantic::SessionAbandoned(replacement_binding),
    )
    .expect("same production coordinates admit the resealed frame");
    let mut changed_redo = Vec::with_capacity(redo.len());
    write_field(&mut changed_redo, REDO_DOMAIN);
    changed_redo.extend_from_slice(&1_u64.to_le_bytes());
    changed_redo.extend_from_slice(record_header);
    changed_redo.extend_from_slice(&1_u64.to_le_bytes());
    write_field(&mut changed_redo, target);
    changed_redo.extend_from_slice(&Sha256::digest(&frame_bytes));
    write_field(&mut changed_redo, &new_control);
    write_field(&mut changed_redo, &replacement_projection.encode());
    assert_eq!(changed_redo.len(), redo.len());
    let lsn = number(record_header, 4);
    assert_eq!(lsn, number(frame, 28), "one WAL member binds its redo LSN");
    let range = WalLsnRange::new(
        LogSequenceNumber::new(lsn),
        LogSequenceNumber::new(lsn.checked_add(1).unwrap()),
    )
    .unwrap();
    decode_physical_redo_records(&changed_redo, range, 16, format)
        .expect("production canonical redo decoder admits the resealed terminal");
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode_frames(
            &replacement_projection.encode(),
            PhysicalRecoveryProjectionDecodeLimits {
                frames: 16,
                record_identities: 16,
                placements: 16,
                segment_updates: 16,
                manifests: 16,
                total_entries: 64,
                inline_allocations: 16,
            },
        )
        .unwrap(),
        replacement_projection
    );
    let mut changed_binding = binding.to_vec();
    let digest_offset = changed_binding.len().checked_sub(40)?;
    assert_eq!(number(&changed_binding, digest_offset), 32);
    changed_binding[digest_offset + 8..].copy_from_slice(&Sha256::digest(&changed_redo));
    let mut changed_payload = Vec::new();
    write_field(&mut changed_payload, &changed_binding);
    write_field(&mut changed_payload, &changed_redo);
    assert_eq!(changed_payload.len(), frame.len() - WAL_HEADER - WAL_FOOTER);
    let mut changed_frame = frame[..WAL_HEADER].to_vec();
    changed_frame[84..116].copy_from_slice(&Sha256::digest(&changed_payload));
    changed_frame.extend_from_slice(&changed_payload);
    let footer = Sha256::digest(&changed_frame);
    changed_frame.extend_from_slice(&footer);
    assert_eq!(changed_frame.len(), frame.len());
    Some((changed_frame, checkpoint_sequence.get()))
}

fn selected_format(root: &Path) -> PhysicalRecordFormatDeclaration {
    let records = root.join("families/records");
    let selector = DurableRootSelector::decode(
        &fs::read(records.join(RecordArtifactFile::CurrentRootSelector.file_name())).unwrap(),
    )
    .expect("selected root selector");
    let root_bytes = fs::read(
        records.join("roots").join(
            RecordArtifactFile::RootManifest {
                generation: selector.root_generation(),
            }
            .file_name(),
        ),
    )
    .unwrap();
    let (manifest, format) = DurablePhysicalRootManifest::decode(&root_bytes, u16::MAX)
        .expect("addressed selected root manifest");
    assert_eq!(manifest.generation(), selector.root_generation());
    assert_eq!(format, selector.format());
    assert_eq!(manifest.encode(format), root_bytes);
    format
}

fn number(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
fn take<'a>(bytes: &mut &'a [u8], length: usize) -> Option<&'a [u8]> {
    let (head, tail) = bytes.split_at_checked(length)?;
    *bytes = tail;
    Some(head)
}
fn read_u64(bytes: &mut &[u8]) -> Option<u64> {
    Some(u64::from_le_bytes(take(bytes, 8)?.try_into().ok()?))
}
fn field<'a>(bytes: &mut &'a [u8]) -> Option<&'a [u8]> {
    let length = usize::try_from(read_u64(bytes)?).ok()?;
    take(bytes, length)
}
fn write_field(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_le_bytes());
    target.extend_from_slice(value);
}
fn collect(directory: &Path, paths: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, paths);
        } else {
            paths.push(path);
        }
    }
}
