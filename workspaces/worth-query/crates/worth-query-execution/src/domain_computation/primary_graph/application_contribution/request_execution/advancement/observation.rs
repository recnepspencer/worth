//! Owned completion reports observe request custody without extending its lifetime.
use super::WorthQueryAdvancementDenial;
use std::cell::RefCell;
use worth_foundational::ExecutionReport;
thread_local! {
    static REQUESTS: RefCell<Vec<Result<ExecutionReport, WorthQueryAdvancementDenial>>> = const { RefCell::new(Vec::new()) };
}
pub(super) fn record_report(report: ExecutionReport) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    REQUESTS.with(|requests| requests.borrow_mut().push(Ok(report)));
}
pub(super) fn record_result<R>(result: &Result<R, WorthQueryAdvancementDenial>) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    if let Err(cause) = result {
        REQUESTS.with(|requests| requests.borrow_mut().push(Err(*cause)));
    }
}
/// Takes this thread's completed request reports, including admission refusals.
pub fn advancement_requests_on_this_thread_for_test(
) -> Vec<Result<ExecutionReport, WorthQueryAdvancementDenial>> {
    REQUESTS.with(|requests| std::mem::take(&mut *requests.borrow_mut()))
}

thread_local! {
    static CALLER_PASSES: RefCell<Vec<(Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>, u64)>> = const { RefCell::new(Vec::new()) };
}
/// Observes pass results without adding an execution scope or changing control flow.
pub fn caller_pass_reports_on_this_thread_for_test() -> Vec<(
    Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>,
    u64,
)> {
    CALLER_PASSES.with(|passes| std::mem::take(&mut *passes.borrow_mut()))
}
pub(in crate::domain_computation::primary_graph) fn record_caller_pass<R>(
    result: &Result<R, crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial>,
) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    CALLER_PASSES.with(|passes| {
        passes.borrow_mut().push((
            result.as_ref().map(|_| ()).map_err(|denial| denial.kind()),
            worth_signal::facade::observed_signal_request_work_on_this_thread_for_test(),
        ))
    });
}

thread_local! { static CLEANUP_REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
fn clear_observations() {
    CLEANUP_REGISTERED.with(|registered| registered.set(false));
    REQUESTS.with(|records| *records.borrow_mut() = Default::default());
    CALLER_PASSES.with(|records| *records.borrow_mut() = Default::default());
}
