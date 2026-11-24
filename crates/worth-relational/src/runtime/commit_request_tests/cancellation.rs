use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_relational_storage::internal_port::durability::medium::read_protection::RelationalDurableReadProtectionOwnerPort as _;
use crate::authority::commit::pipeline::request_checkpoints::{observation, CommitStage};
use crate::tests::support::{
    batch_create, create_entity_outcome, release_test_commit_snapshot, runtime_with_test_schema,
    test_owner_begin_transaction_for_main,
};
use crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::WorkerIntentBatch;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

fn transaction(
    runtime: &crate::runtime::RelationalRuntime,
    batches: usize,
) -> crate::mvcc::BranchBoundRelationalTransaction {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    for index in 0..batches {
        transaction
            .push_batch(
                WorkerIntentBatch::new(format!("stage-{index}")),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
    }
    transaction
}

#[test]
fn cancellation_between_validation_and_publication_stops_without_publishing() {
    let runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let before = runtime.observe_branch(&identity).unwrap().0;
    let cancellation = Arc::new(CancellationSource::new());
    let backing =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&backing);
    let visits = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&visits);
    let cancel_at_publication = Arc::clone(&cancellation);
    let publication = runtime.publication_binding();
    let preparation = runtime.preparation_runtime_snapshot();
    let before_candidates = publication.prepared_candidate_count();
    let before_slots = preparation.published_snapshot_count();
    let candidates_at_stage = Arc::new(AtomicUsize::new(0));
    let candidates_seen = Arc::clone(&candidates_at_stage);
    let slots_at_stage = Arc::new(AtomicUsize::new(0));
    let slots_seen = Arc::clone(&slots_at_stage);
    let result = observation::with_observer(
        Box::new(move |stage| {
            if stage == CommitStage::Publication {
                seen.fetch_add(1, Ordering::Relaxed);
                candidates_seen.store(publication.prepared_candidate_count(), Ordering::Relaxed);
                slots_seen.store(preparation.published_snapshot_count(), Ordering::Relaxed);
                cancel_at_publication.cancel();
            }
        }),
        || runtime.commit_branch_transaction(&execution, transaction(&runtime, 1)),
    );
    // If the consult disappears this assertion fails; no thread waits for an event.
    assert_eq!(
        visits.load(Ordering::Relaxed),
        1,
        "publication consult must reach its observer"
    );
    assert_eq!(
        candidates_at_stage.load(Ordering::Relaxed),
        before_candidates + 1
    );
    assert_eq!(slots_at_stage.load(Ordering::Relaxed), before_slots + 1);
    assert_eq!(
        runtime.publication_binding().prepared_candidate_count(),
        before_candidates
    );
    assert_eq!(
        runtime
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
        before_slots
    );
    let TransactionCommitError::Execution { denial, .. } =
        result.expect_err("cancelled request cannot publish")
    else {
        panic!("interruption retains its typed request cause")
    };
    assert_eq!(
        denial.kind,
        CommitExecutionDenialKind::Cause(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        )
    );
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, before);
}

// Observe the actual Runtime prepared root and prior canonical position.
#[test]
fn memory_frame_quota_and_cancel_refuse_before_position_or_head() {
    let setup = worth_harness::facade::execution_fixture::serial_request();
    let setup_request = ExecutionRequest::serial(&setup);
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("quota-memory-frame"),
            setup_request.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction.validate(&setup_request, &runtime).unwrap();
    let candidate = runtime
        .prepare_validated_proposal(&setup_request, proposal)
        .unwrap();
    let (_, _, next) = candidate.change_summary_basis_and_roots().unwrap();
    let envelope = next.canonical_envelope().unwrap();
    #[derive(serde::Serialize)]
    struct Reference<'a> {
        position: crate::publication::patch::data::PatchStreamPosition,
        canonical: crate::history::data::AuthoritativeArtifactEnvelopeRef<'a>,
        original_inputs:
            Option<&'a crate::history::data::canonical_commit_envelope::recorded_inputs::RecordedCanonicalInputs>,
    }
    let position = runtime
        .history
        .latest_recorded_patch_position()
        .map_or(1, |position| position.0 + 1);
    let reference = rmp_serde::to_vec_named(&Reference {
        position: crate::publication::patch::data::PatchStreamPosition(position),
        canonical: crate::history::data::AuthoritativeArtifactEnvelopeRef::new(envelope),
        original_inputs: envelope.recorded_original_inputs(),
    })
    .unwrap();
    let bill = (8 + 2 + 8 + 8 + 32 + 8 + reference.len()) as u64;
    let controller = controller_memory();
    assert!(bill > controller);
    let counters = runtime.patch_position_reservation_counters();
    let before_head = runtime.history.latest_recorded_patch_position();
    let limited = SerialMemoryBudget::new(Some(controller.checked_add(bill).unwrap() - 1));
    let host = SerialRequest::from_memory(limited.clone(), CancellationSource::new().token(), None);
    let original = ExecutionRequest::serial(&host);
    let outcome = runtime
        .publication_port()
        .compare_and_publish(&original, candidate);
    let crate::mvcc::RelationalPublicationOutcome::RequestStopped(
        worth_execution::WorkCeilingDenial::Admission(
            worth_execution::LeaseDenial::MemoryExhausted(
                worth_execution::MemoryLimitDenial::Capacity { requested, .. },
            ),
        ),
    ) = outcome
    else {
        panic!("the actual frame reservation must refuse")
    };
    assert_eq!(
        requested, bill,
        "independent reference frame bytes, not the controller or preparation"
    );
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        counters.assignments
    );
    assert_eq!(
        runtime.history.latest_recorded_patch_position(),
        before_head
    );
    runtime.compare_current_exact(&basis).unwrap();
    assert_eq!(limited.charged_memory_bytes(), 0);
    drop(next);
    drop(basis);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
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
    let cancel = CancellationSource::new();
    let cancelled = SerialMemoryBudget::new(Some(8 << 20));
    let host = SerialRequest::from_memory(cancelled.clone(), cancel.token(), None);
    let original = ExecutionRequest::serial(&host);
    cancel.cancel();
    assert!(matches!(
        runtime
            .publication_port()
            .compare_and_publish(&original, candidate),
        crate::mvcc::RelationalPublicationOutcome::RequestStopped(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        )
    ));
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        counters.assignments
    );
    assert_eq!(
        runtime.history.latest_recorded_patch_position(),
        before_head
    );
    assert_eq!(cancelled.charged_memory_bytes(), 0);
    let result = create_entity_outcome(&setup_request, &runtime, "caller-after-refusals");
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        counters.assignments + 1
    );
    release_test_commit_snapshot(&runtime, &result);
}
fn controller_memory() -> u64 {
    let budget = SerialMemoryBudget::new(Some(64 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut observed = None;
    request
        .run_native_with_checkpoint(|_| observed = Some(budget.charged_memory_bytes()))
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    observed.unwrap()
}
