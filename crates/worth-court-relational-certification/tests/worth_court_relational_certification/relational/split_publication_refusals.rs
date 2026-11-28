//! Refused candidates return their exact registration and publication slot.
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    inspection::RelationalMvccCostScope,
    mvcc::{
        PreparedRelationalCommitCandidate, RelationalPublicationDeferred,
        RelationalPublicationDenial, RelationalPublicationOutcome, RelationalTransactionIntent,
    },
    runtime::{
        RelationalPublicationRegistryCustody, RelationalRuntime, RelationalRuntimeCloseDenial,
    },
    transactions::{TransactionCommitError, WorkerIntentBatch},
};
use worth_testkit_relational::{
    durable_publication::memory_runtime,
    shared_relational_certification_test_execution_policy::serial_request,
};

fn one_slot_runtime() -> RelationalRuntime {
    let mut config = memory_runtime().config();
    config.publication.policy.max_published_snapshot_handles = Some(1);
    RelationalRuntime::new(config).unwrap()
}

fn prepare(
    runtime: &RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
) -> Result<PreparedRelationalCommitCandidate, TransactionCommitError> {
    let mut transaction = runtime
        .begin_branch_transaction(basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(WorkerIntentBatch::new("split"), request.allocation_policy())
        .unwrap();
    let proposal = runtime.validate_branch_transaction(request, transaction)?;
    runtime.prepare_validated_proposal(request, proposal)
}

fn observe_occupied_slot(
    runtime: RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
) -> RelationalRuntime {
    let refused = runtime.prepare_close().unwrap_err();
    assert!(matches!(
        refused.reason(),
        RelationalRuntimeCloseDenial::PublicationCustodyRetained(holders)
            if matches!(holders.candidates, RelationalPublicationRegistryCustody::Registered { count: 1, .. })
    ));
    let runtime = refused.into_runtime();
    assert!(matches!(
        prepare(&runtime, request, basis),
        Err(TransactionCommitError::PublicationDeferred {
            deferred: RelationalPublicationDeferred::PublishedSnapshotCapacityExhausted {
                maximum_handles: 1,
            },
            ..
        })
    ));
    runtime
}

fn prove_returned_slot(runtime: RelationalRuntime, basis: AdmittedRelationalBranchBasis) {
    let host = serial_request();
    let request = ExecutionRequest::serial(&host);
    let reacquired = prepare(&runtime, &request, &basis).unwrap();
    drop(reacquired);
    drop(basis);
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn split_publication_cancelled_request_releases_candidate_and_records_discard() {
    let runtime = one_slot_runtime();
    let identity = runtime.main_branch_identity();
    let (before, basis) = runtime.observe_branch(&identity).unwrap();
    let cancellation = CancellationSource::new();
    let backing =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&backing);
    let candidate = prepare(&runtime, &execution, &basis).unwrap();
    let runtime = observe_occupied_slot(runtime, &execution, &basis);
    let scope = RelationalMvccCostScope::capture(&runtime, vec![identity.clone()]);
    cancellation.cancel();
    let outcome = runtime
        .publication_port()
        .compare_and_publish(&execution, candidate);
    assert!(matches!(
        outcome,
        RelationalPublicationOutcome::RequestStopped(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled
        ))
    ));
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, before);
    let observation = runtime
        .inspect_what_happened()
        .observe_mvcc_counters(&scope)
        .unwrap();
    assert_eq!(observation.sharing_cost_delta().candidate_discards, 1);
    assert_eq!(observation.sharing_cost_delta().publication_attempts, 0);
    prove_returned_slot(runtime, basis);
}

#[test]
fn foreign_candidate_leaves_the_target_owner_and_its_counters_unchanged() {
    let source = one_slot_runtime();
    let target = one_slot_runtime();
    let backing = serial_request();
    let execution = ExecutionRequest::serial(&backing);
    let source_identity = source.main_branch_identity();
    let (source_head, basis) = source.observe_branch(&source_identity).unwrap();
    let target_identity = target.main_branch_identity();
    let (target_head, target_basis) = target.observe_branch(&target_identity).unwrap();
    let candidate = prepare(&source, &execution, &basis).unwrap();
    let source = observe_occupied_slot(source, &execution, &basis);
    let scope = RelationalMvccCostScope::capture(&target, vec![target_identity.clone()]);
    let outcome = target
        .publication_port()
        .compare_and_publish(&execution, candidate);
    assert!(
        matches!(
            outcome,
            RelationalPublicationOutcome::Denied(RelationalPublicationDenial::ForeignRuntime {
                expected_runtime_instance_id, actual_runtime_instance_id,
            }) if expected_runtime_instance_id == target.runtime_instance_id()
                && actual_runtime_instance_id == source.runtime_instance_id()
        ),
        "{outcome:?}"
    );
    let observation = target.observe_mvcc_counters(&scope).unwrap();
    assert_eq!(observation.sharing_cost_delta().candidate_discards, 0);
    assert_eq!(observation.sharing_cost_delta().publication_attempts, 0);
    assert_eq!(
        target.observe_branch(&target_identity).unwrap().0,
        target_head
    );
    assert_eq!(
        source.observe_branch(&source_identity).unwrap().0,
        source_head
    );
    prove_returned_slot(source, basis);
    prove_returned_slot(target, target_basis);
}
