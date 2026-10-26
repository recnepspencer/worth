use super::*;
use crate::facade::durability::{RecoveryFailureClass, RecoveryVerificationMode};
use crate::facade::mvcc::RelationalTransactionIntent;

#[test]
fn checkpoint_size_tracks_branch_state_not_covered_transitions() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let history = |extra_steps: bool| {
        let mut runtime = durable_runtime_with_test_schema();
        create_entity(execution, &runtime, "source");
        for index in 0..32 {
            let name = format!("branch-{index:03}");
            fork(execution, &runtime, &name);
            let identity = runtime.branch_identity(&BranchId(name)).unwrap();
            if extra_steps {
                let (_, basis) = runtime.observe_branch(&identity).unwrap();
                let transaction = runtime
                    .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                    .unwrap();
                runtime.archive_branch(execution, &identity).unwrap();
                runtime
                    .owner_component_services()
                    .lifecycle_port()
                    .delete_branch(execution, &identity)
                    .unwrap();
                drop(transaction);
                drop(basis);
            }
            runtime.delete_branch(execution, &identity).unwrap();
        }
        let count = if extra_steps { 128 } else { 64 };
        assert_eq!(artifacts(&runtime).len(), count);
        let checkpoint = runtime
            .durability_authority()
            .native_checkpoint(execution.allocation_policy())
            .unwrap();
        assert_eq!(
            checkpoint.covered_branch_transitions(),
            Some(Sequence(count as u64))
        );
        assert!(
            artifacts(&runtime).is_empty(),
            "covered transition log is released"
        );
        let bytes = checkpoint.bytes().len();
        attached_test_medium(&runtime).file_checkpoint(checkpoint);
        let recovered = reopen(execution, &runtime);
        assert!(artifacts(&recovered).is_empty());
        fork(execution, &recovered, "after-checkpoint");
        assert_eq!(
            attached_test_medium(&recovered).transition_head(),
            Sequence(count as u64 + 1)
        );
        bytes
    };
    let direct = history(false);
    let through_archive_and_pending_deletion = history(true);
    // Identical current cells and retired names, but twice as many transitions.
    // Small headroom permits integer encoding/runtime identity width changes.
    assert!(
        through_archive_and_pending_deletion <= direct + 32,
        "covered history grew the manifest: {direct} -> {through_archive_and_pending_deletion}"
    );
}

#[test]
fn a_deleting_checkpoint_cell_requires_its_retired_name() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "pending");
    let identity = runtime
        .branch_identity(&BranchId("pending".into()))
        .unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    runtime
        .owner_component_services()
        .lifecycle_port()
        .delete_branch(execution, &identity)
        .unwrap();
    drop(transaction);
    drop(basis);
    checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    plan.checkpoint
        .as_mut()
        .unwrap()
        .retired_branch_names
        .clear();
    let mut recovered = durable_runtime_with_test_schema();
    let error = expect_durability_failure(
        recovered
            .durability_recovery()
            .recover(execution, plan)
            .unwrap_err(),
    );
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        error
            .detail
            .contains("deleting branch `pending` has no retired name"),
        "{error:?}"
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}
