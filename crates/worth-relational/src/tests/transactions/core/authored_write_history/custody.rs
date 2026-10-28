use super::*;

#[test]
fn discarded_preparation_never_becomes_an_authored_occurrence() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let original = test_owner_main_basis(&runtime).unwrap();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            update_batch(entity, "discarded"),
            execution.allocation_policy(),
        )
        .unwrap();
    let prepared = runtime
        .prepare_branch_transaction(&execution, transaction)
        .unwrap();
    runtime.discard_prepared_candidate(prepared).unwrap();
    let target = test_owner_main_basis(&runtime).unwrap();
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
    let performed = update_entity(&execution, &runtime, entity, "after");
    let target = test_owner_main_basis(&runtime).unwrap();
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
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let original = test_owner_main_basis(&runtime).unwrap();
    let changed = update_entity(&execution, &runtime, entity, "after");
    let target = test_owner_main_basis(&runtime).unwrap();
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
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let original = test_owner_main_basis(&runtime).unwrap();
    let equal = update_entity(&execution, &runtime, entity, "before");
    let changed = update_entity(&execution, &runtime, entity, "after");
    let target = test_owner_main_basis(&runtime).unwrap();
    {
        let generation = runtime
            .history
            .authored_write_generation(target.observation().commit_id().unwrap())
            .unwrap();
        assert!(
            generation
                .prefix
                .accounted_bytes()
                .is_some_and(|bytes| bytes > 0),
            "serial prefix cells retain their original request grants"
        );
        assert!(
            generation
                .writes
                .accounted_bytes()
                .is_some_and(|bytes| bytes > 0),
            "serial key cells retain their original request grants"
        );
        assert!(
            generation
                .writes
                .values()
                .all(|postings| postings.0.accounted_bytes().is_some_and(|bytes| bytes > 0)),
            "nested posting cells are funded too"
        );
    }
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
    let mut unavailable = cursor(&runtime, execution, &original, &target, entity);
    assert!(runtime
        .history_authority()
        .evict_commit_envelope_for_durable_recovery_test(equal.commit.commit_id));
    for _ in 0..2 {
        assert!(
            matches!(
                unavailable.next_page(&runtime, execution, limits(1)),
                Err(RelationalAuthoredWriteDenial::CanonicalEvidenceUnavailable)
            ),
            "explicit owner eviction never becomes an empty page or advanced continuation"
        );
    }
    for outcome in [&created, &equal, &changed] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
