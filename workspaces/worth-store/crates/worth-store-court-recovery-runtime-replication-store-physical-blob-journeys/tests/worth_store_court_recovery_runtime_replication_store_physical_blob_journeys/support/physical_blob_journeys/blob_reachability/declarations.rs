pub(crate) use {std::num::NonZeroU16, std::num::NonZeroU64};

pub(crate) use {worth_proof::AdmittedBlobReleaseProof};

pub(crate) use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReachabilityLimits, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::BlobRecordReachability, worth_store::physical_runtime::PhysicalIndexPointKey, worth_store::physical_runtime::PhysicalMutationDeadline};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_contracts::DurableArtifactFamilyId};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::DerivedFamilyRootDirectoryV1};

pub(crate) use {crate::physical_blob_journeys::blob_abort::deadline, crate::physical_blob_journeys::blob_abort::limits as terminal_limits, crate::physical_blob_journeys::blob_abort::unfinished, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};



pub(crate) fn inspection_limits() -> BlobReachabilityLimits {
    worth_store::physical_runtime::BlobReachabilityLimits::new(
        std::num::NonZeroU64::new(256).unwrap(),
        std::num::NonZeroU64::new(64 << 20).unwrap(),
        std::num::NonZeroU64::new(8).unwrap(),
        std::num::NonZeroU64::new(1024).unwrap(),
    )
}

pub(crate) fn class_of(
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

pub(crate) fn persisted(
    record: worth_store::physical_runtime::PhysicalRecordId,
) -> worth_store_physical_format::PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new(
        record.allocation_epoch(),
        record.ordinal(),
    )
    .unwrap()
}