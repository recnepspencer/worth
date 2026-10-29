pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {worth_store_physical_format::store_namespace::ProposedStoreIdentity, worth_store_physical_format::store_namespace::StableStoreIdentity, worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord, worth_store_physical_format::store_namespace::StoreNamespaceVersion};
pub(crate) use {worth_store_physical_format::BlobChunkFrameV1, worth_store_physical_format::BlobChunkOccurrenceV1, worth_store_physical_format::BlobGenerationPublicationV1, worth_store_physical_format::BlobRecordKind, worth_store_physical_format::BlobTreeEntryV1, worth_store_physical_format::BlobTreeNodeKind, worth_store_physical_format::BlobTreeNodeV1, worth_store_physical_format::BlobTreeOccurrenceV1, worth_store_physical_format::PersistedRecordIdentity};
pub(crate) use {worth_store_physical_integrity::inspect_physical_integrity_window, worth_store_physical_integrity::validate_blob_record, worth_store_physical_integrity::BlobRecordIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityScrubWindow, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) fn store(byte: u8) -> StableStoreIdentity {
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

pub(crate) fn record(ordinal: u64) -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
}

pub(crate) fn range(bytes: &[u8]) -> PhysicalByteRange {
    worth_store_physical_integrity::PhysicalByteRange::new(0, bytes.len() as u64).unwrap()
}

pub(crate) fn chunk_bytes() -> Vec<u8> {
    worth_store_physical_format::BlobChunkFrameV1::encode(
        worth_store_physical_format::BlobChunkOccurrenceV1::new([3; 16], [4; 16], 0).unwrap(),
        64 << 10,
        &[5; 64 << 10],
    )
    .unwrap()
}

pub(crate) fn chunk_scope(bytes: &[u8], identity: PersistedRecordIdentity) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::blob_chunk_frame(store(3), identity, range(bytes))
}