use crate::runtime::RelationalRuntime;
use crate::tests::support::{create_entity, runtime_with_test_schema};

#[test]
fn branch_names_include_live_and_retired_names_after_delete_and_restore() {
    use crate::facade::branch::RelationalBranchNames;
    use crate::history::data::BranchId;
    use crate::tests::support::create_branch_from_main;

    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "branch-name-anchor");
    let live = create_branch_from_main(&runtime, "product-12");
    let retired = create_branch_from_main(&runtime, "product-15");
    let identity = runtime.branch_identity(&retired).unwrap();
    assert!(runtime
        .delete_branch(&identity)
        .unwrap()
        .deleted()
        .is_some());
    let names: RelationalBranchNames = runtime.branch_names();
    assert_eq!(names.registered(), &[BranchId("main".into()), live]);
    assert_eq!(names.retired(), &[retired]);
    let hold = runtime.try_hold_admission().unwrap();
    let image = hold.native_checkpoint().unwrap();
    hold.seal();
    let mut restored: RelationalRuntime = runtime_with_test_schema();
    restored
        .durability_recovery()
        .restore_native_checkpoint(&image)
        .unwrap();
    assert_eq!(restored.branch_names(), names);
}

#[test]
fn branch_names_include_archived_and_deleting_names_during_a_hold() {
    use crate::branch::RelationalBranchLifecyclePosture;
    use crate::tests::support::create_branch_from_main;

    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "name-posture-anchor");
    let archived = create_branch_from_main(&runtime, "archived-name");
    let deleting = create_branch_from_main(&runtime, "deleting-name");
    let archived_identity = runtime.branch_identity(&archived).unwrap();
    runtime.archive_branch(&archived_identity).unwrap();
    assert_eq!(
        runtime
            .history
            .branch_cell(&archived)
            .unwrap()
            .lifecycle_posture(),
        RelationalBranchLifecyclePosture::Archived
    );
    let deleting_identity = runtime.branch_identity(&deleting).unwrap();
    let (_, basis) = runtime.observe_branch(&deleting_identity).unwrap();
    let transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    assert!(runtime
        .delete_branch(&deleting_identity)
        .unwrap()
        .deleted()
        .is_none());
    assert_eq!(
        runtime
            .history
            .branch_cell(&deleting)
            .unwrap()
            .lifecycle_posture(),
        RelationalBranchLifecyclePosture::Deleting
    );
    drop(transaction);
    drop(basis);
    let hold = runtime.try_hold_admission().unwrap();
    let names = hold.branch_names();
    assert!(names.registered().contains(&archived));
    assert!(!names.retired().contains(&archived));
    assert!(names.registered().contains(&deleting));
    assert!(names.retired().contains(&deleting));
    let image = hold.native_checkpoint().unwrap();
    hold.seal();
    let mut restored = runtime_with_test_schema();
    restored
        .durability_recovery()
        .restore_native_checkpoint(&image)
        .unwrap();
    assert_eq!(restored.branch_names(), names);
}
