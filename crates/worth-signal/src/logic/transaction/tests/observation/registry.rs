use super::super::runtime_world::build_runtime;
use super::world::{NoopObservationListener, RecordingObservationListener};
use crate::facade::{DiagnosticsTier, ObservationPolicy};
use std::sync::{Arc, Mutex};

#[test]
fn observation_registry_assigns_deterministic_ids_and_indexes_nodes() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let a = graph.node().build();
    let b = graph.node().build();
    let c = graph.node().build();
    let mut runtime = build_runtime(graph);

    let first = runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [b, a, b],
        Box::new(NoopObservationListener),
    );
    let second = runtime.observe_nodes(
        ObservationPolicy::visited(),
        [c, b],
        Box::new(NoopObservationListener),
    );

    let summary = runtime.observation_summary();
    assert_eq!(summary.active_observer_count, 2);
    assert_eq!(summary.indexed_node_count, 3);
    assert_eq!(
        summary
            .ordered_observer_ids
            .iter()
            .map(|id| id.get())
            .collect::<Vec<_>>(),
        vec![first.observer_id().get(), second.observer_id().get()]
    );

    let matched_b = runtime.matching_observers_for_node(b);
    assert_eq!(
        matched_b.iter().map(|id| id.get()).collect::<Vec<_>>(),
        vec![first.observer_id().get(), second.observer_id().get()]
    );

    let matched_a = runtime.observe().matching_observers_for_node(a);
    assert_eq!(
        matched_a.iter().map(|id| id.get()).collect::<Vec<_>>(),
        vec![first.observer_id().get()]
    );

    let matched_c = runtime.matching_observers_for_node(c);
    assert_eq!(
        matched_c.iter().map(|id| id.get()).collect::<Vec<_>>(),
        vec![second.observer_id().get()]
    );
}

#[test]
fn unobserve_removes_registration_and_cleans_node_indexes() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let a = graph.node().build();
    let b = graph.node().build();
    let mut runtime = build_runtime(graph);

    let keep = runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [a],
        Box::new(NoopObservationListener),
    );
    let remove = runtime.observe_nodes(
        ObservationPolicy::recomputed(),
        [a, b],
        Box::new(NoopObservationListener),
    );

    assert!(runtime.unobserve(remove));
    assert!(
        !runtime.unobserve(remove),
        "stale handles must not unsubscribe again"
    );

    let summary = runtime.observation_summary();
    assert_eq!(summary.active_observer_count, 1);
    assert_eq!(summary.indexed_node_count, 1);
    assert_eq!(
        summary
            .ordered_observer_ids
            .iter()
            .map(|id| id.get())
            .collect::<Vec<_>>(),
        vec![keep.observer_id().get()]
    );
    assert_eq!(
        runtime
            .matching_observers_for_node(a)
            .iter()
            .map(|id| id.get())
            .collect::<Vec<_>>(),
        vec![keep.observer_id().get()]
    );
    assert!(
        runtime.matching_observers_for_node(b).is_empty(),
        "node index should be cleaned when the final observer is removed"
    );
}

#[test]
fn observation_preview_uses_read_only_runtime_context() {
    let mut runtime = build_runtime(crate::data::graph::SignalGraph::new());
    let called = Arc::new(Mutex::new(Vec::<(u64, DiagnosticsTier)>::new()));
    let called_clone = Arc::clone(&called);

    let handle = runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        std::iter::empty(),
        Box::new(RecordingObservationListener {
            calls: called_clone,
        }),
    );

    assert!(
        runtime
            .observations()
            .notify_preview(&runtime, handle.observer_id()),
        "registered observer should be preview-notifiable"
    );
    assert_eq!(
        called
            .lock()
            .expect("observation preview mutex poisoned")
            .as_slice(),
        &[(
            runtime.observe().current_branch().id.0,
            runtime.observe().diagnostics_profile(),
        )]
    );
}
