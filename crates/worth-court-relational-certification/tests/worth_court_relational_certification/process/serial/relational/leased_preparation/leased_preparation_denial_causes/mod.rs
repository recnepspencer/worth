#[allow(unused_imports)] use crate::leased_preparation::isolated_context::leased_preparation_denial_causes::*;
pub(crate) use {worth_execution::ExecutionAllocationPolicy as AllocationPolicy, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::history::BranchId, worth_execution::CancellationToken, worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause, worth_relational::facade::symbols::ClientKey, worth_relational::facade::transactions::CommitExecutionDenialKind, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::indexes::DerivedIndexBuildRequest, worth_relational::facade::indexes::DerivedIndexDefinition, worth_relational::facade::indexes::DerivedIndexExecutionDenialKind, worth_relational::facade::indexes::DerivedIndexId, worth_relational::facade::indexes::DerivedIndexKind, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::identity::KindId, worth_execution::LeaseRequest, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::identity::PartitionId, worth_relational::facade::runtime::RelationalRuntimeApi, worth_relational::facade::mvcc::RelationalTransactionIntent, worth_relational::facade::transactions::TransactionCommitError, worth_relational::facade::transactions::WorkerIntentBatch, crate::leased_preparation::ordinary_context::authority, crate::leased_preparation::ordinary_context::lease_request, crate::leased_preparation::ordinary_context::lease_request_with_work, crate::leased_preparation::ordinary_context::serial, crate::leased_preparation::ordinary_context::support};
pub(crate) use {std::time::Instant};

#[test]
fn cancelled_lease_preserves_both_request_causes() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _serial = serial();
    let source = worth_execution::CancellationSource::new();
    source.cancel();
    assert_request_cause(
        execution,
        lease_request(32 * 1024 * 1024, source.token()),
        |cause| cause == worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
    );
}

#[test]
fn expired_lease_preserves_both_request_causes() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _serial = serial();
    let mut request = lease_request(32 * 1024 * 1024, worth_execution::CancellationToken::new());
    request.deadline = Some(std::time::Instant::now());
    assert_request_cause(execution, request, |cause| cause == worth_relational::facade::transactions::RelationalExecutionDenialCause::DeadlineElapsed);
}

#[test]
fn exhausted_work_lease_preserves_both_request_causes() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _serial = serial();
    assert_request_cause(
        execution,
        lease_request_with_work(32 * 1024 * 1024, worth_execution::CancellationToken::new(), 0),
        |cause| cause == worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted,
    );
}

#[test]
fn exhausted_policy_memory_preserves_both_request_causes() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _serial = serial();
    assert_request_cause(
        execution,
        lease_request(0, worth_execution::CancellationToken::new()),
        |cause| {
            matches!(
                cause,
                worth_relational::facade::transactions::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                    requested: 1..,
                    admitted: 0,
                    ancestor: 0
                }
            )
        },
    );
}

#[test]
fn exhausted_process_memory_preserves_both_request_causes() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _serial = serial();
    let held = authority()
        .request_lease(lease_request(64 * 1024 * 1024, worth_execution::CancellationToken::new()))
        .expect("competing request is admitted");
    let _reservation = held
        .reserve_memory(64 * 1024 * 1024)
        .expect("process memory is held");
    assert_request_cause(
        execution,
        lease_request(32 * 1024 * 1024, worth_execution::CancellationToken::new()),
        |cause| {
            matches!(
                cause,
                worth_relational::facade::transactions::RelationalExecutionDenialCause::ProcessMemoryExhausted {
                    requested: 1..,
                    admitted: 0
                }
            )
        },
    );
}
