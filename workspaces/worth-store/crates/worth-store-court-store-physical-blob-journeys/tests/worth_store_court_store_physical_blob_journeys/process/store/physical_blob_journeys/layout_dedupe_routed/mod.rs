#[allow(unused_imports)]
use crate::physical_blob_journeys::layout_dedupe_routed::*;
use std::num::NonZeroU64;

use worth_store_blob_chunks::BlobChunkSize;
use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobIngestSession,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::PhysicalBlobFacade,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PublishedBlobGeneration,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

/// Two concurrent ingests of the same bytes both write original
/// occurrences, because neither is indexed when the other looks up. When the
/// second publication is maintained, its original chunk meets a dedupe cell
/// whose source publication is still routed: maintenance verifies that source
/// and keeps the cell, so later ingests reuse the first publication.
#[test]
fn original_occurrence_meeting_routed_source_keeps_cell_and_live_reuse() {
    let root = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.layout.dedupe.routed");
    let payload = vec![0x5A; CHUNK_BYTES];
    let serving = serving_from_initialization(root.path());
    let blobs = serving.blobs().unwrap();
    let mut first = begin(&blobs, &scope);
    let mut second = begin(&blobs, &scope);
    for ingest in [&mut first, &mut second] {
        ingest.push(&payload[..CHUNK_BYTES / 2]).unwrap();
        ingest.push(&payload[CHUNK_BYTES / 2..]).unwrap();
    }
    first.finish().unwrap();
    let first_source = selected_publication(&serving);
    second.finish().unwrap();
    let second_source = selected_publication(&serving);
    assert_ne!(first_source, second_source);
    assert_eq!(
        reuse_claims(&serving),
        Vec::<PersistedRecordIdentity>::new(),
        "both concurrent ingests wrote original occurrences",
    );

    let mut third = begin(&blobs, &scope);
    third.push(&payload[..CHUNK_BYTES / 2]).unwrap();
    third.push(&payload[CHUNK_BYTES / 2..]).unwrap();
    let reused = third.finish().unwrap();
    assert_eq!(
        reuse_claims(&serving),
        vec![first_source],
        "the routed cell is unchanged and still names the first publication",
    );
    assert!(read_exact(&blobs, &scope, reused, &payload) > 0);
    serving.close();
}
