use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{BTreeNodeCellV1, BTreeNodeV1, PersistedRecordIdentity};
use worth_store_physical_integrity::{
    validate_btree_node, BTreeNodeIntegrityValidation, PhysicalArtifactScope, PhysicalByteRange,
    PhysicalDamageCause, PhysicalIntegrityRejection, PhysicalIntegrityVersionAxis,
    UntrustedPhysicalArtifact,
};

fn record() -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([9; 16], 7).unwrap()
}

fn scope(length: usize, family_code: u16) -> PhysicalArtifactScope {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([3; 16]).unwrap(),
    )
    .published_identity();
    PhysicalArtifactScope::btree_node(
        store,
        record(),
        family_code,
        PhysicalByteRange::new(4096, length as u64).unwrap(),
    )
}

fn node_bytes() -> Vec<u8> {
    BTreeNodeV1::leaf(
        4,
        vec![
            BTreeNodeCellV1::leaf(b"a".to_vec(), b"first".to_vec()),
            BTreeNodeCellV1::leaf(b"z".to_vec(), b"last".to_vec()),
        ],
        None,
        None,
    )
    .unwrap()
    .encode(16_272)
    .unwrap()
}

#[test]
fn intact_semantic_node_is_bound_to_exact_scope_and_input_incarnation() {
    let bytes = node_bytes();
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (result, counters) = validate_btree_node(input, scope(bytes.len(), 4));
    let BTreeNodeIntegrityValidation::Intact(view) = result else {
        panic!("valid encoded node must validate");
    };
    assert_eq!(view.node().cells().len(), 2);
    assert!(view.matches_input(input));
    let copied = bytes.clone();
    assert!(!view.matches_input(UntrustedPhysicalArtifact::from_bounded_bytes(&copied)));
    assert_eq!(counters.inspected_frames(), 1);
}

#[test]
fn family_mismatch_and_checksum_corruption_are_not_empty_lookups() {
    let bytes = node_bytes();
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (result, _) = validate_btree_node(input, scope(bytes.len(), 5));
    let BTreeNodeIntegrityValidation::Rejected(PhysicalIntegrityRejection::Damaged(damage)) =
        result
    else {
        panic!("family mismatch must be damage");
    };
    assert_eq!(damage.cause(), PhysicalDamageCause::FamilyMismatch);

    let mut corrupt = bytes;
    corrupt[104] ^= 1;
    let (result, _) = validate_btree_node(
        UntrustedPhysicalArtifact::from_bounded_bytes(&corrupt),
        scope(corrupt.len(), 4),
    );
    let BTreeNodeIntegrityValidation::Rejected(PhysicalIntegrityRejection::Damaged(damage)) =
        result
    else {
        panic!("checksum corruption must be damage");
    };
    assert_eq!(damage.cause(), PhysicalDamageCause::ChecksumMismatch);
}

#[test]
fn unsupported_node_version_is_typed() {
    let mut bytes = node_bytes();
    bytes[8] = 2;
    let (result, _) = validate_btree_node(
        UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        scope(bytes.len(), 4),
    );
    let BTreeNodeIntegrityValidation::Rejected(PhysicalIntegrityRejection::Unsupported(version)) =
        result
    else {
        panic!("unsupported node version must be typed");
    };
    assert_eq!(version.axis(), PhysicalIntegrityVersionAxis::BTreeNode);
    assert_eq!(version.observed(), 2);
}
