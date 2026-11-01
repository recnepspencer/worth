//! A real staged page is discarded on cancellation or an outside head move.
use super::*;
use std::time::Duration;
use worth_query_testkit_document_retention::facade::document_retention_model::{
    operator_identity::{WorthQueryCancellationSource, WorthQueryRequestScope},
    presented_request::set_retention,
    settled_verdict::{settle, RetentionVerdict},
};

#[test]
fn cancellation_after_assembly_preserves_the_cursor_for_a_fresh_read() {
    let host = publish_on_first_program();
    let (mut document, expected) = chain(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let cursor = document
        .history_page(&request, limits(1), None)
        .unwrap()
        .continuation()
        .unwrap();
    let cancel = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancel.token());
    let stopped = host.runtime().request(&principal, &stopped_scope);
    let pause = document.pause_after_history_page_assembly_for_test();
    let (reached, result) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| document.history_page(&stopped, limits(1), Some(&cursor)));
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        cancel.cancel();
        pause.release();
        (reached, worker.join())
    });
    assert!(
        reached,
        "the actual page must be staged before cancellation"
    );
    let refused = result.unwrap().unwrap_err();
    assert!(matches!(refused, Failure::Document(Native::Paging(PageFailure::Request(worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)))))
        || matches!(refused, Failure::ExecutionRequest(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled))), "original cancellation, no page: {refused:?}");
    let page = document
        .history_page(&request, limits(1), Some(&cursor))
        .unwrap();
    assert_page_entry(page.rows()[0].step(), &expected[1]);
    close(&host, document, 4);
}

#[test]
fn an_outside_head_move_after_assembly_refuses_the_staged_page() {
    let host = publish_on_first_program();
    let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let branch = document
        .current_step(&request)
        .unwrap()
        .endpoint()
        .product_branch();
    let pause = document.pause_after_history_page_assembly_for_test();
    let (reached, update, result) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| document.history_page(&request, limits(1), None));
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        let update = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if reached {
                settle(set_retention(&host, branch, 3, 0x9182_9040))
            } else {
                panic!("assembly was not reached");
            }
        }));
        pause.release();
        (reached, update, worker.join())
    });
    assert!(reached);
    assert_eq!(update.unwrap(), RetentionVerdict::Performed(3));
    let refused = result.unwrap().unwrap_err();
    assert!(
        matches!(
            refused,
            Failure::Document(Native::NativeBasis(
                worth_relational::facade::branch::RelationalBranchBasisDenial::UnavailableRetainedTarget
            ))
        ),
        "the original selected source changed: {refused:?}"
    );
    close(&host, document, 1);
}
