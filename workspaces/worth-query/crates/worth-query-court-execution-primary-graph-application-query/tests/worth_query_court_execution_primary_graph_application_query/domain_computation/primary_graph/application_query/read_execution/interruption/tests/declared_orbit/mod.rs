//! Original Query stops cross the actual aggregate Native/outer execution handoff.
use crate::domain_computation::primary_graph::application_query::read_execution::interruption::tests::*;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::Orbit;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read;
#[test]
fn original_query_cancellation_and_deadline_survive_native_orbit_entry() {
    let _isolation =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let time = worth_execution::MonotonicTimeController::new().unwrap();
    time.hold();
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::world_on_clock(time.clock());
    for deadline_stop in [false, true] {
        let cancellation = WorthQueryCancellationSource::new();
        let deadline = time.clock().now()
            + if deadline_stop {
                Duration::from_secs(1)
            } else {
                Duration::from_secs(60)
            };
        let request =
            WorthQueryRequestScope::new(Some(deadline), cancellation.token(), time.clock());
        let visits = Rc::new(Cell::new(0));
        let observed = Rc::clone(&visits);
        let advancing_time = time.clone();
        CHECKPOINT.with(|slot| {
            *slot.borrow_mut() = Some(Box::new(move |subject| {
                if subject != "Next" {
                    return;
                }
                let visit = observed.get() + 1;
                observed.set(visit);
                // The first Next safe point belongs to selection. The second is
                // invoked by the original Query control inside Native aggregate read.
                if visit == 2 {
                    if deadline_stop {
                        advancing_time.advance(Duration::from_secs(1)).unwrap();
                    } else {
                        cancellation.cancel();
                    }
                }
            }))
        });
        let _checkpoint = CheckpointGuard;
        let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
        let result = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::execute(
            &world,
            &worth_execution::ExecutionRequest::serial(&host),
            &request,
            Orbit::reference(),
            0,
        );
        assert!(
            visits.get() >= 2,
            "the real Native Query callback must execute"
        );
        let expected = if deadline_stop {
            WorthQueryApplicationOneShotDenialKind::DeadlineExceeded
        } else {
            WorthQueryApplicationOneShotDenialKind::Cancelled
        };
        assert!(
            matches!(result, Err(worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::Stop::Read(ref stop)) if stop.kind() == expected),
            "the first typed Query stop must survive the outer handoff"
        );
    }
}
