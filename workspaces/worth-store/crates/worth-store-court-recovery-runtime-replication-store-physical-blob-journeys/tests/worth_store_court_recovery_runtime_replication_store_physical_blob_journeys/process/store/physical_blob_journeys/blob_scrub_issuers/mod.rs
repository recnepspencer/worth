#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_scrub_issuers::*;

use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

#[test]
fn selected_catalog_issues_publication_before_decoding_damaged_target() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.scrub.publication.issuer");
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
    let (object, generation, original_record) = {
        let serving = serving_from_initialization(directory.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
            object,
            worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
            (2 * CHUNK_BYTES) as u64,
            &scope,
            worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(64).unwrap(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
            .unwrap();
        ingest.push(&vec![0x41; CHUNK_BYTES]).unwrap();
        ingest.push(&vec![0x42; CHUNK_BYTES]).unwrap();
        let published = ingest.finish().unwrap();
        let target = blobs
            .scrub_publication_target(object.bytes(), published.generation().sequence())
            .unwrap();
        let record = target.scope().blob_record_identity().unwrap().0;
        assert_eq!(
            target.source(),
            worth_store::physical_runtime::PhysicalIntegrityScrubSource::SelectedRecord(record)
        );
        assert!(matches!(
            blobs.scrub_publication_target([9; 16], published.generation().sequence()),
            Err(worth_store::physical_runtime::BlobScrubTargetFailure::Selection(_))
        ));
        serving.close();
        (object, published.generation().sequence(), record)
    };
    // The C.11 frame remains internally valid after mutation. Only the
    // protected catalog key disagrees with its object field.
    mutate_selected_inner(
        directory.path(),
        worth_store_physical_format::BlobRecordKind::GenerationPublished,
        |frame| {
            frame[C11_HEADER + 32] ^= 1;
        },
    );
    assert_offline_damaged(
        directory.path(),
        original_record,
        "blob_generation_publication",
    );
    let serving = serving_from_open(directory.path());
    let blobs = serving.blobs().unwrap();
    let target = blobs
        .scrub_publication_target(object.bytes(), generation)
        .unwrap();
    assert_eq!(
        target.scope().blob_record_identity().unwrap().0,
        original_record
    );
    assert!(matches!(
        blobs.scrub_publication_target([9; 16], generation),
        Err(worth_store::physical_runtime::BlobScrubTargetFailure::Selection(_))
    ));
    assert_damaged_selected_target(
        &serving,
        target,
        original_record,
        worth_store_physical_format::BlobRecordKind::GenerationPublished,
    );
}

#[test]
fn selected_publication_issues_root_tree_target_before_decoding_wrong_index() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.scrub.tree.issuer");
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
    let (published, original_record) = {
        let serving = serving_from_initialization(directory.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
            object,
            worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
            (2 * CHUNK_BYTES) as u64,
            &scope,
            worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(64).unwrap(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
            .unwrap();
        ingest.push(&vec![0x52; CHUNK_BYTES]).unwrap();
        ingest.push(&vec![0x53; CHUNK_BYTES]).unwrap();
        let published = ingest.finish().unwrap();
        let target = blobs
            .scrub_tree_node_target(published, &scope, 0, 0)
            .unwrap();
        let record = target.scope().blob_record_identity().unwrap().0;
        assert!(matches!(
            blobs.scrub_tree_node_target(published, &scope, (2 * CHUNK_BYTES) as u64, 0),
            Err(worth_store::physical_runtime::BlobScrubTargetFailure::Selection(_))
        ));
        serving.close();
        (published, record)
    };
    mutate_selected_inner(
        directory.path(),
        worth_store_physical_format::BlobRecordKind::TreeNode,
        |frame| {
            // C.11 occurrence index is outside the canonical content digest but
            // inside the encoded frame/root digest.
            frame[C11_HEADER + 32] ^= 1;
        },
    );
    assert_offline_damaged(directory.path(), original_record, "blob_tree_node");
    let serving = serving_from_open(directory.path());
    let blobs = serving.blobs().unwrap();
    let target = blobs
        .scrub_tree_node_target(published, &scope, 0, 0)
        .unwrap();
    assert_eq!(
        target.scope().blob_record_identity().unwrap().0,
        original_record
    );
    assert_damaged_selected_target(
        &serving,
        target,
        original_record,
        worth_store_physical_format::BlobRecordKind::TreeNode,
    );
}
