//! Owned completion reports observe request custody without extending its lifetime.
use super::WorthQueryAdvancementDenial;
use std::cell::RefCell;
use worth_foundational::ExecutionReport;
thread_local! {
    static REQUESTS: RefCell<Vec<Result<ExecutionReport, WorthQueryAdvancementDenial>>> = const { RefCell::new(Vec::new()) };
}
pub(super) fn record_report(report: ExecutionReport) {
    REQUESTS.with(|requests| requests.borrow_mut().push(Ok(report)));
}
pub(super) fn record_result<R>(result: &Result<R, WorthQueryAdvancementDenial>) {
    if let Err(cause) = result {
        REQUESTS.with(|requests| requests.borrow_mut().push(Err(*cause)));
    }
}
/// Takes this thread's completed request reports, including admission refusals.
pub fn advancement_requests_on_this_thread_for_test(
) -> Vec<Result<ExecutionReport, WorthQueryAdvancementDenial>> {
    REQUESTS.with(|requests| std::mem::take(&mut *requests.borrow_mut()))
}

#[cfg(feature = "test-query-execution-observer")]
thread_local! {
    static CALLER_PASSES: RefCell<Vec<(Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>, u64)>> = const { RefCell::new(Vec::new()) };
}
/// Observes pass results without adding an execution scope or changing control flow.
#[cfg(feature = "test-query-execution-observer")]
pub fn caller_pass_reports_on_this_thread_for_test() -> Vec<(
    Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>,
    u64,
)> {
    CALLER_PASSES.with(|passes| std::mem::take(&mut *passes.borrow_mut()))
}
#[cfg(feature = "test-query-execution-observer")]
pub(in crate::domain_computation::primary_graph) fn record_caller_pass<R>(
    result: &Result<R, crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial>,
) {
    CALLER_PASSES.with(|passes| {
        passes.borrow_mut().push((
            result.as_ref().map(|_| ()).map_err(|denial| denial.kind()),
            worth_signal::facade::observed_signal_request_work_on_this_thread_for_test(),
        ))
    });
}
