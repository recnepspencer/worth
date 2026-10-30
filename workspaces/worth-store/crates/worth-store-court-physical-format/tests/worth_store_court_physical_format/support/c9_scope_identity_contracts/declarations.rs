pub(crate) use core::num::NonZeroU64;

pub(crate) use worth_store_physical_format::physical_work_obligation::{
    decode_physical_work_obligation_v6, encode_physical_work_obligation_v6,
    PhysicalWorkObligationOperationCode, PhysicalWorkObligationTargetCode,
    PhysicalWorkObligationV6,
};
pub(crate) use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
pub(crate) use worth_store_physical_format::wal_frame::{
    decode_wal_frame_v1_header, encode_wal_frame_v1, WalFrameV1EncodeRequest,
    WAL_FRAME_V1_HEADER_BYTES,
};
pub(crate) use worth_store_physical_format::{PhysicalWorkObligationIdentity, WalSegmentIdentity};

pub(crate) fn store() -> worth_store_physical_format::store_namespace::StableStoreIdentity {
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes(
            [7; 16],
        )
        .unwrap(),
    )
    .published_identity()
}
