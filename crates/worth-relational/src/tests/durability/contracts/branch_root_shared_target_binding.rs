//! A branch root readmitted once may be shared by sibling cells, but each cell's
//! exact target must still bind it: a later cell cannot ride on the first
//! cell's readmission with a relabeled descriptor.
use super::branch_root_content_binding::{
    assert_corrupt_recovery_leaves_no_recovered_state, basis_target, replace_descriptor,
};
use super::*;

#[test]
fn checkpoint_rejects_later_shared_root_cell_with_relabeled_schema_root() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut plan = shared_root_plan(execution, "shared-root-schema-relabeling");
    let later = later_shared_root_cell(&mut plan);
    let target = basis_target(later);
    let mut substituted_schema_root = *target.roots().schema_root();
    substituted_schema_root[0] ^= 0x40;
    replace_descriptor(
        later,
        crate::branch::RelationalBranchRootDescriptor::new(
            *target.roots().truth_root(),
            substituted_schema_root,
        ),
    );

    assert_corrupt_recovery_leaves_no_recovered_state(
        execution,
        plan,
        "durable branch-root image does not match target",
    );
}

#[test]
fn checkpoint_rejects_later_shared_root_cell_with_relabeled_truth_root() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut plan = shared_root_plan(execution, "shared-root-truth-relabeling");
    let later = later_shared_root_cell(&mut plan);
    let target = basis_target(later);
    let mut substituted_truth_root = *target.roots().truth_root();
    substituted_truth_root[0] ^= 0x80;
    replace_descriptor(
        later,
        crate::branch::RelationalBranchRootDescriptor::new(
            substituted_truth_root,
            *target.roots().schema_root(),
        ),
    );

    assert_corrupt_recovery_leaves_no_recovered_state(
        execution,
        plan,
        "durable branch-root image does not match target",
    );
}

/// A production checkpoint whose two branch cells select one commit and so
/// share one branch-root image.
fn shared_root_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &str,
) -> RecoveryPlan {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, label);
    create_branch_from_main(&runtime, "shared-root-sibling");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .expect("production checkpoint succeeds");
    runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap()
}

/// The cell readmission reaches second, after the first cell has readmitted the
/// shared root from its authentic target.
fn later_shared_root_cell(
    plan: &mut RecoveryPlan,
) -> &mut crate::branch::RelationalBranchCellCheckpoint {
    let checkpoint = plan
        .checkpoint
        .as_mut()
        .expect("production recovery plan selects a checkpoint");
    assert_eq!(
        checkpoint.branch_roots.len(),
        1,
        "sibling heads share one root"
    );
    let [first, later] = checkpoint.branch_cells.as_mut_slice() else {
        panic!("the checkpoint carries exactly the main and sibling cells");
    };
    let (first_target, later_target) = (basis_target(first), basis_target(later));
    assert_eq!(
        first_target.selected_commit_id(),
        later_target.selected_commit_id()
    );
    assert_eq!(first_target.roots(), later_target.roots());
    later
}
