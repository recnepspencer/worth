use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits,
    PhysicalMutationDeadline, PublishedBlobGeneration, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::fixture::{
    admitted_blob_scope, admitted_blob_scope_for_replay_boundary, placement,
    serving_from_initialization, serving_from_open,
};

const CHUNK_BYTES: usize = 64 * 1024;

fn publish_one(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    payload: &[u8],
) -> PublishedBlobGeneration {
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        payload.len() as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), (CHUNK_BYTES / 2) as u64, limits)
        .unwrap();
    ingest.push(&payload[..CHUNK_BYTES / 2]).unwrap();
    ingest.push(&payload[CHUNK_BYTES / 2..]).unwrap();
    ingest.finish().unwrap()
}

#[test]
fn same_scope_reuse_survives_reopen_without_selected_scan_and_other_scope_writes_new_chunk() {
    let root = tempfile::tempdir().unwrap();
    let payload = vec![0x6D; CHUNK_BYTES];
    let serving = serving_from_initialization(root.path());
    let scope = admitted_blob_scope("c11.layout.dedupe.same");
    let other_scope = admitted_blob_scope_for_replay_boundary("c11.layout.dedupe.other");
    let original = publish_one(&serving, &scope, &payload);
    let reused = publish_one(&serving, &scope, &payload);
    let isolated = publish_one(&serving, &other_scope, &payload);
    serving.close();

    let reopened = serving_from_open(root.path());
    let limits = BlobReadLimits::new(NonZeroU64::new(1).unwrap());
    for (published, admitted, expected_reuse) in [
        (original, &scope, false),
        (reused, &scope, true),
        (isolated, &other_scope, false),
    ] {
        let blobs = reopened.blobs().unwrap();
        let mut read = blobs
            .read(published, admitted, 0, CHUNK_BYTES as u64, limits)
            .unwrap();
        let mut result = Vec::new();
        let mut transfer = [0_u8; 4096];
        while read.remaining_bytes() != 0 {
            let count = read.read_next(&mut transfer).unwrap();
            result.extend_from_slice(&transfer[..count]);
        }
        assert_eq!(result, payload);
        assert_eq!(
            read.observation().reuse_source_selected_reads() > 0,
            expected_reuse
        );
        assert_eq!(read.observation().touched_chunks(), 1);
    }
    reopened.close();
}
