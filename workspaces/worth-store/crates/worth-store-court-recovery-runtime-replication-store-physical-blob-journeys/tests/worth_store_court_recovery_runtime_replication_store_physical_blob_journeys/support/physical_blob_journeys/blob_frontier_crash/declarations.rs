pub(crate) use {std::num::NonZeroU64, std::time::Duration};

pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReadOpenFailure, worth_store::physical_runtime::PhysicalRecordId};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::BlobSessionFrontierV1};

pub(crate) use {crate::physical_blob_journeys::blob_crash::kill_at, crate::blob_crash::recovery::recover_closed_store, crate::physical_blob_journeys::blob_crash::SCOPE_KEY, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::serving_from_open};

pub(crate) const CHUNK_BYTES: usize = 64 << 10;

pub(crate) fn assert_record(
    selected: worth_store_physical_format::PersistedRecordIdentity,
    record: PhysicalRecordId,
) {
    assert_eq!(selected.allocation_epoch(), record.allocation_epoch());
    assert_eq!(selected.ordinal(), record.ordinal());
}