//! Original-request retained proposal custody through the actual public Native pipeline.
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::transactions::{
    CommitExecutionDenialKind, RelationalExecutionDenialCause, TransactionCommitError,
};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction,
    durable_publication::{batch_create, memory_runtime},
};

const SMALL_OWNER_CEILING: u64 = 4096;

#[test]
fn validated_proposal_backing_charges_original_request_until_native_consumption() {
    let runtime = memory_runtime();
    let identity = runtime.main_branch_identity();
    let before = runtime.observe_branch(&identity).unwrap().0;
    let budget = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(batch_create("charged"), request.allocation_policy())
        .unwrap();
    let proposal = transaction.validate(&request, &runtime).unwrap();
    assert!(
        budget.charged_memory_bytes() > SMALL_OWNER_CEILING,
        "the accepted owner retains its real state bill: {}",
        budget.charged_memory_bytes()
    );
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, before);
    cancellation.cancel();
    let denied = runtime
        .prepare_validated_proposal(&request, proposal)
        .unwrap_err();
    let TransactionCommitError::Execution { denial, .. } = denied else {
        panic!("the original cancelled request must stop Native consumption");
    };
    assert_eq!(
        denial.kind,
        CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::Cancelled)
    );
    assert_eq!(
        budget.charged_memory_bytes(),
        0,
        "consuming refusal releases original retained state"
    );
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, before);
}

#[test]
fn unfunded_validated_proposal_backing_refuses_and_releases_original_custody() {
    let runtime = memory_runtime();
    let identity = runtime.main_branch_identity();
    let before = runtime.observe_branch(&identity).unwrap().0;
    let budget = SerialMemoryBudget::new(Some(SMALL_OWNER_CEILING));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(batch_create("unfunded"), request.allocation_policy())
        .unwrap();
    let denied = transaction.validate(&request, &runtime).unwrap_err();
    let TransactionCommitError::Execution { denial, .. } = denied else {
        panic!("unfunded real validation must preserve its typed execution refusal");
    };
    assert!(matches!(denial.kind,
        CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::PolicyMemoryExhausted {
            requested, admitted, ancestor: 0,
        }) if requested > SMALL_OWNER_CEILING && admitted < requested));
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, before);
}
