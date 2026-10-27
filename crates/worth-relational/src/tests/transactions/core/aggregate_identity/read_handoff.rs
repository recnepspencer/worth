//! Native custom entry, proposal retention and final history use one aggregate artifact.
use super::fixtures::*;
use super::read_handoff_rule::{registration, Control, ReadMode};
use super::read_handoff_truth::assert_published;
use crate::aggregate_identity::AggregateReadDenial;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::tests::support::*;
use crate::validation::data::{InvariantViolationFields, RelationCardinalityBoundary};
use std::num::{NonZeroU64, NonZeroUsize};
use std::sync::Arc;
use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy as Allocation, ExecutionRequest,
    SerialMemoryBudget, SerialRequest,
};
use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};

fn with_readers(
    control: &Arc<Control>,
    mode: ReadMode,
    limit: Option<NonZeroU64>,
    count: usize,
) -> RelationalRuntime {
    (0..count)
        .fold(
            RelationalRuntimeApi::builder().schema_registry(registry()),
            |builder, id| builder.custom_invariant(registration(id, mode, limit, control)),
        )
        .build()
}

fn publish(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    batch: WorkerIntentBatch,
) -> Result<CommitResult, TransactionCommitError> {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(batch, Allocation::SystemAllocation)
        .unwrap();
    transaction.commit(execution, runtime)
}

#[test]
fn created_members_keep_one_original_projection_payer_through_two_rules_and_final_seal() {
    let memory = SerialMemoryBudget::new(Some(128 * 1024 * 1024));
    let host = SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let execution = &ExecutionRequest::serial(&host);
    let control = Arc::new(Control::default());
    control.arm(Vec::new());
    let runtime = with_readers(&control, ReadMode::Proposed, None, 2);
    let born = publish(execution, &runtime, birth(&[1, 2, 3, 0])).unwrap();
    assert_published(execution, &runtime, &born, &control, None);
    release_test_commit_snapshot(&runtime, &born);
    let held = memory.charged_memory_bytes();
    assert!(
        held > 0,
        "reads survive proposal/write consumption on their original payer"
    );
    let survivor = control.captures.lock().unwrap()[0].proposed[0].clone();
    control.captures.lock().unwrap().clear();
    assert_eq!(
        memory.charged_memory_bytes(),
        held,
        "every rule/phase retained the same conservative grant"
    );
    assert_eq!(survivor.members().collect::<Vec<_>>(), members(&born, 4));
    drop(survivor);
    assert_eq!(memory.charged_memory_bytes(), 0);
    assert!(runtime.record_identity.pending_snapshot().is_empty());
}

#[test]
fn proposed_split_merge_and_before_image_agree_with_complete_published_tuples() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let control = Arc::new(Control::default());
    let runtime = with_readers(&control, ReadMode::Both, None, 2);
    let born = publish(execution, &runtime, birth(&[1, 2, 3, 0])).unwrap();
    let ids = members(&born, 4);
    control.arm(vec![ids[0], ids[2]]);
    let split = publish(
        execution,
        &runtime,
        rewire(
            execution,
            &runtime,
            &born.snapshot,
            &[(ids[1], ids[0]), (ids[3], ids[2])],
        ),
    )
    .unwrap();
    assert_published(execution, &runtime, &split, &control, Some(&born.snapshot));
    let old = identity(&records(execution, &runtime, &born.snapshot), ids[0]);
    let new = identity(&records(execution, &runtime, &split.snapshot), ids[2]);
    assert_ne!(old, new);
    control.arm(vec![ids[0], ids[2]]);
    let merged = publish(
        execution,
        &runtime,
        rewire(
            execution,
            &runtime,
            &split.snapshot,
            &[(ids[1], ids[2]), (ids[3], ids[0])],
        ),
    )
    .unwrap();
    assert_published(
        execution,
        &runtime,
        &merged,
        &control,
        Some(&split.snapshot),
    );
    assert_eq!(
        identity(&records(execution, &runtime, &merged.snapshot), ids[0]),
        old
    );
    control.captures.lock().unwrap().clear();
    for result in [&born, &split, &merged] {
        release_test_commit_snapshot(&runtime, result);
    }
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn malformed_permutation_retains_exact_native_law_and_never_contacts_custom_reader() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let control = Arc::new(Control::default());
    let runtime = with_readers(&control, ReadMode::Proposed, None, 1);
    let born = publish(execution, &runtime, birth(&[1, 2, 0])).unwrap();
    let ids = members(&born, 3);
    control.arm(vec![ids[0]]);
    let before = runtime.history().branch_head(&BranchId("main".into()));
    let error = publish(
        execution,
        &runtime,
        rewire(execution, &runtime, &born.snapshot, &[(ids[0], ids[0])]),
    )
    .unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("original Native conflict: {error:?}")
    };
    let crate::transactions::data::ConflictClass::InvariantViolation {
        fields:
            InvariantViolationFields::RelationCardinalityEndpoint {
                contract_id,
                relation_kind_id,
                entity_id,
                boundary,
                count,
                limit,
            },
        ..
    } = error.class
    else {
        panic!("original cardinality fields: {error:?}")
    };
    assert_eq!(contract_id.as_str(), "successor:orbit");
    assert_eq!(relation_kind_id, KindId(2));
    assert!(
        matches!(entity_id, crate::transactions::data::EntityReference::Existing(id)
        if (id == ids[0] && count == 2) || (id == ids[1] && count == 0))
    );
    assert_eq!(boundary, RelationCardinalityBoundary::Target);
    assert_eq!(limit, 1);
    assert!(control.captures.lock().unwrap().is_empty());
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".into())),
        before
    );
    assert!(runtime.record_identity.pending_snapshot().is_empty());
    let lawful = publish(
        execution,
        &runtime,
        rewire(execution, &runtime, &born.snapshot, &[(ids[0], ids[1])]),
    )
    .unwrap();
    assert!(control
        .captures
        .lock()
        .unwrap()
        .iter()
        .any(|capture| capture.evaluation && !capture.proposed.is_empty()));
    control.captures.lock().unwrap().clear();
    for result in [&born, &lawful] {
        release_test_commit_snapshot(&runtime, result);
    }
}

