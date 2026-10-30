#![cfg(all())]

use std::num::NonZeroU64;

use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;
use {
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::LayoutRebuildFailure,
    worth_store::physical_runtime::LayoutRebuildLimits,
    worth_store::physical_runtime::PhysicalIndexPointKey,
    worth_store::physical_runtime::PhysicalMutationDeadline,
};

use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

#[test]
fn full_selected_authority_rebuild_restores_catalog_and_dedupe_after_derived_root_drop() {
    const CHUNK_BYTES: usize = 64 * 1024;
    let root = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(root.path());
    let scope = admitted_blob_scope("c11.layout.full-authority-rebuild");
    let read_limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        (2 * CHUNK_BYTES) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, read_limits)
        .unwrap();
    ingest.push(&vec![0x31; CHUNK_BYTES]).unwrap();
    ingest.push(&vec![0x72; CHUNK_BYTES]).unwrap();
    let published = ingest.finish().unwrap();
    let key = worth_store::physical_runtime::PhysicalIndexPointKey::blob_catalog(
        object,
        published.generation().sequence(),
    )
    .unwrap();
    let before = serving
        .layouts()
        .unwrap()
        .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
        .unwrap()
        .point(key)
        .unwrap()
        .selected_record()
        .unwrap();
    let held_layout = serving.layouts().unwrap();
    let retired = serving
        .certification_drop_blob_derived_roots(
            placement(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        held_layout
            .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(key)
            .unwrap()
            .selected_record(),
        Some(before),
        "a previously held protected root keeps its old catalog path"
    );
    for &record in &retired {
        assert!(
            !serving.certification_selected_layout_record(record),
            "retired directory and derived path nodes cannot remain current-selected"
        );
    }
    drop(held_layout);
    assert_eq!(
        serving
            .layouts()
            .unwrap()
            .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(key)
            .unwrap()
            .selected_record(),
        None
    );
    let insufficient = worth_store::physical_runtime::LayoutRebuildLimits::new(
        std::num::NonZeroU64::new(1).unwrap(),
        std::num::NonZeroU64::new(1).unwrap(),
    );
    assert!(matches!(
        serving.layouts().unwrap().rebuild(
            worth_store_contracts::DurableArtifactFamilyId::BlobCatalog,
            insufficient,
            placement(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
        ),
        Err(worth_store::physical_runtime::LayoutRebuildFailure::SelectedBoundExhausted)
    ));
    assert_eq!(
        serving
            .layouts()
            .unwrap()
            .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(key)
            .unwrap()
            .selected_record(),
        None
    );
    let bounded = worth_store::physical_runtime::LayoutRebuildLimits::new(
        std::num::NonZeroU64::new(1_000).unwrap(),
        std::num::NonZeroU64::new(1_000).unwrap(),
    );
    let rebuilt = serving
        .layouts()
        .unwrap()
        .rebuild(
            worth_store_contracts::DurableArtifactFamilyId::DedupeIndex,
            bounded,
            placement(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(rebuilt.validated_chunks(), 2);
    assert!(rebuilt.dedupe_root().is_some());
    assert!(
        !retired.contains(&rebuilt.catalog_root()),
        "byte-identical catalog rebuild must not replay a retired C.5 RecordId"
    );
    assert!(
        !retired.contains(&rebuilt.dedupe_root().unwrap()),
        "byte-identical dedupe rebuild must not replay a retired C.5 RecordId"
    );
    assert!(serving.certification_selected_layout_record(rebuilt.catalog_root()));
    assert!(serving.certification_selected_layout_record(rebuilt.dedupe_root().unwrap()));
    assert_eq!(
        serving
            .layouts()
            .unwrap()
            .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(key)
            .unwrap()
            .selected_record(),
        Some(before)
    );
    serving.close();
    let reopened = serving_from_open(root.path());
    assert!(reopened.certification_selected_layout_record(rebuilt.catalog_root()));
    assert!(reopened.certification_selected_layout_record(rebuilt.dedupe_root().unwrap()));
    assert_eq!(
        reopened
            .layouts()
            .unwrap()
            .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(key)
            .unwrap()
            .selected_record(),
        Some(before)
    );
    reopened.close();
}

#[test]
fn held_catalog_root_survives_ordinary_second_publication_cow_and_reopen() {
    const CHUNK_BYTES: usize = 64 * 1024;
    let root = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(root.path());
    let scope = admitted_blob_scope("c11.layout.held-cow-root");
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let blobs = serving.blobs().unwrap();
    let deadline =
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap();
    let first = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        first,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline,
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), (CHUNK_BYTES / 2) as u64, limits)
        .unwrap();
    ingest.push(&vec![0x31; CHUNK_BYTES / 2]).unwrap();
    ingest.push(&vec![0x31; CHUNK_BYTES / 2]).unwrap();
    let first_published = ingest.finish().unwrap();
    let first_key = worth_store::physical_runtime::PhysicalIndexPointKey::blob_catalog(
        first,
        first_published.generation().sequence(),
    )
    .unwrap();
    let held = serving.layouts().unwrap();
    let first_record = held
        .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
        .unwrap()
        .point(first_key)
        .unwrap()
        .selected_record()
        .unwrap();

    let second = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        second,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline,
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), (CHUNK_BYTES / 2) as u64, limits)
        .unwrap();
    ingest.push(&vec![0x72; CHUNK_BYTES / 2]).unwrap();
    ingest.push(&vec![0x72; CHUNK_BYTES / 2]).unwrap();
    let second_published = ingest.finish().unwrap();
    let second_key = worth_store::physical_runtime::PhysicalIndexPointKey::blob_catalog(
        second,
        second_published.generation().sequence(),
    )
    .unwrap();
    assert_eq!(
        held.btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(first_key)
            .unwrap()
            .selected_record(),
        Some(first_record)
    );
    assert_eq!(
        held.btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(second_key)
            .unwrap()
            .selected_record(),
        None
    );
    let current = serving.layouts().unwrap();
    assert_eq!(
        current
            .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(first_key)
            .unwrap()
            .selected_record(),
        Some(first_record)
    );
    assert!(current
        .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
        .unwrap()
        .point(second_key)
        .unwrap()
        .selected_record()
        .is_some());
    drop(current);
    drop(held);
    serving.close();
    let reopened = serving_from_open(root.path());
    assert_eq!(
        reopened
            .layouts()
            .unwrap()
            .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(first_key)
            .unwrap()
            .selected_record(),
        Some(first_record)
    );
    assert!(reopened
        .layouts()
        .unwrap()
        .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
        .unwrap()
        .point(second_key)
        .unwrap()
        .selected_record()
        .is_some());
    reopened.close();
}
