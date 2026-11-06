//! Actual final handoff refuses source races and cancellation before owner moves.
use super::*;
use std::time::Duration;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-scope=1 scoped-thread-spawn=1: source_change_after_proof_keeps_the_original_context_for_retry joins the bounded native owner and competing host or control actors to certify custody at the real handoff."
)]
fn source_change_after_proof_keeps_the_original_context_for_retry() {
    let host = publish_on_first_program();
    let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let original = document.current_step(&request).unwrap();
    let branch = original.endpoint().product_branch();
    outside(&host, branch, "revoked", 0x9182_b500);
    let pause = document.pause_before_history_context_refresh_for_test();
    let (reached, update, result) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| document.refresh_context(&request, limits(1)));
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        let update = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert!(reached, "actual Native proof must reach final guards");
            outside(&host, branch, "active", 0x9182_b501);
        }));
        pause.release();
        (reached, update, worker.join())
    });
    assert!(reached);
    update.unwrap();
    let refused = result.unwrap().unwrap_err();
    assert!(matches!(refused, Failure::Document(Native::NativeBasis(worth_relational::facade::branch::RelationalBranchBasisDenial::UnavailableRetainedTarget))), "final actual Native source guard: {refused:?}");
    bound(document.refresh_context(&request, limits(1)).unwrap_err());
    document.refresh_context(&request, limits(2)).unwrap();
    assert_eq!(document.current_step(&request).unwrap(), original);
    close(&host, document, 1);
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-scope=1 scoped-thread-spawn=1: cancellation_after_proof_keeps_the_same_document_retryable joins the bounded native owner and competing host or control actors to certify custody at the real handoff."
)]
fn cancellation_after_proof_keeps_the_same_document_retryable() {
    let host = publish_on_first_program();
    let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let original = document.current_step(&request).unwrap();
    outside(
        &host,
        original.endpoint().product_branch(),
        "revoked",
        0x9182_b600,
    );
    let cancel = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancel.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let stopped = host.request(&principal, &stopped_scope);
    let pause = document.pause_before_history_context_refresh_for_test();
    let (reached, result) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| document.refresh_context(&stopped, limits(1)));
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        cancel.cancel();
        pause.release();
        (reached, worker.join())
    });
    assert!(reached, "actual proof precedes cancellation");
    let denied = result.unwrap().unwrap_err();
    assert!(matches!(denied, Failure::Document(Native::Stopped(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled))))
        || matches!(denied, Failure::ExecutionRequest(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled)))
        || matches!(denied, Failure::Document(Native::NativeBasis(worth_relational::facade::branch::RelationalBranchBasisDenial::Cancelled))), "original request stop: {denied:?}");
    assert!(matches!(
        document.current_step(&request).unwrap_err(),
        Failure::Document(Native::HeadDiverged)
    ));
    document.refresh_context(&request, limits(1)).unwrap();
    assert_eq!(document.current_step(&request).unwrap(), original);
    close(&host, document, 1);
}
