#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_reachability::reuse::*;

use worth_store_physical_format::decode_blob_record;

use {
    crate::physical_blob_journeys::blob_abort::deadline,
    crate::physical_blob_journeys::blob_reachability::class_of,
    crate::physical_blob_journeys::blob_reachability::inspection_limits,
    crate::physical_blob_journeys::blob_reachability::persisted,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};
use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::serving_from_initialization_with_format_and_placement,
};

#[test]
fn released_source_chunk_stays_reachable_through_live_v2_reuse() {
    let directory = tempfile::tempdir().unwrap();
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .page_size(worth_store::physical_runtime::PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    );
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(512).unwrap())
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
    crate::physical_blob_journeys::blob_expiry::completed_checkpoint(&serving, 0xb1);
    let destination = publish(&serving, &scope, placement, &payload);
    let selected = selected_blob_records(&serving);
    let source_session = selected
        .iter()
        .find_map(|(record, bytes)| {
            if persisted(*record) != marker.record() {
                return None;
            }
            match decode_blob_record(bytes) {
                Ok(worth_store_physical_format::BlobRecordV1::GenerationPublished(value)) => {
                    Some(value.session())
                }
                _ => None,
            }
        })
        .expect("selected source publication yields its authenticated session");
    let source_path_nodes = selected
        .iter()
        .filter_map(|(record, bytes)| match decode_blob_record(bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::TreeNode(value))
                if value.occurrence().session() == source_session =>
            {
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
            Ok(worth_store_physical_format::BlobRecordV1::ChunkReuseClaimV2(value)) => {
                Some(value.claim().selected_chunk())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        source_chunks.len(),
        2,
        "real destination must reuse both source chunks"
    );
    let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
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
        .reclaim(worth_store::physical_runtime::BlobReclaimRequest::released(
            proof,
            placement,
            deadline(),
            worth_store::physical_runtime::BlobReclaimLimits::new(
                std::num::NonZeroU64::new(512).unwrap(),
                std::num::NonZeroU64::new(64 << 20).unwrap(),
                std::num::NonZeroU16::new(16).unwrap(),
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
            worth_store::physical_runtime::BlobRecordReachability::Reachable
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
            worth_store::physical_runtime::BlobReadLimits::new(
                std::num::NonZeroU64::new(128).unwrap(),
            ),
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
