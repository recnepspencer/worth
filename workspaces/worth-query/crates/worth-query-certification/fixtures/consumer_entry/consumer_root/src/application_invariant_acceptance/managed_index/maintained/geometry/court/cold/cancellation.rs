//! Cancel after an actual cold-build node write, then retry from the still-cold basis.
use super::*;
pub(super) fn run(
    world: &host::Host,
    principal: &Principal,
    fresh_scope: &WorthQueryRequestScope,
    calls: &Arc<provider::Calls>,
    rules: &Arc<rule::Calls>,
) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let request = world
        .runtime()
        .request(principal, &scope)
        .on_branch(world.current_world());
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let memory = accounted_request();
    let observed_memory = Arc::clone(&memory);
    let reached = Arc::new(Mutex::new(None));
    let observed = Arc::clone(&reached);
    let builds = calls.builds.load(Ordering::SeqCst);
    rules.records.lock().unwrap().clear();
    *calls.after_build_node_write.lock().unwrap() = Some(Box::new(move || {
        *observed.lock().unwrap() = Some(observed_memory.charged_memory_bytes());
        cancellation.cancel();
    }));
    let edit = Edit::Move {
        directory: "A".into(),
        vertex: "v1".into(),
        x: 20,
        y: 0,
    };
    let result = request
        .advanced()
        .mutate(edit.clone())
        .idempotency(&0x919463u64)
        .execute_in_program(world, ExecutionAllocationPolicy::Execution(&memory))
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = result
    else {
        panic!("cold build cancellation must refuse: {result:?}")
    };
    assert_eq!(
        denial.invariant_execution_failure().unwrap().kind(),
        WorthQueryInvariantExecutionDenialKind::RequestInterrupted(
            RelationalOperationInterruption::Cancelled
        )
    );
    drop(denial);
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds + 1);
    assert!(
        reached.lock().unwrap().is_some_and(|bytes| bytes > 0),
        "real cold node backing used the original payer"
    );
    assert!(
        rules.records.lock().unwrap().is_empty(),
        "no rule runs after stopped preparation"
    );
    assert_eq!(memory.charged_memory_bytes(), 0);
    let fresh = world
        .runtime()
        .request(principal, fresh_scope)
        .on_branch(world.current_world());
    assert_eq!(
        fresh.retain_current_read().unwrap().selected_commit(),
        &head
    );
    cold_read(world, principal, fresh_scope, calls);
    commit(world, principal, fresh_scope, edit, 0x919463);
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds + 2,
        "retry reconstructs from real Native Source again"
    );
}
