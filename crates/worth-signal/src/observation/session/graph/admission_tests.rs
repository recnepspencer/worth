use super::*;
use crate::data::telemetry::InvalidationPerformedCounter;
use crate::logic::transaction::SignalObservationSurface;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::Ordering;

#[test]
fn observation_admission_failure_preserves_unpublished_session_state() {
    let mut graph = SignalGraph::new();
    let request = SignalObservationRequest::operation();
    let bindings = graph.invalidation_performed_work.shared_bindings();
    // Explicit capture-storage fault fixture, not synthetic execution evidence.
    let values = graph.invalidation_performed_counters.shared_values();
    values[InvalidationPerformedCounter::NodesEvaluated.index()].store(17, Ordering::Relaxed);
    graph
        .observation_sessions
        .record_completion(SignalObservationCompletion::Cancelled);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _held = bindings.lock().unwrap();
        panic!("injected capture admission failure");
    }))
    .is_err());
    let failure = catch_unwind(AssertUnwindSafe(|| {
        graph.begin_observation_session(request)
    }));
    assert!(failure.is_err());
    assert_eq!(graph.observation_session_active_generation(), 0);
    assert_eq!(
        graph
            .invalidation_performed_counters()
            .value(InvalidationPerformedCounter::NodesEvaluated),
        17
    );
    assert_eq!(
        graph.observation_sessions.last_completion(),
        Some(SignalObservationCompletion::Cancelled)
    );
    assert!(!graph
        .diagnostics_state_mut()
        .has_observation_activation(SignalObservationSurface::PerformedWork.bit()));
    assert!(!graph
        .diagnostics_state_mut()
        .has_observation_activation(SignalObservationSurface::PerformedCounters.bit()));

    // Repair only the injected mutex fault; admission itself never repairs it.
    assert!(bindings.is_poisoned());
    bindings.clear_poison();
    let session = graph.begin_observation_session(request).unwrap();
    assert_eq!(
        session.generation(),
        1,
        "failed preparation published no generation"
    );
    assert_eq!(
        graph
            .invalidation_performed_counters()
            .value(InvalidationPerformedCounter::NodesEvaluated),
        0
    );
    assert!(graph
        .diagnostics_state_mut()
        .has_observation_activation(SignalObservationSurface::PerformedWork.bit()));
    assert!(graph
        .finish_optional_observation_session(&session)
        .unwrap()
        .is_none());
    drop(session);
    assert_eq!(graph.observation_session_active_generation(), 0);
}

#[test]
fn observation_typed_denial_precedes_poisoned_capture_preparation() {
    let mut graph = SignalGraph::new();
    let session = graph
        .begin_observation_session(SignalObservationRequest::counters())
        .unwrap();
    let generation = session.generation();
    let bindings = graph.invalidation_performed_work.shared_bindings();
    // Missing reconstruction metadata must not authorize a rebind on denial.
    graph.observation_capture_cleanup = None;
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _held = bindings.lock().unwrap();
        panic!("injected capture admission failure");
    }))
    .is_err());
    assert!(matches!(
        graph.begin_observation_session(SignalObservationRequest::operation()),
        Err(SignalObservationAdmissionDenial::SessionAlreadyActive)
    ));
    assert_eq!(graph.observation_session_active_generation(), generation);
    assert!(graph.observation_capture_cleanup.is_none());
    assert!(std::sync::Arc::ptr_eq(
        &bindings,
        &graph.invalidation_performed_work.shared_bindings()
    ));
    assert!(graph
        .finish_optional_observation_session(&session)
        .unwrap()
        .is_none());
    drop(session);
    assert_eq!(graph.observation_session_active_generation(), 0);
    assert!(
        bindings.is_poisoned(),
        "unrequested work capture is untouched"
    );
}