#[test]
fn swallowed_original_memory_refusal_cannot_publish_and_funded_twin_succeeds() {
    let memory = SerialMemoryBudget::new(Some(128 * 1024 * 1024));
    let host = SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let execution = &ExecutionRequest::serial(&host);
    let control = Arc::new(Control::default());
    let runtime = with_readers(&control, ReadMode::Proposed, None, 1);
    let born = publish(execution, &runtime, birth(&[1, 0])).unwrap();
    let ids = members(&born, 2);
    let batch = rewire(
        execution,
        &runtime,
        &born.snapshot,
        &[(ids[0], ids[0]), (ids[1], ids[1])],
    );
    let before = runtime.history().branch_head(&BranchId("main".into()));
    control.arm(vec![ids[0]]);
    *control.pressure.lock().unwrap() = Some(memory.clone());
    let error = publish(execution, &runtime, batch.clone()).unwrap_err();
    assert!(matches!(cause(error), Cause::PolicyMemoryExhausted { .. }));
    assert!(control
        .captures
        .lock()
        .unwrap()
        .iter()
        .any(|capture| capture.errors.iter().any(|error| matches!(
            error,
            AggregateReadDenial::Execution(Cause::PolicyMemoryExhausted { .. })
        ))));
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".into())),
        before
    );
    assert!(runtime.record_identity.pending_snapshot().is_empty());
    control.held.lock().unwrap().take();
    control.pressure.lock().unwrap().take();
    control.arm(vec![ids[0]]);
    let funded = publish(execution, &runtime, batch).unwrap();
    assert!(control
        .captures
        .lock()
        .unwrap()
        .iter()
        .any(|capture| capture.evaluation && !capture.proposed.is_empty()));
    control.captures.lock().unwrap().clear();
    for result in [&born, &funded] {
        release_test_commit_snapshot(&runtime, result);
    }
    assert_eq!(memory.charged_memory_bytes(), 0);
}

