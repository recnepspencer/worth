//! Original Query stops cross the actual aggregate Native/outer execution handoff.
use super::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::{self, declarations::Orbit, read};
#[test]
fn original_query_cancellation_and_deadline_survive_native_orbit_entry() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = declared_queries::world();
    for deadline_stop in [false, true] {
        let cancellation = WorthQueryCancellationSource::new();
        let deadline = Instant::now()
            + if deadline_stop {
                Duration::from_secs(1)
            } else {
                Duration::from_secs(60)
            };
        let request = WorthQueryRequestScope::new(Some(deadline), cancellation.token());
        let visits = Rc::new(Cell::new(0));
        let observed = Rc::clone(&visits);
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
            matches!(result, Err(read::Stop::Read(ref stop)) if stop.kind() == expected),
            "the first typed Query stop must survive the outer handoff"
        );
    }
}
