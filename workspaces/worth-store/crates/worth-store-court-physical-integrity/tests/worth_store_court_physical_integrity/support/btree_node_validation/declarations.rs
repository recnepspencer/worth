pub(crate) use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
pub(crate) use worth_store_physical_format::{
    BTreeNodeCellV1, BTreeNodeV1, PersistedRecordIdentity,
};
pub(crate) use worth_store_physical_integrity::{
    validate_btree_node, BTreeNodeIntegrityValidation, PhysicalArtifactScope, PhysicalByteRange,
    PhysicalDamageCause, PhysicalIntegrityRejection, PhysicalIntegrityVersionAxis,
    UntrustedPhysicalArtifact,
};

pub(crate) fn record() -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new([9; 16], 7).unwrap()
}

pub(crate) fn scope(length: usize, family_code: u16) -> PhysicalArtifactScope {
    let store = worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes(
            [3; 16],
        )
        .unwrap(),
    )
    .published_identity();
    worth_store_physical_integrity::PhysicalArtifactScope::btree_node(
        store,
        record(),
        family_code,
        worth_store_physical_integrity::PhysicalByteRange::new(4096, length as u64).unwrap(),
    )
}

pub(crate) fn node_bytes() -> Vec<u8> {
    worth_store_physical_format::BTreeNodeV1::leaf(
        4,
        vec![
            worth_store_physical_format::BTreeNodeCellV1::leaf(b"a".to_vec(), b"first".to_vec()),
            worth_store_physical_format::BTreeNodeCellV1::leaf(b"z".to_vec(), b"last".to_vec()),
        ],
        None,
        None,
    )
    .unwrap()
    .encode(16_272)
    .unwrap()
}
