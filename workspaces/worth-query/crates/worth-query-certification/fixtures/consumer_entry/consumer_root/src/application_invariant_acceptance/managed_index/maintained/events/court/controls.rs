//! Original caller cancellation during actual Source staging cannot publish.
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use worth_query_host::facade::admission::authenticated_principal::*;
#[test]
fn cancelled_event_source_staging_preserves_head_and_allows_a_fresh_request() {
    let (world, calls, records, mut key) = setup();
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let request = world
        .runtime()
        .request(&principal, &stopped)
        .on_branch(world.current_world());
    let before = request.query(find("D", "")).execute().unwrap();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let reached = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&reached);
    records.records.lock().unwrap().clear();
    *calls.after_first_node.lock().unwrap() = Some(Box::new(move || {
        observed.store(true, Ordering::SeqCst);
        cancellation.cancel();
    }));
    key += 1;
    let result = request
        .advanced()
        .mutate(mutation::Rename {
            directory: "D".into(),
            prefix: "".into(),
            item: "x".into(),
            replacement: "stopped-root".into(),
        })
        .expect_result_set(before.result_set_observation().unwrap().clone())
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = result
    else {
        panic!("cancelled Source must refuse: {result:?}");
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
    assert!(reached.load(Ordering::SeqCst));
    assert!(records.records.lock().unwrap().is_empty());
    let fresh = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    assert_eq!(
        fresh.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        fresh
            .query(query::ReadSource {
                directory: "D".into()
            })
            .execute()
            .unwrap()
            .rows()
            .iter()
            .find(|row| row.identity == "x")
            .unwrap()
            .name,
        "match-x"
    );
}
