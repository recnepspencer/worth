pub(crate) use {core::num::NonZeroU64};

pub(crate) use {worth_store_physical_format::physical_work_obligation::decode_physical_work_obligation_v6, worth_store_physical_format::physical_work_obligation::encode_physical_work_obligation_v6, worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode, worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationTargetCode, worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationV6};
pub(crate) use {worth_store_physical_format::store_namespace::ProposedStoreIdentity, worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord, worth_store_physical_format::store_namespace::StoreNamespaceVersion};
pub(crate) use {worth_store_physical_format::wal_frame::decode_wal_frame_v1_header, worth_store_physical_format::wal_frame::encode_wal_frame_v1, worth_store_physical_format::wal_frame::WalFrameV1EncodeRequest, worth_store_physical_format::wal_frame::WAL_FRAME_V1_HEADER_BYTES};
pub(crate) use {worth_store_physical_format::PhysicalWorkObligationIdentity, worth_store_physical_format::WalSegmentIdentity};

pub(crate) fn store() -> worth_store_physical_format::store_namespace::StableStoreIdentity {
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
    )
    .published_identity()
}