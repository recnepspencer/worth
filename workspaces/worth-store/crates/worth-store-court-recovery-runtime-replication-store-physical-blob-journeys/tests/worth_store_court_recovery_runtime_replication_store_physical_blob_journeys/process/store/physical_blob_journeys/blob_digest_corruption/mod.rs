#![cfg(all())]

#[allow(unused_imports)] use crate::physical_blob_journeys::blob_digest_corruption::*;
use {std::fs, std::num::NonZeroU64, std::path::Path};

use {sha2::Digest, sha2::Sha256};
use {worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PublishedBlobGeneration};
use {worth_store_blob_chunks::BlobChunkSize};
use {worth_store_physical_format::BlobRecordDenial, worth_store_physical_format::DecodedBlobChunkFrameV1};

use {crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};

#[test]
fn damaged_inner_digest_field_localizes_without_inventing_an_observed_digest() {
    let (root, scope, published, limits) = one_chunk_store("c11.digest-field.corrupt");
    let before = observe_closed_store_named(root.path(), "c11-digest-field", "before");
    let identity = mutate_selected_chunk(root.path(), &before, crate::physical_blob_journeys::blob_digest_corruption::Mutation::StoredDigestField);
    let after = observe_closed_store_named(root.path(), "c11-digest-field", "damaged");
    let row = selected_chunk(&after);
    assert_eq!(row["identity"], identity);
    assert_eq!(row["outcome"]["posture"], "damaged", "{after}");

    let serving = serving_from_open(root.path());
    let blobs = serving.blobs().unwrap();
    let mut read = blobs.read(published, &scope, 0, 1, limits).unwrap();
    let error = read.read_next(&mut [0; 1]).unwrap_err();
    assert!(
        matches!(
            error,
            worth_store::physical_runtime::BlobReadFailure::ChunkFrameCorruption { ordinal: 0, .. }
        ),
        "stored-digest field damage has no trustworthy observed content digest: {error:?}"
    );
    let target = read
        .damaged_chunk_scrub_target()
        .unwrap()
        .expect("selected inner frame failure retains exact chunk identity");
    assert_eq!(
        format!(
            "blob-record:{}",
            hex_record(target.scope().blob_record_identity().unwrap().0)
        ),
        identity
    );
}

#[test]
fn malformed_chunk_length_localizes_to_selected_chunk_record() {
    let (root, scope, published, limits) = one_chunk_store("c11.chunk-length.corrupt");
    let before = observe_closed_store_named(root.path(), "c11-chunk-length", "before");
    let identity = mutate_selected_chunk(root.path(), &before, crate::physical_blob_journeys::blob_digest_corruption::Mutation::ContentLengthField);
    let after = observe_closed_store_named(root.path(), "c11-chunk-length", "damaged");
    let row = selected_chunk(&after);
    assert_eq!(row["identity"], identity);
    assert_eq!(row["outcome"]["posture"], "damaged", "{after}");

    let serving = serving_from_open(root.path());
    let blobs = serving.blobs().unwrap();
    let mut read = blobs.read(published, &scope, 0, 1, limits).unwrap();
    let error = read.read_next(&mut [0; 1]).unwrap_err();
    assert!(
        matches!(
            error,
            worth_store::physical_runtime::BlobReadFailure::ChunkFrameCorruption { ordinal: 0, .. }
        ),
        "structurally malformed selected frame must identify its chunk: {error:?}"
    );
    let target = read.damaged_chunk_scrub_target().unwrap().unwrap();
    assert_eq!(
        format!(
            "blob-record:{}",
            hex_record(target.scope().blob_record_identity().unwrap().0)
        ),
        identity
    );
}

#[test]
fn valid_chunk_with_resealed_changed_content_blames_parent_edge_not_chunk() {
    let (root, scope, published, limits) = one_chunk_store("c11.valid-chunk.wrong-edge");
    let before = observe_closed_store_named(root.path(), "c11-edge-mismatch", "before");
    let identity = mutate_selected_chunk(root.path(), &before, crate::physical_blob_journeys::blob_digest_corruption::Mutation::ValidChangedContent);
    let after = observe_closed_store_named(root.path(), "c11-edge-mismatch", "changed");
    let chunk = selected_chunk(&after);
    assert_eq!(chunk["identity"], identity);
    assert_eq!(chunk["outcome"]["posture"], "intact", "{after}");
    assert!(
        after["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["family"] == "blob_tree_node" && row["outcome"]["posture"] == "damaged"),
        "the selected parent edge, not the valid chunk, is inconsistent: {after}"
    );

    let serving = serving_from_open(root.path());
    let blobs = serving.blobs().unwrap();
    let mut read = blobs.read(published, &scope, 0, 1, limits).unwrap();
    let error = read.read_next(&mut [0; 1]).unwrap_err();
    assert!(matches!(error, worth_store::physical_runtime::BlobReadFailure::TreeDamaged), "{error:?}");
    assert!(
        read.damaged_chunk_scrub_target().unwrap().is_none(),
        "a valid chunk must not acquire a false chunk-damage target"
    );
}
