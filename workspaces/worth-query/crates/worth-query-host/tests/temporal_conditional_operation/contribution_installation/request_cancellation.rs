//! Caller-only cancellation after advancement opening precedes program reads.
use super::*;
use std::cell::Cell;
use std::rc::Rc;
use worth_execution::{
    CancellationSource, ExecutionRequest, RequestConsultObservationForTest, SerialMemoryBudget,
    SerialRequest,
};
#[test]
fn program_commit_keeps_caller_only_cancellation_after_advancement_opening() {
    let query = Rc::new(worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread());
    let application = super::application_fixture::installed_application();
    let installed = application.conditional::<TemporalConditional>().unwrap();
    let installed = installed.as_ref().as_ref().unwrap();
    let branch = application.current_world();
    let source = CancellationSource::new();
    let caller =
        SerialRequest::from_memory(SerialMemoryBudget::new(64 << 20), source.token(), None);
    let injected = Rc::new(Cell::new(false));
    let stopped = super::source_change::change_input_with_commit_probe(
        &ExecutionRequest::serial(&caller),
        &application,
        &installed.invariant,
        branch,
        "changed",
        || {
            let query = Rc::clone(&query);
            let source = source.clone();
            let injected = Rc::clone(&injected);
            RequestConsultObservationForTest::install_on_this_thread_for_test(move |_| {
                if query.advancement_is_active_for_test() {
                    injected.set(true);
                    source.cancel();
                }
            })
        },
    );
    assert!(
        injected.get(),
        "cancellation occurs after the advancement is admitted"
    );
    assert!(
        matches!(
            stopped,
            primary_graph::WorthQueryApplicationCommitOutcome::Cancelled
        ),
        "the caller-only cancellation must prevent commit"
    );
    let healthy = crate::test_request_policy::serial_request();
    super::source_change::change_input(
        &ExecutionRequest::serial(&healthy),
        &application,
        &installed.invariant,
        branch,
        "changed",
    )
    .require_committed()
    .expect("the stopped request leaves the lawful program commit available");
}
