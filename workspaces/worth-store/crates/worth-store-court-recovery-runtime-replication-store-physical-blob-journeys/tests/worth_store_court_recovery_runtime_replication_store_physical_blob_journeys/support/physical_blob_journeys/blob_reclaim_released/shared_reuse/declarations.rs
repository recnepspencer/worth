pub(crate) use {std::num::NonZeroU16, std::num::NonZeroU64};

pub(crate) use worth_proof::AdmittedBlobReleaseProof;
pub(crate) use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobReadFailure, worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReadOpenFailure,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimReceipt,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::BlobReclaimRetirement,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PublishedBlobGeneration,
    worth_store::physical_runtime::RecordReadDenial,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::IndexedThroughBlobPublication,
};

pub(crate) use {
    crate::physical_blob_journeys::blob_reclaim_released::publish_one,
    crate::physical_blob_journeys::blob_reclaim_released::shared_format,
    crate::physical_blob_journeys::blob_reclaim_released::shared_placement,
    crate::physical_blob_journeys::blob_reclaim_released::SHARED_CHUNK as CHUNK,
};
pub(crate) use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::serving_from_initialization_with_format_and_placement,
};

pub(crate) fn source_first() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization_with_format_and_placement(
        directory.path(),
        shared_format(),
        shared_placement(),
    );
    let scope = admitted_blob_scope("c11.blob.release.shared.source.first");
    let payload = multi_chunk_payload();
    let original = publish_one(&serving, &scope, &payload);
    let original_marker = marker(&serving);
    crate::physical_blob_journeys::blob_expiry::completed_checkpoint(&serving, 0xd1);
    let reused = publish_one(&serving, &scope, &payload);
    assert_selected_reuse_claim(&serving);
    for index in 0..4_u8 {
        let unrelated = vec![0x20 + index; CHUNK];
        publish_one(&serving, &scope, &unrelated);
        crate::physical_blob_journeys::blob_expiry::completed_checkpoint(&serving, 0xd2 + index);
    }

    let source_first = release(&serving, original, original_marker);
    assert_retired(&source_first);
    assert!(source_first
        .dropped_records()
        .contains(&original_marker.record()));
    assert!(source_first.remaining_payload_records() > 0);
    assert_invisible(&serving, &scope, original);
    assert_eq!(assert_bytes_exact(&serving, &scope, reused, &payload), 4);
    crate::physical_blob_journeys::blob_reclaim_released::shared_reuse::stale_new_reuse::assert_fresh_ingest_replaces_stale_cells(&serving, &scope, &payload);
    crate::physical_blob_journeys::blob_expiry::completed_checkpoint(&serving, 0xd6);
    serving.close();

    crate::physical_blob_journeys::blob_reclaim_released::shared_reuse::fresh_process::assert_open_denied(directory.path());
    crate::physical_blob_journeys::blob_reclaim_released::shared_reuse::denied_reopen::assert_open_requires_c8_custody(directory.path());
}

pub(crate) fn marker(serving: &ServingPhysicalRuntime) -> IndexedThroughBlobPublication {
    serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("published generation selected")
}

pub(crate) fn request(
    serving: &ServingPhysicalRuntime,
    published: PublishedBlobGeneration,
    marker: IndexedThroughBlobPublication,
) -> BlobReclaimRequest<'_> {
    let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        published.object().bytes(),
        published.generation().sequence(),
        marker.record().allocation_epoch(),
        marker.record().ordinal(),
        marker.encoded_digest(),
        [0x73; 32],
    )
    .unwrap();
    worth_store::physical_runtime::BlobReclaimRequest::released(
        proof,
        shared_placement(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(512).unwrap(),
            std::num::NonZeroU64::new(64 << 20).unwrap(),
            std::num::NonZeroU16::new(16).unwrap(),
        )
        .unwrap(),
    )
}

pub(crate) fn release(
    serving: &ServingPhysicalRuntime,
    published: PublishedBlobGeneration,
    marker: IndexedThroughBlobPublication,
) -> BlobReclaimReceipt {
    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(request(serving, published, marker))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        receipt.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped
    );
    receipt
}

pub(crate) fn assert_retired(receipt: &BlobReclaimReceipt) {
    assert_eq!(
        receipt.retirement(),
        worth_store::physical_runtime::BlobReclaimRetirement::Completed
    );
    assert!(receipt.bytes_released() > 0);
    assert_eq!(
        receipt.bytes_released(),
        receipt
            .displaced_extents()
            .iter()
            .map(|extent| extent.range().length())
            .sum::<u64>()
    );
}

pub(crate) fn assert_invisible(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    published: PublishedBlobGeneration,
) {
    let result = serving.blobs().unwrap().resolve_publication(
        published.object().bytes(),
        published.generation().sequence(),
        scope,
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(1).unwrap()),
    );
    match result {
        Err(worth_store::physical_runtime::BlobReadOpenFailure::PublicationNotFound) => {}
        Err(worth_store::physical_runtime::BlobReadOpenFailure::Read(
            worth_store::physical_runtime::BlobReadFailure::RecordRead(error),
        )) if error.denial() == worth_store::physical_runtime::RecordReadDenial::RecordNotFound => {
        }
        other => panic!("released publication has wrong visibility: {other:?}"),
    }
}

pub(crate) fn assert_bytes_exact(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    published: PublishedBlobGeneration,
    expected: &[u8],
) -> u64 {
    let blobs = serving.blobs().unwrap();
    let mut read = blobs
        .read(
            published,
            scope,
            0,
            expected.len() as u64,
            worth_store::physical_runtime::BlobReadLimits::new(
                std::num::NonZeroU64::new(1).unwrap(),
            ),
        )
        .unwrap();
    let mut bytes = vec![0_u8; expected.len()];
    let mut used = 0;
    while used < bytes.len() {
        let count = read.read_next(&mut bytes[used..]).unwrap();
        assert!(count > 0);
        used += count;
    }
    assert_eq!(read.read_next(&mut bytes).unwrap(), 0);
    assert_eq!(&bytes, expected);
    assert_eq!(read.observation().touched_chunks(), 2);
    read.observation().reuse_source_selected_reads()
}

pub(crate) fn multi_chunk_payload() -> Vec<u8> {
    let mut bytes = vec![0x5a; CHUNK + 17];
    bytes[CHUNK..].fill(0x7c);
    bytes
}

pub(crate) fn assert_selected_reuse_claim(serving: &ServingPhysicalRuntime) {
    assert_eq!(
        worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records(
            serving
        )
        .iter()
        .filter(|(_, bytes)| matches!(
            decode_blob_record(bytes),
            Ok(worth_store_physical_format::BlobRecordV1::ChunkReuseClaimV2(_))
        ))
        .count(),
        2
    );
}
