//! Original-request cancellation releases partial staged backing.
use super::*;
pub(super) fn run(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<super::super::schema::Catalog>,
    provider: &Arc<prefix::Calls>,
    rules: &Arc<rule::Calls>,
) {
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let request = world
        .runtime()
        .request(principal, &stopped)
        .on_branch(world.current_world());
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let reached = Arc::new(Mutex::new(None));
    let seen = Arc::clone(&reached);
    rules.records.lock().unwrap().clear();
    let source = request.query(find("A", "fr")).execute().unwrap();
    let memory = accounted_request();
    let observed_memory = Arc::clone(&memory);
    let baseline = memory.charged_memory_bytes();
    let bases = world.runtime().application_query_basis_observer();
    let active = bases.observe().active();
    *provider.after_first_node.lock().unwrap() = Some(Box::new(move || {
        *seen.lock().unwrap() = Some(observed_memory.charged_memory_bytes());
        cancellation.cancel();
    }));
    let result = request
        .advanced()
        .mutate(rename("fr", "blocked"))
        .expect_result_set(source.result_set_observation().unwrap().clone())
        .idempotency(&0x919203u64)
        .execute_in_program(world, ExecutionAllocationPolicy::Execution(&memory));
    let outcome = result.unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("original-request cancellation must stop candidate validation: {outcome:?}")
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(
        denial.invariant_execution_failure().unwrap().kind(),
        WorthQueryInvariantExecutionDenialKind::RequestInterrupted(
            RelationalOperationInterruption::Cancelled
        )
    );
    drop(denial);
    assert!(
        reached
            .lock()
            .unwrap()
            .is_some_and(|charged| charged > baseline),
        "the original request was cancelled after a genuine staged node write"
    );
    assert!(
        rules.records.lock().unwrap().is_empty(),
        "stopped preparation cannot reach rule callbacks"
    );
    assert_eq!(
        memory.charged_memory_bytes(),
        baseline,
        "cancelled partial preparation releases original-request reservations"
    );
    assert_eq!(
        bases.observe().active(),
        active,
        "the cancelled decision query releases its admitted basis"
    );
    let fresh_scope = authentication::request_scope();
    let fresh = world
        .runtime()
        .request(principal, &fresh_scope)
        .on_branch(world.current_world());
    assert_eq!(
        fresh.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert_eq!(
        values(fresh.query(find("B", "")).execute().unwrap().rows()),
        expected("B", "free", "")
    );
    let source = fresh.query(find("A", "fr")).execute().unwrap();
    let result = fresh
        .advanced()
        .mutate(rename("fr", "fresh"))
        .expect_result_set(source.result_set_observation().unwrap().clone())
        .idempotency(&0x919204u64)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
    assert_eq!(
        values(fresh.query(find("B", "")).execute().unwrap().rows()),
        expected("B", "fresh", "")
    );
}
