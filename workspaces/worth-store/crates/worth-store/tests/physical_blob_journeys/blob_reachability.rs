use std::num::{NonZeroU16, NonZeroU64};

use worth_proof::AdmittedBlobReleaseProof;

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReachabilityLimits, BlobReadLimits,
    BlobReclaimLimits, BlobReclaimRequest, BlobRecordReachability, PhysicalIndexPointKey,
    PhysicalMutationDeadline,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{decode_blob_record, BlobRecordV1, DerivedFamilyRootDirectoryV1};

use super::{
    blob_abort::{deadline, limits as terminal_limits, unfinished},
    blob_frontier::selected_blob_records,
    fixture::{admitted_blob_scope, placement, serving_from_initialization},
};

#[cfg(feature = "certification-test-authority")]
#[path = "blob_reachability/reuse.rs"]
mod reuse;

fn inspection_limits() -> BlobReachabilityLimits {
    BlobReachabilityLimits::new(
        NonZeroU64::new(256).unwrap(),
        NonZeroU64::new(64 << 20).unwrap(),
        NonZeroU64::new(8).unwrap(),
        NonZeroU64::new(1024).unwrap(),
    )
}

#[test]
fn selected_session_frontier_is_live_but_aborted_chunk_defers_to_recovery_hold() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.reachability.failed.scope");
    let mut ingest = unfinished(&serving, &scope);
    let token = ingest.resume_token();
    let selected = selected_blob_records(&serving);
    let chunk = selected
        .iter()
        .find_map(|(record, bytes)| {
            matches!(decode_blob_record(bytes), Ok(BlobRecordV1::Chunk(_)))
                .then_some(persisted(*record))
        })
        .expect("real ingest selected a chunk");
    let before = serving
        .blobs()
        .unwrap()
        .classify_selected_reachability(inspection_limits())
        .unwrap();
    assert_eq!(
        class_of(&before, chunk),
        BlobRecordReachability::FailedOperationResidue
    );
    ingest.checkpoint().unwrap();
    let frontier = serving
        .blobs()
        .unwrap()
        .classify_selected_reachability(inspection_limits())
        .unwrap();
    assert_eq!(
        class_of(&frontier, chunk),
        BlobRecordReachability::Reachable
    );
    drop(ingest);
    serving
        .blobs()
        .unwrap()
        .abort_ingest(token, &scope, placement(), deadline(), terminal_limits())
        .unwrap();
    let after = serving
        .blobs()
        .unwrap()
        .classify_selected_reachability(inspection_limits())
        .unwrap();
    assert_eq!(class_of(&after, chunk), BlobRecordReachability::HeldOnly);
    assert!(after.retained_roots_scanned() > 0);
    assert!(selected_blob_records(&serving)
        .iter()
        .any(|(record, _)| persisted(*record) == chunk));
    // The owner-retained recovery predecessor still routes this exact chunk.
    // It cannot be dropped as an external C10 reader can; a new current root
    // retaining the chunk would simply make that new predecessor hold it too.
    assert!(after.selected_records_scanned() >= before.selected_records_scanned());
    serving.close();
}

#[test]
fn published_tree_is_reachable_without_classification_granting_release() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.reachability.published.scope");
    let blobs = serving.blobs().unwrap();
    let read_limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(64 << 10).unwrap(),
        64 << 10,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), 32 << 10, read_limits)
        .unwrap();
    ingest.push(&vec![0x8a; 32 << 10]).unwrap();
    ingest.push(&vec![0x8a; 32 << 10]).unwrap();
    ingest.finish().unwrap();
    let selected_before = selected_blob_records(&serving);
    let observed = blobs
        .classify_selected_reachability(inspection_limits())
        .unwrap();
    for (record, bytes) in &selected_before {
        if matches!(
            decode_blob_record(bytes),
            Ok(BlobRecordV1::Chunk(_)
                | BlobRecordV1::TreeNode(_)
                | BlobRecordV1::GenerationPublished(_))
        ) {
            assert_eq!(
                class_of(&observed, persisted(*record)),
                BlobRecordReachability::Reachable
            );
        }
    }
    assert_eq!(
        selected_blob_records(&serving),
        selected_before,
        "classification is read-only"
    );
    serving.close();
}