#[test]
fn deliberate_stored_group_spends_rule_allowance_with_accounted_and_unaccounted_twins() {
    for accounted in [false, true] {
        for limit in [4_000, 200_000] {
            let control = Arc::new(Control::default());
            let runtime = with_readers(&control, ReadMode::Committed, NonZeroU64::new(limit), 1);
            let setup = crate::test_execution_policy::serial_request();
            let setup_execution = &ExecutionRequest::serial(&setup);
            let ring = (0..128).map(|index| (index + 1) % 128).collect::<Vec<_>>();
            let born = publish(setup_execution, &runtime, birth(&ring)).unwrap();
            let ids = members(&born, 128);
            let batch = rewire(
                setup_execution,
                &runtime,
                &born.snapshot,
                &[(ids[0], ids[1])],
            );
            control.arm(vec![ids[0]]);
            let policy = ExecutionRequestPolicy::for_workers(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                NonZeroUsize::MIN,
            );
            let unaccounted =
                SerialRequest::from_policy(&policy, CancellationSource::new().token(), None);
            let lease = crate::tests::support::test_execution_authority()
                .request_lease(worth_execution::LeaseRequest {
                    policy: ExecutionRequestPolicy::new(
                        ExecutionPosture::Serial,
                        DeterminismContract::CanonicalBitwise,
                        worth_foundational::ExecutionBudget::new(
                            NonZeroUsize::MIN,
                            Some(128 * 1024 * 1024),
                            Some(20_000_000),
                        ),
                    ),
                    cancellation: CancellationSource::new().token(),
                    deadline: None,
                })
                .unwrap();
            let execution = if accounted {
                ExecutionRequest::leased(&lease)
            } else {
                ExecutionRequest::serial(&unaccounted)
            };
            assert_eq!(execution.has_accounting(), accounted);
            let before = runtime.history().branch_head(&BranchId("main".into()));
            let result = publish(&execution, &runtime, batch);
            let captures = control.captures.lock().unwrap();
            assert!(
                !captures.is_empty(),
                "actual reader must be reached before the rule limit refuses"
            );
            if limit == 4_000 {
                assert_eq!(cause(result.unwrap_err()), Cause::WorkExhausted);
                assert!(captures.iter().any(|capture| capture
                    .errors
                    .contains(&AggregateReadDenial::Execution(Cause::WorkExhausted))));
                assert_eq!(
                    runtime.history().branch_head(&BranchId("main".into())),
                    before
                );
            } else {
                let result = result.unwrap();
                assert!(captures.iter().any(|capture| capture.evaluation
                    && capture.committed.iter().any(|read| read.len() == 128)));
                release_test_commit_snapshot(&runtime, &result);
            }
            drop(captures);
            control.captures.lock().unwrap().clear();
            release_test_commit_snapshot(&runtime, &born);
            assert_eq!(lease.charged_memory_bytes(), 0);
            assert!(runtime.record_identity.pending_snapshot().is_empty());
        }
    }
}

#[test]
fn cancellation_after_real_preparation_and_discard_release_original_custody() {
    let setup = crate::test_execution_policy::serial_request();
    let setup_execution = &ExecutionRequest::serial(&setup);
    let control = Arc::new(Control::default());
    let runtime = with_readers(&control, ReadMode::Proposed, None, 1);
    let born = publish(setup_execution, &runtime, birth(&[1, 0])).unwrap();
    let ids = members(&born, 2);
    let batch = rewire(
        setup_execution,
        &runtime,
        &born.snapshot,
        &[(ids[0], ids[0]), (ids[1], ids[1])],
    );
    let before = runtime.history().branch_head(&BranchId("main".into()));
    control.arm(vec![ids[0]]);
    let cancellation = CancellationSource::new();
    *control.cancel.lock().unwrap() = Some(cancellation.clone());
    let host =
        crate::test_execution_policy::serial_request().with_cancellation(cancellation.token());
    let error = publish(&ExecutionRequest::serial(&host), &runtime, batch.clone()).unwrap_err();
    assert_eq!(cause(error), Cause::Cancelled);
    assert!(control
        .captures
        .lock()
        .unwrap()
        .iter()
        .any(|capture| !capture.proposed.is_empty()));
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".into())),
        before
    );
    control.captures.lock().unwrap().clear();
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
    control.cancel.lock().unwrap().take();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch, Allocation::SystemAllocation)
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(setup_execution, transaction)
        .unwrap();
    assert!(candidate.reservation_count() > 0);
    runtime.discard_prepared_candidate(candidate).unwrap();
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".into())),
        before
    );
    assert!(runtime.record_identity.pending_snapshot().is_empty());
    control.captures.lock().unwrap().clear();
    release_test_commit_snapshot(&runtime, &born);
    assert_eq!(setup.memory().unwrap().charged_memory_bytes(), 0);
}

fn cause(error: TransactionCommitError) -> Cause {
    let TransactionCommitError::Execution { denial, .. } = error else {
        panic!("original execution cause required: {error:?}")
    };
    let crate::transactions::data::CommitExecutionDenialKind::Cause(cause) = denial.kind;
    cause
}
