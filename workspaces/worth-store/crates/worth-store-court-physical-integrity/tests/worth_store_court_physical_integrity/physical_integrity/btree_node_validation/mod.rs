#[allow(unused_imports)] use crate::btree_node_validation::*;
use {worth_store_physical_format::store_namespace::ProposedStoreIdentity, worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord, worth_store_physical_format::store_namespace::StoreNamespaceVersion};
use {worth_store_physical_format::BTreeNodeCellV1, worth_store_physical_format::BTreeNodeV1, worth_store_physical_format::PersistedRecordIdentity};
use {worth_store_physical_integrity::validate_btree_node, worth_store_physical_integrity::BTreeNodeIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::UntrustedPhysicalArtifact};

#[test]
fn intact_semantic_node_is_bound_to_exact_scope_and_input_incarnation() {
    let bytes = node_bytes();
    let input = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (result, counters) = validate_btree_node(input, scope(bytes.len(), 4));
    let worth_store_physical_integrity::BTreeNodeIntegrityValidation::Intact(view) = result else {
        panic!("valid encoded node must validate");
    };
    assert_eq!(view.node().cells().len(), 2);
    assert!(view.matches_input(input));
    let copied = bytes.clone();
    assert!(!view.matches_input(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&copied)));
    assert_eq!(counters.inspected_frames(), 1);
}

#[test]
fn family_mismatch_and_checksum_corruption_are_not_empty_lookups() {
    let bytes = node_bytes();
    let input = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (result, _) = validate_btree_node(input, scope(bytes.len(), 5));
    let worth_store_physical_integrity::BTreeNodeIntegrityValidation::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(damage)) =
        result
    else {
        panic!("family mismatch must be damage");
    };
    assert_eq!(damage.cause(), worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch);

    let mut corrupt = bytes;
    corrupt[104] ^= 1;
    let (result, _) = validate_btree_node(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&corrupt),
        scope(corrupt.len(), 4),
    );
    let worth_store_physical_integrity::BTreeNodeIntegrityValidation::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(damage)) =
        result
    else {
        panic!("checksum corruption must be damage");
    };
    assert_eq!(damage.cause(), worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch);
}

#[test]
fn unsupported_node_version_is_typed() {
    let mut bytes = node_bytes();
    bytes[8] = 2;
    let (result, _) = validate_btree_node(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        scope(bytes.len(), 4),
    );
    let worth_store_physical_integrity::BTreeNodeIntegrityValidation::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(version)) =
        result
    else {
        panic!("unsupported node version must be typed");
    };
    assert_eq!(version.axis(), worth_store_physical_integrity::PhysicalIntegrityVersionAxis::BTreeNode);
    assert_eq!(version.observed(), 2);
}
