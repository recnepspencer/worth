use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobCheckpointLimit, BlobIngestDeclaration, BlobIngestSession,
    BlobReadLimits, PhysicalBlobFacade, PhysicalMutationDeadline, PublishedBlobGeneration,
    ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{decode_blob_record, BlobRecordV1, PersistedRecordIdentity};

use super::{
    blob_frontier::selected_blob_records,
    fixture::{admitted_blob_scope, placement, serving_from_initialization},
};

const CHUNK_BYTES: usize = 64 * 1024;

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
    drop(blobs);
    serving.close();
}

fn begin<'runtime>(
    blobs: &PhysicalBlobFacade<'runtime>,
    scope: &AdmittedBlobScope,
) -> BlobIngestSession<'runtime> {
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let declaration = BlobIngestDeclaration::new(
        blobs.issue_object_id(limits).unwrap(),
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    blobs
        .begin_ingest(declaration, placement(), (CHUNK_BYTES / 2) as u64, limits)
        .unwrap()
}

fn selected_publication(serving: &ServingPhysicalRuntime) -> PersistedRecordIdentity {
    serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("published generation selected")
        .record()
}

fn reuse_claims(serving: &ServingPhysicalRuntime) -> Vec<PersistedRecordIdentity> {
    selected_blob_records(serving)
        .iter()
        .filter_map(|(_, bytes)| match decode_blob_record(bytes) {
            Ok(BlobRecordV1::ChunkReuseClaimV2(value)) => Some(value.claim().source_publication()),
            _ => None,
        })
        .collect()
}

fn read_exact(
    blobs: &PhysicalBlobFacade<'_>,
    scope: &AdmittedBlobScope,
    published: PublishedBlobGeneration,
    payload: &[u8],
) -> u64 {
    let limits = BlobReadLimits::new(NonZeroU64::new(1).unwrap());
    let mut read = blobs
        .read(published, scope, 0, payload.len() as u64, limits)
        .unwrap();
    let mut result = Vec::new();
    let mut transfer = [0_u8; 4096];
    while read.remaining_bytes() != 0 {
        let count = read.read_next(&mut transfer).unwrap();
        result.extend_from_slice(&transfer[..count]);
    }
    assert_eq!(result, payload);
    read.observation().reuse_source_selected_reads()
}
