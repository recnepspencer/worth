#[allow(unused_imports)]
use crate::physical_blob_journeys::layout_dedupe::*;
use std::num::NonZeroU64;

use worth_store_blob_chunks::BlobChunkSize;
use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PublishedBlobGeneration,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::admitted_blob_scope_for_replay_boundary,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

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
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(1).unwrap());
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
