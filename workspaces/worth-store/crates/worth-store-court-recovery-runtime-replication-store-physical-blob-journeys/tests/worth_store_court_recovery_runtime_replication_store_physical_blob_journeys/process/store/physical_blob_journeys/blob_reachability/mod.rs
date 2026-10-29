#[allow(unused_imports)] use crate::physical_blob_journeys::blob_reachability::*;
use {std::num::NonZeroU16, std::num::NonZeroU64};

use {worth_proof::AdmittedBlobReleaseProof};

use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReachabilityLimits, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::BlobRecordReachability, worth_store::physical_runtime::PhysicalIndexPointKey, worth_store::physical_runtime::PhysicalMutationDeadline};
use {worth_store_blob_chunks::BlobChunkSize};
use {worth_store_contracts::DurableArtifactFamilyId};
use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::DerivedFamilyRootDirectoryV1};

use {crate::physical_blob_journeys::blob_abort::deadline, crate::physical_blob_journeys::blob_abort::limits as terminal_limits, crate::physical_blob_journeys::blob_abort::unfinished, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};

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
            matches!(decode_blob_record(bytes), Ok(worth_store_physical_format::BlobRecordV1::Chunk(_)))
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
        worth_store::physical_runtime::BlobRecordReachability::FailedOperationResidue
    );
    ingest.checkpoint().unwrap();
    let frontier = serving
        .blobs()
        .unwrap()
        .classify_selected_reachability(inspection_limits())
        .unwrap();
    assert_eq!(
        class_of(&frontier, chunk),
        worth_store::physical_runtime::BlobRecordReachability::Reachable
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
    assert_eq!(class_of(&after, chunk), worth_store::physical_runtime::BlobRecordReachability::HeldOnly);
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
    let read_limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(64 << 10).unwrap(),
        64 << 10,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
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
            Ok(worth_store_physical_format::BlobRecordV1::Chunk(_)
                | worth_store_physical_format::BlobRecordV1::TreeNode(_)
                | worth_store_physical_format::BlobRecordV1::GenerationPublished(_))
        ) {
            assert_eq!(
                class_of(&observed, persisted(*record)),
                worth_store::physical_runtime::BlobRecordReachability::Reachable
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
    let read_limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let publish = |seed| {
        let object = blobs.issue_object_id(read_limits).unwrap();
        let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
            object,
            worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
            CHUNK as u64,
            &scope,
            worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
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
            worth_store_physical_format::DerivedFamilyRootDirectoryV1::decode(bytes)
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
        worth_store::physical_runtime::BlobRecordReachability::HeldOnly
    );
    assert!(observed.retained_roots_scanned() >= 1);
    let current_count = selected_blob_records(&serving).len() as u64;
    let tight = worth_store::physical_runtime::BlobReachabilityLimits::new(
        std::num::NonZeroU64::new(current_count).unwrap(),
        std::num::NonZeroU64::new(64 << 20).unwrap(),
        std::num::NonZeroU64::new(8).unwrap(),
        std::num::NonZeroU64::new(1024).unwrap(),
    );
    assert!(matches!(
        blobs.classify_selected_reachability(tight),
        Err(worth_store::physical_runtime::BlobReachabilityFailure::SelectedBoundExhausted)
    ));
    drop(held);
    serving.close();
}

#[cfg(all())]
#[test]
fn released_indexed_publication_defers_current_residue_to_recovery_hold() {
    const CHUNK: usize = 64 << 10;
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.reachability.release.scope");
    let blobs = serving.blobs().unwrap();
    let read_limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
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
        worth_store::physical_runtime::PhysicalIndexPointKey::blob_catalog(object, published.generation().sequence()).unwrap();
    assert!(serving
        .layouts()
        .unwrap()
        .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
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
            worth_store_physical_format::DerivedFamilyRootDirectoryV1::decode(bytes)
                .is_ok()
                .then_some(persisted(*record))
        })
        .expect("Store published a selected derived directory");
    let chunk = before
        .iter()
        .find_map(|(record, bytes)| {
            matches!(decode_blob_record(bytes), Ok(worth_store_physical_format::BlobRecordV1::Chunk(_)))
                .then_some(persisted(*record))
        })
        .expect("Store selected a chunk");
    let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
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
        .reclaim(worth_store::physical_runtime::BlobReclaimRequest::released(
            proof,
            placement(),
            deadline(),
            worth_store::physical_runtime::BlobReclaimLimits::new(
                std::num::NonZeroU64::new(256).unwrap(),
                std::num::NonZeroU64::new(64 << 20).unwrap(),
                std::num::NonZeroU16::new(1).unwrap(),
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
        worth_store::physical_runtime::BlobRecordReachability::HeldOnly
    );
    assert_eq!(class_of(&observed, chunk), worth_store::physical_runtime::BlobRecordReachability::HeldOnly);
    assert!(observed.retained_roots_scanned() > 0);
    // Both exact routes are still current-selected and are also present in
    // the recovery predecessor; the deferred classes are checked at the
    // inventory boundary after the held root is removed.
    // This obsolete payload remains on the current selected route while a
    // C10 reader protects that same route across a later root publication.
    let held = serving.records().unwrap();
    let unrelated = admitted_blob_scope("c11.blob.reachability.held.current");
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        CHUNK as u64,
        &unrelated,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
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
        worth_store::physical_runtime::BlobRecordReachability::HeldOnly
    );
    drop(held);
    let after_release = blobs
        .classify_selected_reachability(inspection_limits())
        .unwrap();
    assert_eq!(
        class_of(&after_release, chunk),
        worth_store::physical_runtime::BlobRecordReachability::HeldOnly
    );
    assert!(after_release.retained_roots_scanned() > 0);
    serving.close();
}
#[cfg(all())]
#[path = "reuse/mod.rs"]
pub(crate) mod reuse;
