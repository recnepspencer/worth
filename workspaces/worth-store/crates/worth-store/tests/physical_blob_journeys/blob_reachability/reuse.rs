use std::num::{NonZeroU16, NonZeroU64};

use worth_proof::AdmittedBlobReleaseProof;
use worth_store::physical_runtime::{
    AdmittedBlobScope, AdmittedPhysicalRecordFormat, BlobCheckpointLimit, BlobIngestDeclaration,
    BlobReadLimits, BlobReclaimLimits, BlobReclaimRequest, BlobRecordReachability,
    ManifestEntryCapacity, PhysicalPageSizeClass, PhysicalRecordFormatDeclaration,
    PhysicalRecordPlacementPolicy, PublishedBlobGeneration, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{decode_blob_record, BlobRecordV1};

use super::{class_of, deadline, inspection_limits, persisted, selected_blob_records};
use crate::fixture::{admitted_blob_scope, serving_from_initialization_with_format_and_placement};

const CHUNK: usize = 256 << 10;

#[test]
fn released_source_chunk_stays_reachable_through_live_v2_reuse() {
    let directory = tempfile::tempdir().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .page_size(PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(512).unwrap())
        .admit(format)
        .unwrap();
    let serving =
        serving_from_initialization_with_format_and_placement(directory.path(), format, placement);
    let scope = admitted_blob_scope("c11.blob.reachability.reused.release");
    let mut payload = vec![0x5a; CHUNK + 17];
    payload[CHUNK..].fill(0x7c);
    let source = publish(&serving, &scope, placement, &payload);
    let marker = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .unwrap();
    crate::blob_expiry::completed_checkpoint(&serving, 0xb1);
    let destination = publish(&serving, &scope, placement, &payload);
    let selected = selected_blob_records(&serving);
    let source_session = selected
        .iter()
        .find_map(|(record, bytes)| {
            if persisted(*record) != marker.record() {
                return None;
            }
            match decode_blob_record(bytes) {
                Ok(BlobRecordV1::GenerationPublished(value)) => Some(value.session()),
                _ => None,
            }
        })
        .expect("selected source publication yields its authenticated session");
    let source_path_nodes = selected
        .iter()
        .filter_map(|(record, bytes)| match decode_blob_record(bytes) {
            Ok(BlobRecordV1::TreeNode(value)) if value.occurrence().session() == source_session => {
                Some(persisted(*record))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        !source_path_nodes.is_empty(),
        "source has a selected tree path"
    );
    let source_chunks = selected
        .iter()
        .filter_map(|(_, bytes)| match decode_blob_record(bytes) {
            Ok(BlobRecordV1::ChunkReuseClaimV2(value)) => Some(value.claim().selected_chunk()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        source_chunks.len(),
        2,
        "real destination must reuse both source chunks"
    );
    let proof = AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        source.object().bytes(),
        source.generation().sequence(),
        marker.record().allocation_epoch(),
        marker.record().ordinal(),
        marker.encoded_digest(),
        [0xb2; 32],
    )
    .unwrap();
    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(BlobReclaimRequest::released(
            proof,
            placement,
            deadline(),
            BlobReclaimLimits::new(
                NonZeroU64::new(512).unwrap(),
                NonZeroU64::new(64 << 20).unwrap(),
                NonZeroU16::new(16).unwrap(),
            )
            .unwrap(),
        ))
        .unwrap()
        .wait()
        .unwrap();
    assert!(receipt.dropped_records().contains(&marker.record()));
    assert!(receipt.remaining_payload_records() > 0);
    let current = selected_blob_records(&serving);
    for source_chunk in source_chunks.iter().chain(source_path_nodes.iter()) {
        assert!(current
            .iter()
            .any(|(record, _)| persisted(*record) == *source_chunk));
    }
    let observed = serving
        .blobs()
        .unwrap()
        .classify_selected_reachability(inspection_limits())
        .unwrap();
    for source_chunk in source_chunks.into_iter().chain(source_path_nodes) {
        assert_eq!(
            class_of(&observed, source_chunk),
            BlobRecordReachability::Reachable
        );
    }
    let mut read = serving
        .blobs()
        .unwrap()
        .read(
            destination,
            &scope,
            0,
            payload.len() as u64,
            BlobReadLimits::new(NonZeroU64::new(128).unwrap()),
        )
        .unwrap();
    let mut bytes = vec![0; payload.len()];
    let mut used = 0;
    while used < bytes.len() {
        let count = read.read_next(&mut bytes[used..]).unwrap();
        assert!(count > 0);
        used += count;
    }
    assert_eq!(bytes, payload);
    assert!(read.observation().reuse_source_selected_reads() > 0);
    drop(read);
    serving.close();
}

fn publish(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    payload: &[u8],
) -> PublishedBlobGeneration {
    let blobs = serving.blobs().unwrap();
    let read_limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        payload.len() as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement, (CHUNK / 2) as u64, read_limits)
        .unwrap();
    for piece in payload.chunks(CHUNK / 2) {
        ingest.push(piece).unwrap();
    }
    ingest.finish().unwrap()
}
