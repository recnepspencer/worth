use super::*;

#[test]
fn phase2_ports_deny_after_their_runtime_owner_closes() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "phase2-owner-close-anchor");
    let mut transaction = begin_transaction(&runtime, "main");
    transaction
        .push_batch(
            batch_create("phase2-owner-close-write"),
            Allocation::SystemAllocation,
        )
        .expect("owner-close transaction stages");
    let preparation = runtime.preparation_port();
    let forking = runtime.fork_port();
    let (_, source) = forking
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("source observes before owner close");
    let runtime_instance_id = runtime.runtime_instance_id();
    drop(runtime);

    assert!(matches!(
        preparation.prepare_branch_transaction(execution, transaction),
        Err(crate::transactions::data::TransactionCommitError::PublicationDenied {
            denial: crate::mvcc::RelationalPublicationDenial::OwnerUnavailable {
                runtime_instance_id: observed,
            },
            ..
        }) if observed == runtime_instance_id
    ));
    assert!(matches!(
        forking.fork_branch(BranchId("owner-closed-target".to_owned()), source),
        Err(crate::branch::RelationalForkDenial::OwnerUnavailable)
    ));
}
