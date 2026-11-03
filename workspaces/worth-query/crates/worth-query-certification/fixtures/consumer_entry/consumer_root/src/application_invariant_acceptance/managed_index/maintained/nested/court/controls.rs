//! Original cancellation after real staging disposes deep partial trees and retries.
use super::*;
use std::sync::Mutex;
use worth_query_host::facade::admission::authenticated_principal::*;
#[test]
fn cancelled_staged_nested_source_releases_original_storage_and_allows_fresh_retry() {
    let (world, calls, records, mut model, mut key) = setup();
    for (from, to) in [("x", "a"), ("a", "z"), ("z", "c"), ("q", "x"), ("q", "p")] {
        link(&world, &mut model, &mut key, from, to, 1);
    }
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
    let before = request.query(find("A", "match")).execute().unwrap();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let memory = super::super::super::court::accounted_request();
    let baseline = memory.charged_memory_bytes();
    let observed = Arc::new(Mutex::new(None));
    let reached = Arc::clone(&observed);
    let charged = Arc::clone(&memory);
    records.records.lock().unwrap().clear();
    *calls.after_first_node.lock().unwrap() = Some(Box::new(move || {
        *reached.lock().unwrap() = Some(charged.charged_memory_bytes());
        cancellation.cancel();
    }));
    key += 1;
    let result = request
        .advanced()
        .mutate(mutation::Rename {
            directory: "A".into(),
            prefix: "match".into(),
            item: "x".into(),
            replacement: "stopped-root".into(),
        })
        .expect_result_set(before.result_set_observation().unwrap().clone())
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::Execution(&memory))
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = result
    else {
        panic!("original cancellation must refuse publication: {result:?}");
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
        observed
            .lock()
            .unwrap()
            .is_some_and(|bytes| bytes > baseline),
        "the actual original payer held staged backing before cancellation"
    );
    assert!(
        records.records.lock().unwrap().is_empty(),
        "stopped Source preparation cannot invoke the rule"
    );
    assert_eq!(
        memory.charged_memory_bytes(),
        baseline,
        "partial Native/Query trees release original storage after interruption"
    );
    let fresh = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    assert_eq!(
        fresh.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        exact(fresh.query(find("A", "match")).execute().unwrap().rows()),
        exact(before.rows())
    );
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Name {
            item: "x".into(),
            value: "match-fresh".into(),
        },
    );
    assert_eq!(
        values(fresh.query(find("A", "match")).execute().unwrap().rows()),
        model.expected("A", "match", true)
    );
}
