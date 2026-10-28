use crate::branch::{
    RelationalBranchBasisDenial, RelationalBranchRoot, SelectedRelationalBranchState,
};
use crate::facade::history::BranchId;
use crate::tests::support::{create_entity, install_empty_test_branch, runtime_with_test_schema};

fn committed_child(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
) -> BranchId {
    create_entity(execution, runtime, "root-selection-source");
    let source = BranchId("main".to_owned());
    let (_, basis) = runtime
        .observe_fork_source(&source)
        .expect("committed source has a fork basis");
    let child = BranchId("root-selection-child".to_owned());
    runtime
        .fork_branch(execution, child.clone(), basis)
        .expect("fork installs a committed child root");
    child
}

#[test]
fn unavailable_committed_root_denies_before_transaction_admission() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let child = committed_child(execution, &runtime);
    runtime
        .history
        .branch_cell_mut(&child)
        .expect("child remains registered")
        .clear_root_for_test();
    let symbols_before = runtime.services.symbols.clone();
    let symbol_table_before = runtime.config().identity.symbol_table.clone();
    let commit_count = runtime.history().immutable_commit_count();
    let child_identity = runtime
        .branch_identity(&child)
        .expect("child identity remains owner-issued");
    let denial = runtime
        .admit_branch_basis(&child_identity)
        .expect_err("a committed branch without its exact root cannot mint authority");

    assert_eq!(
        denial,
        RelationalBranchBasisDenial::UnavailableRetainedTarget
    );
    assert_eq!(runtime.history().immutable_commit_count(), commit_count);
    assert_eq!(runtime.services.symbols, symbols_before);
    assert_eq!(runtime.config().identity.symbol_table, symbol_table_before);
}

#[test]
fn committed_root_reference_mismatch_denies_before_transaction_admission() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let child = committed_child(execution, &runtime);
    runtime
        .history
        .branch_cell_mut(&child)
        .expect("child remains registered")
        .install_root(RelationalBranchRoot::empty());
    let commit_count = runtime.history().immutable_commit_count();
    let child_identity = runtime
        .branch_identity(&child)
        .expect("child identity remains owner-issued");
    let denial = runtime
        .admit_branch_basis(&child_identity)
        .expect_err("a root that cannot satisfy the reference cannot mint authority");

    assert_eq!(
        denial,
        RelationalBranchBasisDenial::UnavailableRetainedTarget
    );
    assert_eq!(runtime.history().immutable_commit_count(), commit_count);
}

#[test]
fn selected_state_shape_keeps_empty_and_committed_roots_distinct() {
    let runtime = runtime_with_test_schema();
    let empty = BranchId("root-selection-empty".to_owned());
    install_empty_test_branch(&runtime, empty.clone());
    let empty_identity = runtime
        .branch_identity(&empty)
        .expect("empty identity remains owner-issued");
    let empty_basis = runtime
        .admit_branch_basis(&empty_identity)
        .expect("empty basis");
    let empty_state = runtime
        .selected_branch_state(&empty_basis)
        .expect("empty state is selectable");
    assert!(matches!(
        empty_state,
        SelectedRelationalBranchState::Empty(_)
    ));
    assert_eq!(empty_state.version_id().0, 0);
}
