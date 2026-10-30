#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_outer_corruption::*;
use {std::fs, std::num::NonZeroU64, std::path::Path, std::path::PathBuf};

use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::PersistedRecordIdentity;
use {
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadFailure, worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PublishedBlobGeneration,
};

use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

#[test]
fn outer_crc_damage_is_one_selected_chunk_not_store_wide_health() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.outer-crc.scope");
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(512).unwrap());
    let (published, object) = {
        let serving = serving_from_initialization(directory.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
            object,
            worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
            (3 * CHUNK_BYTES) as u64,
            &scope,
            worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(64).unwrap(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
            .unwrap();
        for ordinal in 0..3_u8 {
            ingest.push(&vec![ordinal + 0x41; CHUNK_BYTES]).unwrap();
        }
        let published = ingest.finish().unwrap();
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
            worth_store::physical_runtime::BlobReadFailure::ChunkOuterCorruption { ordinal: 1, record }
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
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(512).unwrap());
    let (published, object) = {
        let serving = serving_from_initialization(directory.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
            object,
            worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
            (3 * CHUNK_BYTES) as u64,
            &scope,
            worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(64).unwrap(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
            .unwrap();
        for ordinal in 0..3_u8 {
            ingest.push(&vec![ordinal + 0x51; CHUNK_BYTES]).unwrap();
        }
        let published = ingest.finish().unwrap();
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
        !matches!(
            error,
            worth_store::physical_runtime::BlobReadFailure::ChunkOuterCorruption { .. }
        ),
        "identity damage cannot claim selected-data-frame checksum provenance"
    );
    drop(damaged);
    assert!(
        serving.blobs().is_err(),
        "an exact C5 frame with the wrong selected record identity is not isolated CRC damage"
    );
}
