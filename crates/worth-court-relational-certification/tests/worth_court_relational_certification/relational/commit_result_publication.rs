//! Original commit result cleanup is observable at the owner service boundary.
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};
use worth_relational::facade::{
    branch::{
        RelationalBranchBasisDenial, RelationalCommitResultSnapshotRelease as Outcome,
        RelationalCommitResultSnapshotReleaseDenial as Denial,
    },
    change_source::RelationalReadCustodyHolders,
    mvcc::{BranchBoundRelationalTransaction, RelationalTransactionIntent},
    runtime::{
        RelationalRetainedCommitSnapshotDenialKind, RelationalRuntime, RelationalRuntimeCloseDenial,
    },
    transactions::{
        CommitExecutionDenialKind, RelationalExecutionDenialCause, TransactionCommitError,
        WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    durable_publication::{create_entity_outcome, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn native_custody_correction_result_cleanup_is_explicit_idempotent_and_keeps_external_read() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let result = std::sync::Arc::new(create_entity_outcome(execution, &runtime, "owned-result"));
    let cloned = result.clone();
    let service = runtime.owner_component_services().basis_port();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let independent = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    drop(basis);
    assert!(matches!(
        service.release_commit_result_snapshot(&result).unwrap(),
        Outcome::Released(_)
    ));
    assert!(
        matches!(service.release_commit_result_snapshot(&cloned).unwrap(), Outcome::AlreadyReleased { snapshot_id } if snapshot_id == result.snapshot.snapshot_id())
    );
    assert_eq!(
        runtime
            .snapshots()
            .retained_snapshot_for_commit(runtime.runtime_instance_id(), &result.commit)
            .unwrap_err()
            .kind(),
        RelationalRetainedCommitSnapshotDenialKind::SnapshotNotRetained,
    );
    assert_eq!(runtime.retention().active_snapshot_count(), 1);
    let refused = runtime.prepare_close().unwrap_err();
    assert!(
        matches!(refused.reason(), RelationalRuntimeCloseDenial::ReadCustodyRetained(
            RelationalReadCustodyHolders::SnapshotsAndHistory {
                active_snapshots, detached_history,
            }
        ) if active_snapshots != 0 || detached_history)
    );
    let runtime = refused.into_runtime();
    runtime.snapshots().release_snapshot(&independent).unwrap();
    runtime.prepare_close().unwrap().finish();
    assert_eq!(
        cloned.commit, result.commit,
        "actual Arc results remain descriptive after explicit cleanup and close"
    );
    assert!(matches!(
        service.release_commit_result_snapshot(&result),
        Err(Denial::Owner(RelationalBranchBasisDenial::OwnerUnavailable))
    ));
}

fn empty_transaction(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
) -> BranchBoundRelationalTransaction {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("stage-0"),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction
}

#[test]
fn a_completed_empty_commit_funds_its_retained_publication_metadata() {
    let runtime = memory_runtime();
    let backing = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&backing);
    let (result, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            runtime.commit_branch_transaction(&execution, empty_transaction(&runtime, &execution))
        })
        .unwrap();
    let result = result.unwrap();
    assert!(result.changed_records.is_empty());
    assert!(report.charged_work() > 0);
    assert!(backing.memory().unwrap().charged_memory_bytes() > 0);
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    let (refused, _) = execution
        .run(ExecutionWorkCeiling::new(Some(0)), |_| {
            runtime.commit_branch_transaction(&execution, empty_transaction(&runtime, &execution))
        })
        .unwrap();
    assert!(
        matches!(refused, Err(TransactionCommitError::Execution { denial, .. })
        if denial.kind == CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::WorkExhausted))
    );
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        before
    );
    drop((runtime, result));
    assert_eq!(backing.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn cancelled_memory_publication_keeps_position_and_head_then_allows_ordinary_publication() {
    use worth_relational::facade::mvcc::RelationalPublicationOutcome;
    use worth_testkit_relational::{
        aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot},
        durable_publication::batch_create,
    };
    let setup = serial_request();
    let setup_request = ExecutionRequest::serial(&setup);
    let runtime = memory_runtime();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            batch_create("cancel-memory-frame"),
            setup_request.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction.validate(&setup_request, &runtime).unwrap();
    let candidate = runtime
        .prepare_validated_proposal(&setup_request, proposal)
        .unwrap();
    let counters = runtime.patch_position_reservation_counters();
    let positions = || {
        runtime
            .durability()
            .canonical_commit_artifacts(&setup_request)
            .map(|frame| frame.unwrap().position())
            .collect::<Vec<_>>()
    };
    let before = positions();
    let cancel = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
    let original = ExecutionRequest::serial(&host);
    cancel.cancel();
    assert!(matches!(
        runtime
            .publication_port()
            .compare_and_publish(&original, candidate),
        RelationalPublicationOutcome::RequestStopped(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled
        ))
    ));
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        counters.assignments
    );
    assert_eq!(positions(), before);
    runtime
        .owner_component_services()
        .basis_port()
        .compare_current_exact(&basis)
        .unwrap();
    drop(basis);
    let result = create_entity_outcome(&setup_request, &runtime, "caller-after-cancellation");
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        counters.assignments + 1
    );
    let after = positions();
    assert_eq!(after.len(), before.len() + 1);
    assert!(after.starts_with(&before));
    release_test_commit_snapshot(&runtime, &result);
}
