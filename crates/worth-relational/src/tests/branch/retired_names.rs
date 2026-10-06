use crate::branch::{
    RelationalBranchDeleteDenial, RelationalBranchLifecyclePosture, RelationalForkDenial,
};
use crate::history::data::BranchId;
use crate::runtime::RelationalRuntime;
use crate::tests::support::{
    create_branch_from_main, create_entity, persisted_runtime_with_test_schema,
    runtime_with_test_schema,
};

#[test]
fn restored_retired_name_refuses_fork_as_retired_target() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "retired-name-anchor");
    let retired = create_branch_from_main(&runtime, "retired-across-restore");
    let identity = runtime.branch_identity(&retired).unwrap();
    assert!(runtime
        .delete_branch(&identity)
        .unwrap()
        .deleted()
        .is_some());

    let recovered = restored_from_native_checkpoint(&runtime);

    assert!(recovered.history.branch_cell(&retired).is_none());
    assert!(matches!(
        recovered.fork_branch(retired, main_fork_source(&recovered)),
        Err(RelationalForkDenial::RetiredTarget)
    ));
    recovered
        .fork_branch(
            BranchId("fresh-after-restore".to_owned()),
            main_fork_source(&recovered),
        )
        .expect("a name that was never retired still forks after restore");
}

#[test]
fn retired_name_bound_holds_after_restore() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "retired-bound-anchor");
    runtime.fill_retired_branch_name_capacity_for_test();

    let mut recovered = restored_from_native_checkpoint(&runtime);

    let beyond = create_branch_from_main(&recovered, "beyond-retired-bound");
    let identity = recovered.branch_identity(&beyond).unwrap();
    assert!(matches!(
        recovered.delete_branch(&identity),
        Err(RelationalBranchDeleteDenial::RetiredIdentityCapacityExhausted)
    ));
    assert!(recovered.history.branch_cell(&beyond).is_some());
}

#[test]
fn runtime_fork_keeps_retired_names_of_the_history_it_inherits() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "retired-runtime-fork-anchor");
    let retired = create_branch_from_main(&runtime, "retired-before-runtime-fork");
    let identity = runtime.branch_identity(&retired).unwrap();
    assert!(runtime
        .delete_branch(&identity)
        .unwrap()
        .deleted()
        .is_some());

    let forked = runtime.fork().expect("a settled runtime forks");

    assert!(matches!(
        forked.fork_branch(retired, main_fork_source(&forked)),
        Err(RelationalForkDenial::RetiredTarget)
    ));
}

#[test]
fn delete_waiting_at_capture_restores_as_a_deleting_branch_that_then_deletes() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "waiting-delete-anchor");
    let waiting = create_branch_from_main(&runtime, "waiting-delete-across-restore");
    let identity = runtime.branch_identity(&waiting).unwrap();
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("the live branch admits a basis");
    let transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .expect("the transaction owns an active branch operation");
    assert!(runtime
        .delete_branch(&identity)
        .unwrap()
        .deleted()
        .is_none());
    drop(transaction);
    drop(basis);

    let mut recovered = restored_from_native_checkpoint(&runtime);

    let restored = recovered
        .history
        .branch_cell(&waiting)
        .expect("the image carried the cell of the waiting delete");
    assert_eq!(
        restored.lifecycle_posture(),
        RelationalBranchLifecyclePosture::Deleting,
        "a retired name never names a live branch"
    );
    assert!(matches!(
        recovered.fork_branch(waiting.clone(), main_fork_source(&recovered)),
        Err(RelationalForkDenial::RetiredTarget)
    ));
    assert!(recovered
        .delete_branch(restored.identity())
        .unwrap()
        .deleted()
        .is_some());
    assert!(recovered.history.branch_cell(&waiting).is_none());
}

/// Owned by Database Foundation D.9.
///
/// This pins a gap, not a guarantee. In today's persisted mode the segment log
/// does not record a deletion, so a store reopened from its last checkpoint
/// brings the branch back with its name free. D.9 makes retirement a durable
/// fact and deletes this mode; this test is deleted with it.
#[test]
fn d9_persisted_reopen_brings_back_a_branch_deleted_after_the_last_checkpoint() {
    let mut runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "persisted-delete-anchor");
    let deleted = create_branch_from_main(&runtime, "deleted-after-the-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    let identity = runtime.branch_identity(&deleted).unwrap();
    assert!(runtime
        .delete_branch(&identity)
        .unwrap()
        .deleted()
        .is_some());
    assert!(runtime.history.branch_cell(&deleted).is_none());

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut reopened = persisted_runtime_with_test_schema();
    reopened.durability_recovery().recover(plan).unwrap();

    let returned = reopened
        .history
        .branch_cell(&deleted)
        .expect("today the reopened store still carries the deleted branch");
    assert_eq!(
        returned.lifecycle_posture(),
        RelationalBranchLifecyclePosture::Live
    );
    assert!(
        reopened
            .history
            .retired_branch_names_checkpoint()
            .is_empty(),
        "today the reopened store does not know the name was retired"
    );
}

fn restored_from_native_checkpoint(source: &RelationalRuntime) -> RelationalRuntime {
    let checkpoint = source
        .durability_authority()
        .native_checkpoint()
        .expect("the source world encodes as one native checkpoint");
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .restore_native_checkpoint(&checkpoint)
        .expect("the native checkpoint restores");
    recovered
}

fn main_fork_source(
    runtime: &RelationalRuntime,
) -> crate::branch::AdmittedRelationalForkSourceBasis {
    runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main has an exact fork source")
        .1
}
