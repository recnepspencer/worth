use super::*;
use crate::facade::durability::{RecoveryFailureClass, RecoveryVerificationMode};
use crate::facade::mvcc::RelationalTransactionIntent;

#[test]
fn a_deleting_checkpoint_cell_requires_its_retired_name() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
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
    medium.file_embedded_checkpoint(execution, &runtime);
    let mut recovered = durable_runtime_for_medium(&OwnedTestDurableMedium::new());
    let mut plan = medium
        .recovery_plan(
            execution,
            &recovered,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    plan.checkpoint
        .as_mut()
        .unwrap()
        .retired_branch_names
        .clear();
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
