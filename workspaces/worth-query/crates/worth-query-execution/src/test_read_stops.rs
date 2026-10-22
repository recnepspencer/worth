//! One counted oracle for actual request-backed Query doors.
use std::fmt::Debug;
use worth_execution::{
    with_request_consult_observer_for_test, CancellationSource, ExecutionRequest,
    ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};
type ConsultRun<T, E> = (Result<T, E>, u64, usize);
type ReadDoor<'a, T, E> = dyn Fn(&ExecutionRequest<'_, '_>) -> Result<T, E> + 'a;
type CountedDoor<'a, T, E> = dyn Fn(&ReadDoor<'_, T, E>) -> ConsultRun<T, E> + 'a;
pub(crate) fn every_consult<T: Eq + Debug, E: Eq + Debug>(
    door: impl Fn(&ExecutionRequest<'_, '_>) -> Result<T, E>,
    cancelled: E,
    exhausted: E,
) {
    every_consult_with_door(100_000, |counted| counted(&door), cancelled, exhausted);
}

/// Builds fresh mutable custody and opens its door before installing the observer.
pub(crate) fn every_consult_with_door<T: Eq + Debug, E: Eq + Debug>(
    healthy_work: u64,
    with_door: impl Fn(&CountedDoor<'_, T, E>) -> ConsultRun<T, E>,
    cancelled: E,
    exhausted: E,
) {
    every_selected_consult_with_door(healthy_work, || true, with_door, cancelled, exhausted);
}

/// Counts only consults within a guarded read-only semantic observation.
pub(crate) fn every_selected_consult_with_door<T: Eq + Debug, E: Eq + Debug>(
    healthy_work: u64,
    observes: impl Fn() -> bool + 'static,
    with_door: impl Fn(&CountedDoor<'_, T, E>) -> ConsultRun<T, E>,
    cancelled: E,
    exhausted: E,
) {
    selected_consult_answers(
        healthy_work,
        observes,
        with_door,
        false,
        |at, _, error| assert_eq!(error, &cancelled, "consult {at}"),
        |required, error| assert_eq!(error, &exhausted, "charge {required}"),
    );
}

/// Full-value oracles can assert which consult answered and every charged prefix.
#[allow(dead_code)] // The Host proof imports this shared oracle owner.
pub(crate) fn selected_consult_answers<T: Eq + Debug, E: Eq + Debug>(
    healthy_work: u64,
    observes: impl Fn() -> bool + 'static,
    with_door: impl Fn(&CountedDoor<'_, T, E>) -> ConsultRun<T, E>,
    all_charges: bool,
    assert_cancelled: impl Fn(usize, usize, &E),
    assert_work: impl Fn(u64, &E),
) {
    selected_consult_answers_under(
        None,
        healthy_work,
        observes,
        with_door,
        all_charges,
        assert_cancelled,
        assert_work,
    );
}

pub(crate) fn every_consult_under<T: Eq + Debug, E: Eq + Debug>(
    parent: &ExecutionRequest<'_, '_>,
    door: impl Fn(&ExecutionRequest<'_, '_>) -> Result<T, E>,
    cancelled: E,
    exhausted: E,
) {
    selected_consult_answers_under(
        Some(parent),
        100_000,
        || true,
        |counted| counted(&door),
        false,
        |at, _, error| assert_eq!(error, &cancelled, "consult {at}"),
        |required, error| assert_eq!(error, &exhausted, "charge {required}"),
    );
}

fn selected_consult_answers_under<T: Eq + Debug, E: Eq + Debug>(
    parent: Option<&ExecutionRequest<'_, '_>>,
    healthy_work: u64,
    observes: impl Fn() -> bool + 'static,
    with_door: impl Fn(&CountedDoor<'_, T, E>) -> ConsultRun<T, E>,
    all_charges: bool,
    assert_cancelled: impl Fn(usize, usize, &E),
    assert_work: impl Fn(u64, &E),
) {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let observes = std::rc::Rc::new(observes);
    let quotes = std::cell::RefCell::new(Vec::new());
    let run = |k: Option<usize>, ceiling: u64| {
        with_door(&|door| {
            let source = CancellationSource::new();
            let cancel = source.clone();
            let host =
                SerialRequest::from_memory(SerialMemoryBudget::new(None), source.token(), None);
            let child = parent.map(|request| {
                request
                    .in_scope(|lease| {
                        let lease = lease.expect("the fixture lends its active host lease");
                        lease
                            .child(worth_execution::LeaseRequest {
                                policy: worth_foundational::ExecutionRequestPolicy::new(
                                    lease.policy().posture(),
                                    lease.policy().determinism(),
                                    worth_foundational::ExecutionBudget::new(
                                        lease.policy().max_workers(),
                                        lease.available_memory_bytes(),
                                        lease.policy().work_ceiling(),
                                    ),
                                ),
                                cancellation: source.token(),
                                deadline: None,
                            })
                            .expect("the consult proof borrows descendant custody")
                    })
                    .expect("the fixture lends its request")
            });
            let request = child
                .as_ref()
                .map_or_else(|| ExecutionRequest::serial(&host), ExecutionRequest::leased);

            let count = std::rc::Rc::new(std::cell::Cell::new(0));
            let recorded = std::rc::Rc::clone(&count);
            let observes = std::rc::Rc::clone(&observes);
            let selected_indices = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
            let selected = std::rc::Rc::clone(&selected_indices);
            let observation = observe_consults_for_test(move |global_at| {
                if observes() {
                    selected.borrow_mut().push(global_at);
                    let at = recorded.get() + 1;
                    recorded.set(at);
                    if Some(at) == k {
                        cancel.cancel();
                    }
                }
            });
            let ((answer, answer_origins), report) = request
                .run(ExecutionWorkCeiling::new(Some(ceiling)), |_| {
                    let answer = door(&request);
                    (answer, observation.stop_origins_for_test())
                })
                .unwrap();
            *quotes.borrow_mut() = charged_prefixes_for_test(&observation);
            let indices = selected_indices.borrow();
            let origins = observation.stop_origins_for_test();
            let answered_at = if let Some(at) = k.filter(|_| answer.is_err()) {
                let injected = indices[at - 1];
                let first = *answer_origins
                    .first()
                    .expect("the door emitted its stopped cause");
                assert!(
                    first.0 >= injected && first.1 >= injected,
                    "the first emission and its origin follow the injection"
                );
                assert!(
                    origins.starts_with(&answer_origins),
                    "no emission before the answer is dropped"
                );
                for (emitted, origin) in &origins {
                    assert!(*origin >= injected, "no stop predates the injection");
                    assert!(*emitted >= *origin, "a re-report follows its origin");
                }
                // The trace retains every origin, including emissions outside
                // the selected window. The returned index is the selected
                // injection plus the global consult distance to first emission.
                // It proves emission order, not the returned value:
                // a stop returned at selected consult 15 and a swallowed result
                // re-reported by the same-meter latch at 16 are indistinguishable.
                at + (first.0 - injected)
            } else {
                count.get()
            };
            (answer, report.charged_work(), answered_at)
        })
    };
    let (healthy, work, count) = run(None, healthy_work);
    assert!(healthy.is_ok(), "healthy door: {healthy:?}");
    assert!(count > 0);
    let charges = quotes.borrow().clone();
    eprintln!(
        "door consults={count}, work={work}, charges={}",
        charges.len()
    );
    for at in 1..=count {
        let (answer, _, answered_at) = run(Some(at), healthy_work);
        assert_cancelled(
            at,
            answered_at,
            &answer.expect_err("the selected consult stopped"),
        );
    }
    assert_eq!(
        run(Some(count + 1), healthy_work).0,
        healthy,
        "one past the last consult"
    );
    let prefixes = if all_charges {
        charges
            .into_iter()
            .map(|(_, required)| required)
            .collect::<std::collections::BTreeSet<_>>()
    } else {
        [work].into_iter().collect()
    };
    for required in prefixes.into_iter().filter(|required| *required > 0) {
        let answer = run(None, required - 1).0;
        assert_work(
            required,
            &answer.expect_err("one unit short at an actual charge"),
        );
    }
}

/// The same counted oracle for doors borrowing their request from a phase.
pub(crate) fn every_advancement_consult<T: Eq + Debug, E: Eq + Debug>(
    door: impl Fn(
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        u64,
    ) -> Result<T, E>,
    cancelled: E,
    exhausted: E,
) -> u64 {
    every_advancement_consult_with_door(|counted| counted(&door), cancelled, exhausted)
}

type AdvancementDoor<'a, T, E> = dyn Fn(
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        u64,
    ) -> Result<T, E>
    + 'a;
type CountedAdvancementDoor<'a, T, E> = dyn Fn(&AdvancementDoor<'_, T, E>) -> ConsultRun<T, E> + 'a;
pub(crate) fn every_advancement_consult_with_door<T: Eq + Debug, E: Eq + Debug>(
    with_door: impl Fn(&CountedAdvancementDoor<'_, T, E>) -> ConsultRun<T, E>,
    cancelled: E,
    exhausted: E,
) -> u64 {
    use crate::facade::primary_graph::{
        advancement_requests_on_this_thread_for_test,
        bound_advancement_requests_on_this_thread_for_test,
        place_managed_computations_on_this_thread_for_test, WorthQueryExecutionObservation,
        WorthQueryExecutionPlacementForTest,
    };
    use worth_query_admission::facade::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestScope,
    };
    let run = |k: Option<usize>, ceiling: u64| {
        let _guard = WorthQueryExecutionObservation::install_on_this_thread();
        place_managed_computations_on_this_thread_for_test(
            WorthQueryExecutionPlacementForTest::Serial,
        );
        bound_advancement_requests_on_this_thread_for_test(Some(
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                None,
                Some(100_000),
            ),
        ));
        with_door(&|door| {
            let source = WorthQueryCancellationSource::new();
            let request = WorthQueryRequestScope::new(None, source.token());
            let (answer, count) = with_request_consult_observer_for_test(
                move |at| {
                    if Some(at) == k {
                        source.cancel();
                    }
                },
                || door(&request, ceiling),
            );
            let reports = advancement_requests_on_this_thread_for_test();
            let work = reports
                .last()
                .expect("door opened its request")
                .as_ref()
                .unwrap()
                .charged_work();
            (answer, work, count)
        })
    };
    assert_every_consult(run, 100_000, cancelled, exhausted)
}

fn assert_every_consult<T: Eq + Debug, E: Eq + Debug>(
    run: impl Fn(Option<usize>, u64) -> (Result<T, E>, u64, usize),
    healthy_work: u64,
    cancelled: E,
    exhausted: E,
) -> u64 {
    let (healthy, work, count) = run(None, healthy_work);
    assert_ne!(
        healthy.as_ref().err(),
        Some(&cancelled),
        "healthy door must not report cancellation"
    );
    assert_ne!(
        healthy.as_ref().err(),
        Some(&exhausted),
        "healthy door must not exhaust work"
    );
    assert!(count > 0);
    eprintln!("door consults={count}, work={work}");
    for k in 1..=count + 1 {
        let (answer, _, _) = run(Some(k), healthy_work);
        if k > count {
            assert_eq!(answer, healthy, "past consult {count}");
        } else {
            assert_eq!(answer.as_ref().err(), Some(&cancelled), "consult {k}");
        }
    }
    if work > 0 {
        assert_eq!(
            run(None, work - 1).0.err(),
            Some(exhausted),
            "one unit short"
        );
    }
    work
}

/// Signals external cancellation inside a named read consult, using the sealed
/// SDK observation door this oracle owner already holds.
pub(crate) fn cancelled_at_consult_for_test<T>(
    at: usize,
    source: worth_execution::CancellationSource,
    read: impl FnOnce() -> T,
) -> T {
    with_request_consult_observer_for_test(
        move |count| {
            if count == at {
                source.cancel();
            }
        },
        read,
    )
    .0
}

/// Receiver-owned SDK observation guard; only signals the supplied request.
pub(crate) fn observe_consults_for_test(
    observe: impl FnMut(usize) + 'static,
) -> worth_execution::RequestConsultObservationForTest {
    worth_execution::RequestConsultObservationForTest::install_on_this_thread_for_test(observe)
}

pub(crate) fn consult_count_for_test(
    observation: &worth_execution::RequestConsultObservationForTest,
) -> usize {
    observation.count_for_test()
}

#[allow(dead_code)] // The Host clock proof imports this shared harness.
pub(crate) fn charged_prefixes_for_test(
    observation: &worth_execution::RequestConsultObservationForTest,
) -> Vec<(usize, u64)> {
    observation.charged_prefixes_for_test()
}
