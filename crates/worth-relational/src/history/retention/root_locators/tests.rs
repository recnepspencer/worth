//! Secondary-index membership is data, not root or historical-read authority.
use super::*;
use crate::tests::support::*;
use std::sync::Arc;

#[test]
fn removing_one_physical_readmission_preserves_the_other_commit_locator() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &runtime, "physical-locator-source");
    let root = runtime
        .history
        .branch_cell(&crate::history::data::BranchId("main".into()))
        .unwrap()
        .root()
        .unwrap();
    let readmitted = root
        .readmit_descriptor(
            root.descriptor().unwrap().clone(),
            &runtime.services.symbols.interner_snapshot(),
        )
        .unwrap();
    assert!(!Arc::ptr_eq(&root, &readmitted));
    let first = Arc::as_ptr(&root) as usize;
    let second = Arc::as_ptr(&readmitted) as usize;
    let locators = RetiredRootLocators::default();
    // Generations here are secondary-index epochs, not admitted retirement slots.
    locators.register(&root, first, 1);
    locators.register(&readmitted, second, 2);
    assert_eq!(locators.snapshot(committed.commit.commit_id).len(), 2);
    locators.remove(&readmitted, second, 2);
    assert_eq!(
        locators.snapshot(committed.commit.commit_id),
        vec![(first, 1)]
    );
    assert_eq!(
        locators.snapshot_basis(
            committed.commit.version_id,
            root.schema_authority().schema_version()
        ),
        vec![(first, 1)]
    );
}

#[test]
fn commit_aliases_share_exact_basis_membership_without_conflating_schema() {
    // Abstract lookup labels exercise the secondary index only. No root inventory,
    // owner-issued guard or operational authority is constructed from these labels.
    let locators = RetiredRootLocators::default();
    let basis = (VersionId(7), SchemaVersionId(1));
    let other_schema = (VersionId(7), SchemaVersionId(2));
    locators.register_locator(CommitId(11), basis, (101, 1));
    locators.register_locator(CommitId(12), basis, (202, 2));
    locators.register_locator(CommitId(13), other_schema, (303, 3));
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
}
