//! A branch root readmitted once may be shared by sibling cells, but each cell's
//! exact target must still bind it: a later cell cannot ride on the first
//! cell's readmission with a relabeled descriptor.
use super::branch_root_content_binding::{
    assert_corrupt_recovery_leaves_no_recovered_state, basis_target, replace_descriptor,
};
use super::*;
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointFieldsFixturePort as _;

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

    // Since commit-artifact validation runs per branch cell during restore_branch_cells,
    // a later sibling cell with a relabeled schema root fails closed against the artifact.
    assert_corrupt_recovery_leaves_no_recovered_state(
        execution,
        plan,
        "target does not match immutable commit artifact",
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
        "target does not match immutable commit artifact",
    );
}

/// A production checkpoint whose two branch cells select one commit and so
/// share one branch-root image.
fn shared_root_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &str,
) -> RecoveryPlan {
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, label);
    create_branch_from_main(&runtime, "shared-root-sibling");
    capture_and_file_checkpoint(execution, &runtime, &medium);
    medium
        .expanded_recovery_plan(
            execution,
            &runtime,
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
        checkpoint.branch_roots_for_test().len(),
        1,
        "sibling heads share one root"
    );
    let [first, later] = checkpoint.branch_cells_mut_for_test().as_mut_slice() else {
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
