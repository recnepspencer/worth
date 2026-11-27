//! Court 14 (g), public #429 B1b: a stop raised while re-resolution walks
//! the family or compares a candidate surfaces at the certifying site as
//! that stop, never as `Moved`, so it schedules no re-execution. The reader
//! recorded `Unique` from the first completion's sealed head, which a
//! healthy certification answers `Unmoved`; the court then stops the
//! certifying request at each of its consults in turn. The certifying
//! request is a controlled child of the host call's own lease, carrying the
//! court's cancellation and deadline.
use super::{install_family, read_family, retained, Fact, FactMovement, Kind};
use std::time::Duration;
use worth_execution::{
    with_request_consult_observer_for_test, CancellationSource, ExecutionRequest, MonotonicTimeController,
};
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::CertificationBasis;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_first_completion;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;

/// A certification's answer, with a stop kept as its debug form.
#[derive(Debug, PartialEq, Eq)]
enum Answer {
    Unmoved,
    Moved,
    Stopped(String),
}

/// How the certifying request is stopped at one consult.
#[derive(Clone, Copy, Debug)]
enum Stop {
    Cancelled,
    DeadlineElapsed,
}

/// Certify `fact` at the selected head's basis with a child of the host
/// call's lease; `stop` ends that child at consult `at`. Returns the answer
/// and the consults the certification made.
fn certify_stopped(
    world: &AuthorizationWorld,
    fact: &Fact,
    stop: Option<(Stop, usize)>,
) -> (Answer, usize) {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let serial = ExecutionRequest::serial(&host_request);
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let source = CancellationSource::new();
    let controller = MonotonicTimeController::new().unwrap();
    let deadline = match stop {
        Some((Stop::DeadlineElapsed, _)) => {
            let now = controller.hold();
            Some(
                controller
                    .clock()
                    .deadline_at(now + Duration::from_millis(1)),
            )
        }
        _ => None,
    };
    let token = source.token();
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let selected = world.selected_product(&host);
            let parent = host.borrowed_request_for_test();
            let child = parent
                .in_scope(|lease| {
                    lease
                        .expect("the installed phase owns a lease")
                        .controlled_child(token, deadline)
                })
                .unwrap();
            let execution = ExecutionRequest::leased(&child);
            graph
                .integration_handle_for_test()
                .with_runtime(&serial, |runtime| {
                    let snapshot = selected.application_basis_for_test().snapshot_handle();
                    let product = selected.product();
                    let basis = CertificationBasis::of(
                        product.lifecycle_incarnation(),
                        product.reference_generation(),
                        snapshot,
                    );
                    with_request_consult_observer_for_test(
                        move |count| match stop {
                            Some((Stop::Cancelled, at)) if count == at => source.cancel(),
                            Some((Stop::DeadlineElapsed, at)) if count == at => {
                                controller.advance(Duration::from_millis(2)).unwrap();
                            }
                            _ => {}
                        },
                        || match fact.source_currentness_at(runtime, &execution, basis) {
                            Ok(Ok(movement)) if movement == FactMovement::from_equal(true) => {
                                Answer::Unmoved
                            }
                            Ok(Ok(_)) => Answer::Moved,
                            Ok(Err(failure)) => Answer::Stopped(format!("{failure:?}")),
                            Err(stop) => Answer::Stopped(format!("{stop:?}")),
                        },
                    )
                })
        })
        .expect("the fixture certification owns its installed host call")
}

#[test]
fn a_stopped_reresolution_surfaces_its_stop() {
    with_first_completion(
        |world| install_family(world, vec![retained()]),
        |world, _receipt, ()| {
            let (kind, facts) = read_family(world, "1");
            assert_eq!(
                kind,
                Kind::Unique,
                "the first completion's head is the family's one member"
            );
            let [fact @ Fact::FamilySelection { .. }] = facts.as_slice() else {
                panic!("the read must record exactly one FamilySelection fact: {facts:?}");
            };
            let (healthy, consults) = certify_stopped(world, fact, None);
            assert_eq!(
                healthy,
                Answer::Unmoved,
                "the healthy certification is Unmoved"
            );
            assert!(consults > 0, "the certification consults its request");
            for stop in [Stop::Cancelled, Stop::DeadlineElapsed] {
                let mut stops = std::collections::BTreeSet::new();
                for at in 0..=consults {
                    match certify_stopped(world, fact, Some((stop, at))).0 {
                        Answer::Moved => panic!(
                            "a {stop:?} certification at consult {at} of {consults} answered Moved"
                        ),
                        Answer::Unmoved => {}
                        Answer::Stopped(stopped) => {
                            stops.insert(stopped);
                        }
                    }
                }
                eprintln!("court 14 (g): {stop:?} over {consults} consults surfaced {stops:?}");
                let named = match stop {
                    Stop::Cancelled => "Cancelled",
                    Stop::DeadlineElapsed => "Deadline",
                };
                assert!(
                    stops.iter().any(|stopped| stopped.starts_with("OutputObservation")
                        && (stopped.contains(named) || stopped.contains("Interrupted"))),
                    "a consult inside the candidate comparison surfaces the request's own {stop:?} stop: {stops:?}"
                );
            }
        },
    );
}
