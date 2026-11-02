//! Stop after this public provider writes real staged nodes, then retry.
use super::super::super::super::court::accounted_request;
use super::*;
use std::sync::Mutex;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
pub(super) fn run(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
    calls: &Arc<provider::Calls>,
) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let request = world
        .runtime()
        .request(principal, &scope)
        .on_branch(world.current_world());
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let memory = accounted_request();
    let baseline = memory.charged_memory_bytes();
    let observed_memory = Arc::clone(&memory);
    let reached = Arc::new(Mutex::new(None));
    let observed = Arc::clone(&reached);
    *calls.after_node_write.lock().unwrap() = Some(Box::new(move || {
        *observed.lock().unwrap() = Some(observed_memory.charged_memory_bytes());
        cancellation.cancel();
    }));
    let edit = Edit::Move {
        directory: "A".into(),
        vertex: "v1".into(),
        x: 25,
        y: 0,
    };
    let outcome = request
        .advanced()
        .mutate(edit.clone())
        .idempotency(&0x919521u64)
        .execute_in_program(world, ExecutionAllocationPolicy::Execution(&memory))
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("actual public Source cancellation must refuse: {outcome:?}")
    };
    assert_eq!(
        denial.invariant_execution_failure().unwrap().kind(),
        WorthQueryInvariantExecutionDenialKind::RequestInterrupted(
            RelationalOperationInterruption::Cancelled
        )
    );
    drop(denial);
    assert!(reached
        .lock()
        .unwrap()
        .is_some_and(|bytes| bytes > baseline));
    assert_eq!(memory.charged_memory_bytes(), baseline);
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
        values(fresh.query(all("B")).execute().unwrap().rows()),
        expected("B", 20, all("B").probe)
    );
    commit(world, principal, &fresh_scope, edit, 0x919521);
    assert_eq!(
        values(fresh.query(all("B")).execute().unwrap().rows()),
        expected("B", 25, all("B").probe)
    );
}
