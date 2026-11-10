//! A stop that arrives after World published reports the published outcome
//! (public #132).
//!
//! On the worker path the native handoff refuses a pending stop after the body
//! completes. Once World has answered a publication of that body with an
//! effect, the advancement keeps the body's own outcome. A stop that reaches a
//! body which published nothing is still refused at the handoff, and the value
//! is dropped.
use super::kept_refusals::{demand, edit, settled_host, stoppable};
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;

/// Consults counted back from the end of the shrink's unstopped run: past the
/// rebase and the lineage records, where a cancel races completion. The stop
/// keeps about 100 consults of margin against a run-to-run jitter of about 20.
const NEAR_THE_END: usize = 100;

fn stopped_near_the_end(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let (performed, total, commits) = edit(&settled_host(worker), 2, None);
    assert!(
        performed && commits == 1,
        "the unstopped shrink is one commit"
    );
    let host = settled_host(worker);
    let (performed, consults, commits) = edit(&host, 2, Some(total - NEAR_THE_END));
    assert!(
        consults >= total - NEAR_THE_END,
        "the stop fired: {consults} consults"
    );
    assert!(
        performed,
        "the stop lands after the World commit, which it cannot undo"
    );
    assert_eq!(commits, 1);
    let ((settled, _, _), _) = stoppable(&host, None, |request| demand(&host, request));
    assert_eq!(settled, Ok(()), "the shrink's outputs settle");
}

/// The handler denies values above 1000, so this edit publishes nothing.
fn denied_edit(host: &kept_refusals::Host, stop_at: Option<usize>) -> (String, usize, usize) {
    let ((outcome, consults), work) = observe_maintenance_work(64, || {
        stoppable(host, stop_at, |request| {
            let outcome = request
                .advanced()
                .mutate(edit::Intent {
                    key: "a".into(),
                    value: 2000,
                })
                .without_source()
                .idempotency(&"late-stop-denied".to_owned())
                .execute_performed::<program::Synchronous, program::Tree>(
                    host,
                    ExecutionAllocationPolicy::SystemAllocation,
                );
            match outcome {
                Ok(WorthQueryApplicationPerformedMutationOutcome::Performed(_)) => {
                    "Performed".to_owned()
                }
                Ok(WorthQueryApplicationPerformedMutationOutcome::NotPerformed(_)) => {
                    "NotPerformed".to_owned()
                }
                Ok(_) => "other".to_owned(),
                Err(cause) => format!("Err {cause:?}"),
            }
        })
    });
    (
        outcome,
        consults,
        work.published_rows.len() + work.omitted_publications,
    )
}

/// A worker edit that publishes nothing, stopped at its last consult (the
/// native handoff), is refused exactly as before: nothing took effect.
#[test]
fn worker_only_unpublished_edit_stopped_at_its_handoff_is_still_refused() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let (unstopped, total, commits) = denied_edit(&settled_host(true), None);
    assert_eq!(
        (unstopped.as_str(), commits),
        ("NotPerformed", 0),
        "the handler denies"
    );
    let host = settled_host(true);
    let (stopped, consults, commits) = denied_edit(&host, Some(total));
    assert!(consults >= total, "the stop fired: {consults} consults");
    assert_eq!(commits, 0);
    assert_eq!(
        stopped, "Err Mutation(ExecutionRequest(Interrupted(Cancelled)))",
        "no publication, so the handoff refusal stands"
    );
}

/// Counts drops of the value a stopped body returns.
struct DropWitness(Arc<AtomicUsize>);
impl Drop for DropWitness {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// A body that ignores a stop, publishes nothing and returns a value is
/// refused at the native handoff, and the value is dropped, not returned.
fn ignored_stop_is_refused(host: &kept_refusals::Host) {
    let dropped = Arc::new(AtomicUsize::new(0));
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        WorthQueryAdmissionClock::system(),
    );
    let witness = Arc::clone(&dropped);
    let outcome = host
        .runtime()
        .with_application_advancement(&scope, |_phase| {
            cancellation.cancel();
            DropWitness(witness)
        });
    assert!(
        matches!(
            outcome,
            Err(WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            ))
        ),
        "the stop is refused at the handoff"
    );
    assert_eq!(dropped.load(Ordering::SeqCst), 1, "the value is dropped");
}

#[test]
fn worker_only_body_that_ignores_a_stop_and_publishes_nothing_is_refused() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    ignored_stop_is_refused(&settled_host(true));
}

/// A publishing advancement leaves no marker behind: the next, non-publishing
/// advancement on the same thread is still refused at its handoff.
#[test]
fn worker_only_publication_leaves_no_marker_for_the_next_advancement() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = settled_host(true);
    let (performed, _, commits) = edit(&host, 7, None);
    assert!(performed && commits == 1, "the first advancement publishes");
    ignored_stop_is_refused(&host);
}

#[test]
fn a_shrink_stopped_near_its_end_reports_performed() {
    stopped_near_the_end(false);
}
#[test]
fn worker_only_shrink_stopped_near_its_end_reports_performed() {
    stopped_near_the_end(true);
}
