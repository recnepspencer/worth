use std::{
    fs,
    num::NonZeroU64,
    path::{Path, PathBuf},
};

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadFailure, BlobReadLimits,
    PhysicalMutationDeadline, PublishedBlobGeneration,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::PersistedRecordIdentity;

use super::fixture::{
    admitted_blob_scope, placement, serving_from_initialization, serving_from_open,
};

const CHUNK_BYTES: usize = 64 * 1024;
const C5_HEADER: usize = 48;
const C5_EXTENT_METADATA: usize = 64;
const C11_HEADER: usize = 48;
const C11_CHUNK_CLAIM: usize = 80;
const C11_CONTENT_PREFIX: usize = 12;

#[test]
fn outer_crc_damage_is_one_selected_chunk_not_store_wide_health() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.outer-crc.scope");
    let limits = BlobReadLimits::new(NonZeroU64::new(512).unwrap());
    let (published, object) = {
        let serving = serving_from_initialization(directory.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = BlobIngestDeclaration::new(
            object,
            BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
            (3 * CHUNK_BYTES) as u64,
            &scope,
            BlobCheckpointLimit::bounded_horizon(64).unwrap(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
            .unwrap();
        for ordinal in 0..3_u8 {
            ingest.push(&vec![ordinal + 0x41; CHUNK_BYTES]).unwrap();
        }
        let published = ingest.finish().unwrap();
        drop(blobs);
        serving.close();
        (published, object)
    };

    let damaged_record = flip_middle_chunk_without_resealing_outer(directory.path());
    let serving = serving_from_open(directory.path());
    let blobs = serving.blobs().unwrap();
    let resolved = blobs
        .resolve_publication(
            object.bytes(),
            published.generation().sequence(),
            &scope,
            limits,
        )
        .unwrap();
    assert_eq!(resolved, published);
    assert_eq!(read_one_byte(&blobs, resolved, &scope, 0, limits), 0x41);
    assert_eq!(
        read_one_byte(&blobs, resolved, &scope, (2 * CHUNK_BYTES) as u64, limits),
        0x43
    );

    let mut damaged = blobs
        .read(resolved, &scope, CHUNK_BYTES as u64, 1, limits)
        .unwrap();
    let error = damaged.read_next(&mut [0_u8; 1]).unwrap_err();
    assert!(
        matches!(
            &error,
            BlobReadFailure::ChunkOuterCorruption { ordinal: 1, record }
                if *record == damaged_record
        ),
        "outer C.5 failure must carry trusted tree ordinal and selected RecordId: {error:?}"
    );
    drop(damaged);

    assert_eq!(
        read_one_byte(&blobs, resolved, &scope, 0, limits),
        0x41,
        "the bad C.5 data frame must not revoke disjoint selected ranges"
    );
    assert_eq!(
        read_one_byte(&blobs, resolved, &scope, (2 * CHUNK_BYTES) as u64, limits),
        0x43
    );
    assert!(
        serving.blobs().is_ok(),
        "diagnostic and blob facades must remain available"
    );
}

#[test]
fn resealed_selected_chunk_record_identity_mismatch_still_revokes_global_health() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.outer-identity.scope");
    let limits = BlobReadLimits::new(NonZeroU64::new(512).unwrap());
    let (published, object) = {
        let serving = serving_from_initialization(directory.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = BlobIngestDeclaration::new(
            object,
            BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
            (3 * CHUNK_BYTES) as u64,
            &scope,
            BlobCheckpointLimit::bounded_horizon(64).unwrap(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
            .unwrap();
        for ordinal in 0..3_u8 {
            ingest.push(&vec![ordinal + 0x51; CHUNK_BYTES]).unwrap();
        }
        let published = ingest.finish().unwrap();
        drop(blobs);
        serving.close();
        (published, object)
    };

    corrupt_middle_chunk_record_identity_reseal_outer(directory.path());
    let serving = serving_from_open(directory.path());
    let blobs = serving.blobs().unwrap();
    let resolved = blobs
        .resolve_publication(
            object.bytes(),
            published.generation().sequence(),
            &scope,
            limits,
        )
        .unwrap();
    let mut damaged = blobs
        .read(resolved, &scope, CHUNK_BYTES as u64, 1, limits)
        .unwrap();
    let error = damaged.read_next(&mut [0_u8; 1]).unwrap_err();
    assert!(
        !matches!(error, BlobReadFailure::ChunkOuterCorruption { .. }),
        "identity damage cannot claim selected-data-frame checksum provenance"
    );
    drop(damaged);
    assert!(
        serving.blobs().is_err(),
        "an exact C5 frame with the wrong selected record identity is not isolated CRC damage"
    );
}

fn read_one_byte(
    blobs: &worth_store::physical_runtime::PhysicalBlobFacade<'_>,
    published: PublishedBlobGeneration,
    scope: &worth_store::physical_runtime::AdmittedBlobScope,
    offset: u64,
    limits: BlobReadLimits,
) -> u8 {
    let mut read = blobs.read(published, scope, offset, 1, limits).unwrap();
    let mut byte = [0_u8; 1];
    assert_eq!(read.read_next(&mut byte).unwrap(), 1);
    byte[0]
}

/// Select the sole ordinal-one C.11 frame in this three-chunk fixture. The
/// expected identity is read independently from the unmodified C.5 extent
/// metadata; corruption changes only content bytes, never that identity.
fn flip_middle_chunk_without_resealing_outer(root: &Path) -> PersistedRecordIdentity {
    let (path, _, inner_offset, _, record) = locate_middle_chunk(root);
    let mut media = fs::read(&path).unwrap();
    media[inner_offset + C11_HEADER + C11_CHUNK_CLAIM + C11_CONTENT_PREFIX] ^= 1;
    fs::write(path, media).unwrap();
    record
}

fn corrupt_middle_chunk_record_identity_reseal_outer(root: &Path) {
    let (path, outer_offset, _, outer_length, _) = locate_middle_chunk(root);
    let mut media = fs::read(&path).unwrap();
    let frame = &mut media[outer_offset..outer_offset + outer_length];
    frame[C5_HEADER] ^= 0x80;
    let checksum = crc32c(&frame[..44], &frame[C5_HEADER..]);
    frame[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    fs::write(path, media).unwrap();
}

fn locate_middle_chunk(root: &Path) -> (PathBuf, usize, usize, usize, PersistedRecordIdentity) {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(root.join("families/records/arenas")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "data") {
            continue;
        }
        let media = fs::read(&path).unwrap();
        for inner_offset in 0..media.len().saturating_sub(C11_HEADER) {
            let Some(outer_offset) = inner_offset.checked_sub(C5_HEADER + C5_EXTENT_METADATA)
            else {
                continue;
            };
            if !media[inner_offset..].starts_with(b"WRC11BLB")
                || media[inner_offset + 8] != 2
                || !media[outer_offset..].starts_with(b"WRC5FRM\0")
                || media[outer_offset + 8] != 4
            {
                continue;
            }
            let ordinal = u64::from_le_bytes(
                media[inner_offset + C11_HEADER + 32..inner_offset + C11_HEADER + 40]
                    .try_into()
                    .unwrap(),
            );
            if ordinal != 1 {
                continue;
            }
            let outer_payload = u32::from_le_bytes(
                media[outer_offset + 24..outer_offset + 28]
                    .try_into()
                    .unwrap(),
            ) as usize;
            let inner_length = C11_HEADER
                + u32::from_le_bytes(
                    media[inner_offset + 12..inner_offset + 16]
                        .try_into()
                        .unwrap(),
                ) as usize;
            if outer_offset + C5_HEADER + outer_payload > media.len()
                || inner_length != C11_HEADER + C11_CHUNK_CLAIM + C11_CONTENT_PREFIX + CHUNK_BYTES
                || outer_payload
                    <= C5_EXTENT_METADATA + C11_HEADER + C11_CHUNK_CLAIM + C11_CONTENT_PREFIX
            {
                continue;
            }
            let record = PersistedRecordIdentity::new(
                media[outer_offset + C5_HEADER..outer_offset + C5_HEADER + 16]
                    .try_into()
                    .unwrap(),
                u64::from_le_bytes(
                    media[outer_offset + C5_HEADER + 16..outer_offset + C5_HEADER + 24]
                        .try_into()
                        .unwrap(),
                ),
            )
            .unwrap();
            candidates.push((
                path.clone(),
                outer_offset,
                inner_offset,
                C5_HEADER + outer_payload,
                record,
            ));
        }
    }
    assert_eq!(
        candidates.len(),
        1,
        "fixture must have one ordinal-one chunk"
    );
    candidates.pop().unwrap()
}

fn crc32c(prefix: &[u8], payload: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in prefix.iter().chain(payload) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}
