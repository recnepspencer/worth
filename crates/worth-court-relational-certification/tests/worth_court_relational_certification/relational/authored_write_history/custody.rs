use super::super::durable_publication::branch_history_publications::update_entity;
use super::super::proposal_records::{name_field_patch, update_name};
use super::{admit_main, collect, cursor, keys, limits, selected};
use std::num::NonZeroUsize;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::RelationalAuthoredWriteDenial,
    transactions::{
        EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::{
        begin_main_transaction, changed_entities, release_test_commit_snapshot,
    },
    durable_publication::{create_entity_outcome, memory_runtime as runtime_with_test_schema},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn discarded_preparation_never_becomes_an_authored_occurrence() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let original = admit_main(&runtime).unwrap();
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("authored-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: name_field_patch("discarded"),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let prepared = runtime
        .prepare_branch_transaction(&execution, transaction)
        .unwrap();
    runtime.discard_prepared_candidate(prepared).unwrap();
    let target = admit_main(&runtime).unwrap();
    assert_eq!(
        target.observation().commit_id(),
        original.observation().commit_id()
    );
    assert!(collect(
        &runtime,
        execution,
        cursor(&runtime, execution, &original, &target, entity)
    )
    .0
    .is_empty());
    let performed = update_name(&execution, &runtime, entity, "after");
    let target = admit_main(&runtime).unwrap();
    assert_eq!(
        collect(
            &runtime,
            execution,
            cursor(&runtime, execution, &original, &target, entity)
        )
        .0,
        [performed.commit.commit_id]
    );
    for outcome in [&created, &performed] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn stopped_page_handoff_keeps_original_position_and_live_canonical_owners() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let original = admit_main(&runtime).unwrap();
    let changed = update_name(&execution, &runtime, entity, "after");
    let target = admit_main(&runtime).unwrap();
    let cursor = cursor(&runtime, execution, &original, &target, entity);
    let cancelled = worth_execution::CancellationSource::new();
    let request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancelled.token(),
        None,
    );
    let staged = cursor
        .prepare_next_page(&runtime, ExecutionRequest::serial(&request), limits(1))
        .unwrap();
    assert_eq!(
        staged.page().entries().next().unwrap().commit_id(),
        changed.commit.commit_id
    );
    cancelled.cancel();
    assert!(matches!(
        cursor.accept_page(staged),
        Err(RelationalAuthoredWriteDenial::Request(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        ))
    ));
    let finite = worth_execution::SerialRequest::from_policy(
        &worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(NonZeroUsize::MIN, None, Some(1_000_000)),
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let finite_request = ExecutionRequest::serial(&finite);
    let stopped = finite_request.run_native_with_checkpoint(|work| {
        let staged = cursor
            .prepare_next_page(&runtime, finite_request, limits(1))
            .unwrap();
        let beyond_remaining = work
            .remaining_work()
            .expect("finite original work ceiling")
            .checked_add(1)
            .unwrap();
        assert_eq!(
            work.checkpoint(beyond_remaining),
            Err(worth_execution::MapKernelStop::WorkCeiling)
        );
        assert!(matches!(
            cursor.accept_page(staged),
            Err(RelationalAuthoredWriteDenial::Request(
                worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::WorkCeiling
                )
            ))
        ));
    });
    assert_eq!(
        stopped.unwrap_err(),
        worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::WorkCeiling)
    );
    let empty = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(0)),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    assert!(cursor
        .prepare_next_page(&runtime, ExecutionRequest::serial(&empty), limits(1))
        .is_err());
    assert_eq!(
        collect(&runtime, execution, cursor).0,
        [changed.commit.commit_id]
    );
    for outcome in [&created, &changed] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn explicit_discard_and_rebuild_preserve_canonical_noop_occurrences() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let original = admit_main(&runtime).unwrap();
    let equal = update_entity(&execution, &runtime, entity, "before");
    let changed = update_entity(&execution, &runtime, entity, "after");
    let target = admit_main(&runtime).unwrap();
    let old_cursor = cursor(&runtime, execution, &original, &target, entity);
    let expected = collect(
        &runtime,
        execution,
        cursor(&runtime, execution, &original, &target, entity),
    );
    let empty = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(0)),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    assert!(matches!(
        runtime.rebuild_authored_write_generation(
            ExecutionRequest::serial(&empty),
            selected(&runtime, &target)
        ),
        Err(RelationalAuthoredWriteDenial::Request(_))
            | Err(RelationalAuthoredWriteDenial::OrderedStorage(
                worth_execution::ExecutionOrderedMapDenial::Request(_)
            ))
    ));
    assert_eq!(
        collect(
            &runtime,
            execution,
            cursor(&runtime, execution, &original, &target, entity)
        ),
        expected,
        "failed funded reconstruction must preserve the original generation"
    );
    runtime
        .discard_authored_write_generation(execution, selected(&runtime, &target))
        .unwrap();
    assert!(matches!(
        runtime.authored_write_history_cursor(
            execution,
            &original,
            selected(&runtime, &target),
            &keys(entity)
        ),
        Err(RelationalAuthoredWriteDenial::GenerationUnavailable)
    ));
    assert_eq!(
        collect(&runtime, execution, old_cursor),
        expected,
        "existing readers retain their actual immutable grants"
    );
    let rebuilt = runtime
        .rebuild_authored_write_generation(execution, selected(&runtime, &target))
        .unwrap();
    assert!(
        rebuilt >= expected.0.len(),
        "cold ancestry work is separate from relevant writes"
    );
    assert_eq!(
        collect(
            &runtime,
            execution,
            cursor(&runtime, execution, &original, &target, entity)
        ),
        expected
    );
    for outcome in [&created, &equal, &changed] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
