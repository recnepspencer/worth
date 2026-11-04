//! The original request stops after a real refit; fresh identical intent may retry.
use super::*;
pub(super) fn run(
    world: &host::Host,
    principal: &Principal,
    calls: &Arc<provider::Calls>,
    rules: &Arc<rule::Calls>,
) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let request = world
        .runtime()
        .request(principal, &scope)
        .on_branch(world.current_world());
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    rules.records.lock().unwrap().clear();
    let memory = accounted_request();
    let observed_memory = Arc::clone(&memory);
    let baseline = memory.charged_memory_bytes();
    let reached = Arc::new(Mutex::new(None));
    let observed = Arc::clone(&reached);
    *calls.after_node_write.lock().unwrap() = Some(Box::new(move || {
        *observed.lock().unwrap() = Some(observed_memory.charged_memory_bytes());
        cancellation.cancel();
    }));
    let edit = Edit::Move {
        directory: "A".into(),
        vertex: "v5".into(),
        x: 40,
        y: 0,
    };
    let stopped = request
        .advanced()
        .mutate(edit.clone())
        .idempotency(&0x919415u64)
        .execute_in_program(world, ExecutionAllocationPolicy::Execution(&memory))
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = stopped
    else {
        panic!("original cancellation must stop geometry validation: {stopped:?}")
    };
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
            .is_some_and(|bytes| bytes > baseline),
        "actual refit backing was admitted before cancellation"
    );
    assert!(rules.records.lock().unwrap().is_empty());
    assert_eq!(
        memory.charged_memory_bytes(),
        baseline,
        "cancelled geometry staging releases its original reservations"
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
    assert!(fresh
        .query(find("B", 35))
        .execute()
        .unwrap()
        .rows()
        .is_empty());
    commit(world, principal, &fresh_scope, edit, 0x919415);
    assert_eq!(
        values(fresh.query(find("B", 35)).execute().unwrap().rows()),
        expected("B", 40, find("B", 35).probe)
    );
}
