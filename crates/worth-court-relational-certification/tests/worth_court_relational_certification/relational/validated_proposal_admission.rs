//! Exact transaction admission and post-validation head checks.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    mvcc::{RelationalBranchTransactionAdmissionDenial, RelationalTransactionIntent},
    transactions::{CommitConflict, ConflictClass, TransactionCommitError},
};
use worth_testkit_relational::{
    durable_publication::memory_runtime, ordinary_records::create_entity,
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn stale_owner_binding_is_rejected_during_transaction_admission() {
    let host_request = serial_request();
    let execution = &ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let identity = runtime.main_branch_identity();
    let stale_options = runtime
        .admit_branch_basis(&identity)
        .expect("main branch owner binding");
    let _ = create_entity(execution, &runtime, "head-advance");

    assert_eq!(
        runtime
            .begin_branch_transaction(&stale_options, RelationalTransactionIntent::ordinary())
            .unwrap_err(),
        RelationalBranchTransactionAdmissionDenial::StaleBasis
    );
}

#[test]
fn owner_rechecks_head_after_validation_before_commit() {
    let host_request = serial_request();
    let execution = &ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let identity = runtime.main_branch_identity();
    let validation_input = runtime
        .admit_branch_basis(&identity)
        .expect("main branch owner binding");
    let candidate = runtime
        .begin_branch_transaction(&validation_input, RelationalTransactionIntent::ordinary())
        .expect("current basis belongs to the same runtime")
        .validate(execution, &runtime)
        .expect("head is current at validation");
    let candidate_ordinal = candidate.proposal_identity().ordinal();

    let _ = create_entity(execution, &runtime, "post-validation-advance");
    let denied = runtime
        .prepare_validated_proposal(execution, candidate)
        .expect_err("Relational must close the validate/commit race");

    assert!(matches!(
        denied,
        TransactionCommitError::Conflict {
            error: CommitConflict {
                class: ConflictClass::StaleValidationBasis { .. },
                ..
            },
            ..
        }
    ));

    let fresh_options = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .expect("fresh main binding");
    let fresh = runtime
        .begin_branch_transaction(&fresh_options, RelationalTransactionIntent::ordinary())
        .expect("fresh basis belongs to the same runtime")
        .validate(execution, &runtime)
        .expect("fresh validation remains admissible");
    assert_eq!(
        fresh.proposal_identity().ordinal(),
        candidate_ordinal + 2,
        "stale same-branch denial must not consume a proposal ordinal"
    );
}
