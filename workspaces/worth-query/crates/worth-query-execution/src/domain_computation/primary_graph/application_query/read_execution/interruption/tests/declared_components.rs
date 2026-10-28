//! Original Query stop survives the real Native component callback and outer handoff.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::component_native_acquisitions_on_this_thread_for_test as native_entries,
    tests::application_attempt::invariant_orbits::declared_queries::{self, components::Of, read},
};
#[test]
fn original_query_cancellation_and_deadline_survive_native_component_entry() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = declared_queries::world();
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for deadline_stop in [false, true] {
        let cancellation = WorthQueryCancellationSource::new();
        let deadline = Instant::now()
            + if deadline_stop {
                Duration::from_secs(1)
            } else {
                Duration::from_secs(60)
            };
        let request = WorthQueryRequestScope::new(Some(deadline), cancellation.token());
        let entered = native_entries();
        let visits = Rc::new(Cell::new(0));
        let observed = Rc::clone(&visits);
        CHECKPOINT.with(|slot| {
            *slot.borrow_mut() = Some(Box::new(move |subject| {
                // Definition validation precedes acquisition. Stop only after the
                // genuine Native component reader was entered, at its own control.
                if subject != "Shells" || native_entries() == entered {
                    return;
                }
                observed.set(observed.get() + 1);
                if observed.get() == 1 {
                    if deadline_stop {
                        std::thread::sleep(
                            deadline.saturating_duration_since(Instant::now())
                                + Duration::from_millis(1),
                        );
                    } else {
                        cancellation.cancel();
                    }
                }
            }))
        });
        let _checkpoint = CheckpointGuard;
        let host = crate::test_request_policy::serial_request();
        let result = read::execute(
            &world,
            &worth_execution::ExecutionRequest::serial(&host),
            &request,
            Of::reference(),
            0,
        );
        assert!(
            visits.get() > 0,
            "the Native component callback must execute"
        );
        let expected = if deadline_stop {
            WorthQueryApplicationOneShotDenialKind::DeadlineExceeded
        } else {
            WorthQueryApplicationOneShotDenialKind::Cancelled
        };
        assert!(
            matches!(result,Err(read::Stop::Read(ref stop)) if stop.kind()==expected),
            "original typed Query stop must survive Native and outer execution"
        );
    }
}
