use std::num::{NonZeroU16, NonZeroU64};

use worth_proof::AdmittedBlobReleaseProof;
use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadFailure, BlobReadLimits,
    BlobReadOpenFailure, BlobReclaimDisposition, BlobReclaimLimits, BlobReclaimRequest,
    PhysicalIndexPointKey, PhysicalMutationDeadline, RecordReadDenial,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;

use super::blob_ingest_process::observe_closed_store_named;
use super::fixture::{
    admitted_blob_scope, placement, serving_from_initialization, serving_from_open,
};

#[test]
fn published_blob_catalog_point_uses_one_protected_leaf_after_fresh_reopen() {
    const CHUNK_BYTES: usize = 64 * 1024;
    let root = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(root.path());
    let scope = admitted_blob_scope("c11.layout.catalog.point");
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        (2 * CHUNK_BYTES) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
        .unwrap();
    ingest.push(&vec![0x31; CHUNK_BYTES]).unwrap();
    ingest.push(&vec![0x72; CHUNK_BYTES]).unwrap();
    let published = ingest.finish().unwrap();
    drop(blobs);

    let key = PhysicalIndexPointKey::blob_catalog(object, published.generation().sequence())
        .expect("Store-issued blob identity is an admitted catalog key");
    let before_close = serving
        .layouts()
        .unwrap()
        .btree(DurableArtifactFamilyId::BlobCatalog)
        .unwrap()
        .point(key)
        .unwrap();
    let selected = before_close.selected_record().expect("indexed publication");
    let independent_publication = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("selected C.5 publication");
    assert_eq!(selected, independent_publication.record());
    assert_eq!(before_close.counters().page_touches(), 1);
    serving.close();

    let reopened = serving_from_open(root.path());
    let after_reopen = reopened
        .layouts()
        .unwrap()
        .btree(DurableArtifactFamilyId::BlobCatalog)
        .unwrap()
        .point(key)
        .unwrap();
    assert_eq!(after_reopen.selected_record(), Some(selected));
    assert_eq!(
        reopened
            .certification_selected_latest_blob_publication()
            .unwrap()
            .unwrap()
            .record(),
        selected,
    );
    assert_eq!(after_reopen.counters().page_touches(), 1);
    assert_eq!(
        reopened
            .blobs()
            .unwrap()
            .resolve_publication(
                object.bytes(),
                published.generation().sequence(),
                &scope,
                limits
            )
            .unwrap(),
        published,
    );
    reopened.close();

    let report = observe_closed_store_named(root.path(), "c11-layout-catalog", "selected-point");
    assert_eq!(report["completeness"], "complete", "observer incomplete");
    let artifacts = report["artifacts"].as_array().unwrap();
    for family in ["derived_family_root_directory", "btree_node"] {
        let matching = artifacts
            .iter()
            .filter(|row| row["family"] == family)
            .collect::<Vec<_>>();
        if matching.is_empty() {
            let roots = artifacts
                .iter()
                .filter(|row| row["family"] == "root_manifest")
                .map(|row| (&row["generation"], &row["outcome"], &row["range"]))
                .collect::<Vec<_>>();
            let families = artifacts
                .iter()
                .map(|row| &row["family"])
                .collect::<Vec<_>>();
            panic!("missing {family}; roots={roots:?}; families={families:?}");
        }
        let non_intact = matching
            .iter()
            .filter(|row| row["outcome"]["posture"] != "intact")
            .map(|row| (&row["identity"], &row["outcome"]))
            .collect::<Vec<_>>();
        assert!(non_intact.is_empty(), "{family}: {non_intact:?}");
    }
}

#[test]
fn released_indexed_publication_invalidates_selected_catalog_directory() {
    const CHUNK_BYTES: usize = 64 * 1024;
    let root = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(root.path());
    let scope = admitted_blob_scope("c11.layout.catalog.released");
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        (2 * CHUNK_BYTES) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
        .unwrap();
    ingest.push(&vec![0x31; CHUNK_BYTES]).unwrap();
    ingest.push(&vec![0x72; CHUNK_BYTES]).unwrap();
    let published = ingest.finish().unwrap();
    drop(blobs);

    let key = PhysicalIndexPointKey::blob_catalog(object, published.generation().sequence())
        .expect("Store-issued catalog key");
    assert!(serving
        .layouts()
        .unwrap()
        .btree(DurableArtifactFamilyId::BlobCatalog)
        .unwrap()
        .point(key)
        .unwrap()
        .selected_record()
        .is_some());
    let selected = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("indexed publication remains selected");
    let proof = AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        selected.record().allocation_epoch(),
        selected.record().ordinal(),
        selected.encoded_digest(),
        [0x81; 32],
    )
    .unwrap();
    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(BlobReclaimRequest::released(
            proof,
            placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            BlobReclaimLimits::new(
                NonZeroU64::new(256).unwrap(),
                NonZeroU64::new(64 << 20).unwrap(),
                NonZeroU16::new(1024).unwrap(),
            )
            .unwrap(),
        ))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    assert!(receipt.dropped_records().contains(&selected.record()));
    assert!(serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .is_none());
    let after = serving
        .layouts()
        .unwrap()
        .btree(DurableArtifactFamilyId::BlobCatalog)
        .expect("stale directory binding must not break layout admission")
        .point(key)
        .unwrap();
    // The replacement directory retains the catalog root, so the released
    // publication's cell is residue; resolution cannot route it.
    assert_eq!(after.selected_record(), Some(selected.record()));
    assert!(matches!(
        serving.blobs().unwrap().resolve_publication(
            object.bytes(),
            published.generation().sequence(),
            &scope,
            limits,
        ),
        Err(BlobReadOpenFailure::Read(BlobReadFailure::RecordRead(error)))
            if error.denial() == RecordReadDenial::RecordNotFound
    ));
    serving.close();
}
