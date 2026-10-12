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
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let observes = std::rc::Rc::new(observes);
    let run = |k: Option<usize>, ceiling: u64| {
        with_door(&|door| {
            let source = CancellationSource::new();
            let cancel = source.clone();
            let host = SerialRequest::from_memory(
                SerialMemoryBudget::new(64 * 1024 * 1024),
                source.token(),
                None,
            );
            let request = ExecutionRequest::serial(&host);
            let count = std::rc::Rc::new(std::cell::Cell::new(0));
            let recorded = std::rc::Rc::clone(&count);
            let observes = std::rc::Rc::clone(&observes);
            let ((answer, report), _) = with_request_consult_observer_for_test(
                move |_| {
                    if observes() {
                        let at = recorded.get() + 1;
                        recorded.set(at);
                        if Some(at) == k {
                            cancel.cancel();
                        }
                    }
                },
                || {
                    request
                        .run(ExecutionWorkCeiling::new(ceiling), |_| door(&request))
                        .unwrap()
                },
            );
            (answer, report.charged_work(), count.get())
        })
    };
    assert_every_consult(run, healthy_work, cancelled, exhausted);
}

/// The same counted oracle for doors borrowing their request from a phase.
pub(crate) fn every_advancement_consult<T: Eq + Debug, E: Eq + Debug>(
    door: impl Fn(
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        u64,
    ) -> Result<T, E>,
    cancelled: E,
    exhausted: E,
) {
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
                64 * 1024 * 1024,
                100_000,
            ),
        ));
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
    };
    assert_every_consult(run, 100_000, cancelled, exhausted);
}

fn assert_every_consult<T: Eq + Debug, E: Eq + Debug>(
    run: impl Fn(Option<usize>, u64) -> (Result<T, E>, u64, usize),
    healthy_work: u64,
    cancelled: E,
    exhausted: E,
) {
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
