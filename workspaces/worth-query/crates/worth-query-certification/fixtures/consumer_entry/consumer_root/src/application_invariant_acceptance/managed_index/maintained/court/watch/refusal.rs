//! No-effect domain and original cancellation keep the genuine watch unchanged.
use super::*;

pub(super) fn run(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<super::super::super::schema::Catalog>,
    scope: &WorthQueryRequestScope,
    calls: &Arc<prefix::Calls>,
) {
    let request = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world());
    // Deletion/recreation earlier removed B's old relation. Re-establish the
    // genuine shared membership before asking B's reserved-name rule to refuse.
    let shared = request
        .advanced()
        .mutate(Edit::Share {
            directory: "B".into(),
            item: "x".into(),
        })
        .idempotency(&0x9198_034fu64)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        shared,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let (initial, mut watch) = request
        .query(find("A", "race"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    let source = initial.result_set_observation().unwrap().clone();
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let memory = accounted_request();
    let baseline = memory.charged_memory_bytes();
    let denied = request
        .advanced()
        .mutate(rename("race", "taken"))
        .expect_result_set(source.clone())
        .idempotency(&0x9198_0350u64)
        .execute_in_program(world, ExecutionAllocationPolicy::Execution(&memory))
        .unwrap();
    require_violation(denied);
    assert_eq!(memory.charged_memory_bytes(), baseline);
    assert!(matches!(
        request
            .query(find("A", "race"))
            .next_watch(&mut watch)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    *calls.after_first_node.lock().unwrap() = Some(Box::new(move || cancellation.cancel()));
    let denied = world
        .runtime()
        .request(principal, &stopped)
        .on_branch(world.current_world())
        .advanced()
        .mutate(rename("race", "cancelled"))
        .expect_result_set(source)
        .idempotency(&0x9198_0351u64)
        .execute_in_program(world, ExecutionAllocationPolicy::Execution(&memory))
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = denied
    else {
        panic!("stopped Source preparation cannot commit: {denied:?}");
    };
    assert_eq!(
        denial.invariant_execution_failure().unwrap().kind(),
        WorthQueryInvariantExecutionDenialKind::RequestInterrupted(
            RelationalOperationInterruption::Cancelled
        )
    );
    drop(denial);
    assert_eq!(
        memory.charged_memory_bytes(),
        baseline,
        "no-effect contexts and early preparation guards release their original payer"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert!(matches!(
        request
            .query(find("A", "race"))
            .next_watch(&mut watch)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    // A new opening is the public proof that no rejected preparation fence leaked.
    let (_, fresh_watch) = request
        .query(find("A", "race"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    fresh_watch.close().unwrap();
    watch.close().unwrap();
}
