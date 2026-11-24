//! Secondary-index membership is data, not root or historical-read authority.
use super::*;

#[test]
fn commit_aliases_share_exact_basis_membership_without_conflating_schema() {
    // Abstract lookup labels exercise the secondary index only. No root inventory,
    // owner-issued guard or operational authority is constructed from these labels.
    let locators = RetiredRootLocators::default();
    let basis = (VersionId(7), SchemaVersionId(1));
    let other_schema = (VersionId(7), SchemaVersionId(2));
    locators.register_locator(CommitId(11), basis, (101, 1));
    locators.register_locator(CommitId(11), basis, (404, 4));
    locators.register_locator(CommitId(12), basis, (202, 2));
    locators.register_locator(CommitId(13), other_schema, (303, 3));
    assert_eq!(
        locators.snapshot_basis(basis.0, basis.1),
        vec![(101, 1), (202, 2), (404, 4)]
    );
    assert_eq!(locators.snapshot(CommitId(11)), vec![(101, 1), (404, 4)]);
    locators.remove_locator(CommitId(11), basis, (404, 4));
    assert_eq!(locators.snapshot(CommitId(11)), vec![(101, 1)]);
    assert_eq!(
        locators.snapshot_basis(basis.0, basis.1),
        vec![(101, 1), (202, 2)]
    );
    locators.remove_locator(CommitId(12), basis, (202, 2));
    assert!(locators.snapshot(CommitId(12)).is_empty());
    assert_eq!(locators.snapshot_basis(basis.0, basis.1), vec![(101, 1)]);
    assert_eq!(
        locators.snapshot_basis(other_schema.0, other_schema.1),
        vec![(303, 3)]
    );
    locators.remove_locator(CommitId(11), basis, (101, 1));
    assert!(locators.snapshot(CommitId(11)).is_empty());
    assert!(locators.snapshot_basis(basis.0, basis.1).is_empty());
}
