use super::super::runtime_world::build_runtime;
use super::world::{CommittedObservationRecord, Phase3RecordingObservationListener};
use crate::facade::{
    AuthorityPolicy, EvaluationRequestMode, NodeEvaluationResult, ObservationPolicy,
};
use crate::tests::support::{version_ab, ASPECT_A};
use std::sync::{Arc, Mutex};

#[test]
fn observation_phase4_diagnostics_surface_exposes_latest_boundary_summary() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let source = graph
        .node()
        .authority_policy(AuthorityPolicy::AuthoritativeOnly)
        .build();
    let mut runtime = build_runtime(graph);
    let calls = Arc::new(Mutex::new(Vec::<CommittedObservationRecord>::new()));

    let handle = runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [source],
        Box::new(Phase3RecordingObservationListener {
            calls: Arc::clone(&calls),
        }),
    );

    let mut ctx = ();
    let mut tx = runtime.begin(&mut ctx);
    tx.evaluate_with_plan(
        source,
        &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))),
        EvaluationRequestMode::Default,
    )
    .unwrap();

    let result = tx.commit().unwrap();
    let latest_observation = runtime
        .observe()
        .latest_observation_summary()
        .expect("latest observation summary should be retained");
    assert_eq!(latest_observation.delivered_event_count, 1);
    assert_eq!(latest_observation.boundary_events.len(), 1);
    assert_eq!(
        latest_observation.boundary_events[0].observer_id,
        handle.observer_id()
    );
    assert_eq!(
        latest_observation.boundary_events[0].handle_id,
        handle.handle_id()
    );
    assert_eq!(
        latest_observation.boundary_events[0]
            .matched_nodes
            .iter()
            .collect::<Vec<_>>(),
        vec![source]
    );

    let latest_flow = runtime
        .diagnostics()
        .latest_flow()
        .expect("flow diagnostics should exist after commit");
    let flow_observation = latest_flow
        .observation
        .expect("latest flow should carry observation summary");
    assert_eq!(flow_observation, latest_observation);
    assert_eq!(flow_observation, &result.observation);
}

#[test]
fn observation_unobserve_does_not_resurrect_dead_listener_after_branch_restore_churn() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let source = graph
        .node()
        .authority_policy(AuthorityPolicy::AuthoritativeOnly)
        .build();
    let mut runtime = build_runtime(graph);
    let keep_calls = Arc::new(Mutex::new(Vec::<CommittedObservationRecord>::new()));
    let removed_calls = Arc::new(Mutex::new(Vec::<CommittedObservationRecord>::new()));

    let keep = runtime.observe_nodes(
        ObservationPolicy::touched(),
        [source],
        Box::new(Phase3RecordingObservationListener {
            calls: Arc::clone(&keep_calls),
        }),
    );
    let removed = runtime.observe_nodes(
        ObservationPolicy::touched(),
        [source],
        Box::new(Phase3RecordingObservationListener {
            calls: Arc::clone(&removed_calls),
        }),
    );

    let mut ctx = ();
    runtime
        .transaction(&mut ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            Ok(())
        })
        .unwrap();

    assert_eq!(
        keep_calls
            .lock()
            .expect("keep observation mutex poisoned")
            .as_slice(),
        &[CommittedObservationRecord {
            observer_id: keep.observer_id().get(),
            handle_id: keep.handle_id().get(),
            matched_node_count: 1,
            touched: true,
            recomputed: false,
            meaningful_change: false,
            trigger_matched: true,
        }]
    );
    assert_eq!(
        removed_calls
            .lock()
            .expect("removed observation mutex poisoned")
            .as_slice(),
        &[CommittedObservationRecord {
            observer_id: removed.observer_id().get(),
            handle_id: removed.handle_id().get(),
            matched_node_count: 1,
            touched: true,
            recomputed: false,
            meaningful_change: false,
            trigger_matched: true,
        }]
    );

    assert!(runtime.unobserve(removed));
    assert_eq!(
        runtime
            .matching_observers_for_node(source)
            .iter()
            .map(|id| id.get())
            .collect::<Vec<_>>(),
        vec![keep.observer_id().get()],
        "node index should immediately forget the removed observer"
    );

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-observation").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    runtime.switch_branch(main).unwrap();
    runtime
        .restore_branch_snapshot(feature.clone(), &feature_snapshot)
        .unwrap();
    runtime.switch_branch(feature).unwrap();

    let result = runtime
        .transaction(&mut ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            Ok(())
        })
        .unwrap();

    let keep_recorded = keep_calls
        .lock()
        .expect("keep observation mutex poisoned")
        .clone();
    let removed_recorded = removed_calls
        .lock()
        .expect("removed observation mutex poisoned")
        .clone();
    assert_eq!(
        keep_recorded.len(),
        2,
        "surviving observer should still receive post-restore delivery"
    );
    assert_eq!(
        removed_recorded.len(),
        1,
        "dead observer must not be resurrected by branch/snapshot churn"
    );
    assert_eq!(
        runtime
            .matching_observers_for_node(source)
            .iter()
            .map(|id| id.get())
            .collect::<Vec<_>>(),
        vec![keep.observer_id().get()],
        "node index must remain free of the removed observer after restore churn"
    );
    assert_eq!(result.observation.delivered_event_count, 1);
    assert_eq!(result.observation.boundary_events.len(), 1);
    assert_eq!(
        result.observation.boundary_events[0].observer_id.get(),
        keep.observer_id().get(),
        "committed observation summary must only name the surviving observer"
    );

    let latest_observation = runtime
        .observe()
        .latest_observation_summary()
        .expect("latest observation summary should exist after the committed feature tx");
    assert_eq!(latest_observation.delivered_event_count, 1);
    assert_eq!(latest_observation.boundary_events.len(), 1);
    assert_eq!(
        latest_observation.boundary_events[0].observer_id.get(),
        keep.observer_id().get(),
        "retained diagnostics summary must not claim the removed observer fired"
    );
}
