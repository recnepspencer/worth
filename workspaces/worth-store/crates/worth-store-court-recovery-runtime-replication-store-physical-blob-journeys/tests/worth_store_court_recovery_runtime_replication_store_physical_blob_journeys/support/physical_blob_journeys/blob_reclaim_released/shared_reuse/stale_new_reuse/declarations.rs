pub(crate) use worth_store_contracts::DurableArtifactFamilyId;
pub(crate) use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::PhysicalIndexPointKey,
    worth_store::physical_runtime::PublishedBlobGeneration,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

pub(crate) use crate::physical_blob_journeys::blob_reclaim_released::publish_one;
pub(crate) use {
    crate::physical_blob_journeys::blob_reclaim_released::shared_reuse::assert_bytes_exact,
    crate::physical_blob_journeys::blob_reclaim_released::shared_reuse::marker,
};

/// A release retains the dedupe cells naming the released source. A fresh
/// ingest of the same bytes must miss those stale cells, publish its own
/// chunks, and replace the cells so later ingests reuse the new publication.
pub(crate) fn assert_fresh_ingest_replaces_stale_cells(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    payload: &[u8],
) {
    let fresh = publish_one(serving, scope, payload);
    let fresh_marker = marker(serving);
    assert_catalog_selects(serving, fresh, fresh_marker.record());
    assert_eq!(assert_bytes_exact(serving, scope, fresh, payload), 0);
    assert_eq!(claims_sourced_from(serving, fresh_marker.record()), 0);

    crate::physical_blob_journeys::blob_expiry::completed_checkpoint(serving, 0xd7);
    let successor = publish_one(serving, scope, payload);
    assert_eq!(
        claims_sourced_from(serving, fresh_marker.record()),
        2,
        "replaced dedupe cells must route later reuse to the fresh publication",
    );
    assert!(assert_bytes_exact(serving, scope, successor, payload) > 0);
}

pub(crate) fn assert_catalog_selects(
    serving: &ServingPhysicalRuntime,
    published: PublishedBlobGeneration,
    expected: PersistedRecordIdentity,
) {
    let key = worth_store::physical_runtime::PhysicalIndexPointKey::blob_catalog(
        published.object(),
        published.generation().sequence(),
    )
    .unwrap();
    let point = serving
        .layouts()
        .unwrap()
        .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
        .unwrap()
        .point(key)
        .unwrap();
    assert_eq!(point.selected_record(), Some(expected));
}

pub(crate) fn claims_sourced_from(
    serving: &ServingPhysicalRuntime,
    source: PersistedRecordIdentity,
) -> usize {
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records(serving)
        .iter()
        .filter(|(_, bytes)| match decode_blob_record(bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::ChunkReuseClaimV2(value)) => {
                value.claim().source_publication() == source
            }
            _ => false,
        })
        .count()
}