#[test]
fn held_c10_root_keeps_displaced_catalog_node_held_only() {
    const CHUNK: usize = 64 << 10;
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.reachability.held.scope");
    let blobs = serving.blobs().unwrap();
    let read_limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let publish = |seed| {
        let object = blobs.issue_object_id(read_limits).unwrap();
        let declaration = BlobIngestDeclaration::new(
            object,
            BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
            CHUNK as u64,
            &scope,
            BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            deadline(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), (CHUNK / 2) as u64, read_limits)
            .unwrap();
        ingest.push(&vec![seed; CHUNK / 2]).unwrap();
        ingest.push(&vec![seed; CHUNK / 2]).unwrap();
        ingest.finish().unwrap();
    };
    publish(0x31);
    let before = selected_blob_records(&serving);
    let old_directory = before
        .iter()
        .find_map(|(record, bytes)| {
            DerivedFamilyRootDirectoryV1::decode(bytes)
                .is_ok()
                .then_some(persisted(*record))
        })
        .expect("first indexed publication selected a directory");
    let held = serving.records().unwrap();
    publish(0x72);
    let observed = blobs
        .classify_selected_reachability(inspection_limits())
        .unwrap();
    assert_eq!(
        class_of(&observed, old_directory),
        BlobRecordReachability::HeldOnly
    );
    assert!(observed.retained_roots_scanned() >= 1);
    let current_count = selected_blob_records(&serving).len() as u64;
    let tight = BlobReachabilityLimits::new(
        NonZeroU64::new(current_count).unwrap(),
        NonZeroU64::new(64 << 20).unwrap(),
        NonZeroU64::new(8).unwrap(),
        NonZeroU64::new(1024).unwrap(),
    );
    assert!(matches!(
        blobs.classify_selected_reachability(tight),
        Err(worth_store::physical_runtime::BlobReachabilityFailure::SelectedBoundExhausted)
    ));
    drop(held);
    serving.close();
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn released_indexed_publication_defers_current_residue_to_recovery_hold() {
    const CHUNK: usize = 64 << 10;
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.reachability.release.scope");
    let blobs = serving.blobs().unwrap();
    let read_limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, read_limits)
        .unwrap();
    ingest.push(&vec![0x31; CHUNK]).unwrap();
    ingest.push(&vec![0x72; CHUNK]).unwrap();
    let published = ingest.finish().unwrap();
    let key =
        PhysicalIndexPointKey::blob_catalog(object, published.generation().sequence()).unwrap();
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
        .unwrap();
    let before = selected_blob_records(&serving);
    let derived = before
        .iter()
        .find_map(|(record, bytes)| {
            DerivedFamilyRootDirectoryV1::decode(bytes)
                .is_ok()
                .then_some(persisted(*record))
        })
        .expect("Store published a selected derived directory");
    let chunk = before
        .iter()
        .find_map(|(record, bytes)| {
            matches!(decode_blob_record(bytes), Ok(BlobRecordV1::Chunk(_)))
                .then_some(persisted(*record))
        })
        .expect("Store selected a chunk");
    let proof = AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        selected.record().allocation_epoch(),
        selected.record().ordinal(),
        selected.encoded_digest(),
        [0x91; 32],
    )
    .unwrap();
    let receipt = blobs
        .reclaim(BlobReclaimRequest::released(
            proof,
            placement(),
            deadline(),
            BlobReclaimLimits::new(
                NonZeroU64::new(256).unwrap(),
                NonZeroU64::new(64 << 20).unwrap(),
                NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        ))
        .unwrap()
        .wait()
        .unwrap();
    assert!(receipt.dropped_records().contains(&selected.record()));
    assert!(receipt.remaining_payload_records() > 0);
    let observed = blobs
        .classify_selected_reachability(inspection_limits())
        .unwrap();
    assert!(
        !observed
            .records()
            .iter()
            .any(|row| row.record() == selected.record()),
        "an unheld dropped publication is not a selected classification candidate"
    );
    assert!(
        observed.records().iter().any(|row| row.record() == derived),
        "selected derived directory was removed, not residue"
    );
    assert!(
        observed.records().iter().any(|row| row.record() == chunk),
        "selected chunk was removed, not unreferenced"
    );
    assert_eq!(
        class_of(&observed, derived),
        BlobRecordReachability::HeldOnly
    );
    assert_eq!(class_of(&observed, chunk), BlobRecordReachability::HeldOnly);
    assert!(observed.retained_roots_scanned() > 0);
    // Both exact routes are still current-selected and are also present in
    // the recovery predecessor; the deferred classes are checked at the
    // inventory boundary after the held root is removed.
    // This obsolete payload remains on the current selected route while a
    // C10 reader protects that same route across a later root publication.
    let held = serving.records().unwrap();
    let unrelated = admitted_blob_scope("c11.blob.reachability.held.current");
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        CHUNK as u64,
        &unrelated,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), (CHUNK / 2) as u64, read_limits)
        .unwrap();
    ingest.push(&vec![0xa5; CHUNK / 2]).unwrap();
    ingest.push(&vec![0xa5; CHUNK / 2]).unwrap();
    ingest.finish().unwrap();
    assert!(selected_blob_records(&serving)
        .iter()
        .any(|(record, _)| persisted(*record) == chunk));
    let protected = blobs
        .classify_selected_reachability(inspection_limits())
        .unwrap();
    assert_eq!(
        class_of(&protected, chunk),
        BlobRecordReachability::HeldOnly
    );
    drop(held);
    let after_release = blobs
        .classify_selected_reachability(inspection_limits())
        .unwrap();
    assert_eq!(
        class_of(&after_release, chunk),
        BlobRecordReachability::HeldOnly
    );
    assert!(after_release.retained_roots_scanned() > 0);
    serving.close();
}

fn class_of(
    observed: &worth_store::physical_runtime::BlobReachabilityObservation,
    record: worth_store_physical_format::PersistedRecordIdentity,
) -> BlobRecordReachability {
    observed
        .records()
        .iter()
        .find(|row| row.record() == record)
        .expect("selected record is present in bounded observation")
        .class()
}

fn persisted(
    record: worth_store::physical_runtime::PhysicalRecordId,
) -> worth_store_physical_format::PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new(
        record.allocation_epoch(),
        record.ordinal(),
    )
    .unwrap()
}
