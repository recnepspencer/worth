use super::super::runtime_world::build_runtime;
use super::world::NoopObservationListener;
use crate::facade::{
    AuthorityPolicy, EvaluationRequestMode, NodeEvaluationResult, ObservationPolicy,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn observation_phase2_stages_candidates_without_dispatch() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let source = graph.node().build();
    let mut runtime = build_runtime(graph);

    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [source],
        Box::new(NoopObservationListener),
    );

    let mut ctx = ();
    let mut tx = runtime.begin(&mut ctx);
    tx.mark_dirty(source, ASPECT_A).unwrap();

    let summary = tx.observation_scratch_summary();
    assert_eq!(summary.staged_candidate_observer_count, 1);
    assert_eq!(summary.staged_candidate_match_count, 1);
    assert_eq!(summary.classified_event_count, 0);
}

#[test]
fn observation_phase2_lowers_recomputed_and_meaningful_change_from_report() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let source = graph
        .node()
        .authority_policy(AuthorityPolicy::AuthoritativeOnly)
        .build();
    let mut runtime = build_runtime(graph);

    let handle = runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [source],
        Box::new(NoopObservationListener),
    );

    let mut ctx = ();
    let mut tx = runtime.begin(&mut ctx);
    tx.evaluate_with_plan(
        source,
        &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))),
        EvaluationRequestMode::Default,
    )
    .unwrap();

    let summary = tx.observation_scratch_summary();
    assert_eq!(summary.staged_candidate_observer_count, 1);
    assert_eq!(summary.classified_event_count, 1);
    assert_eq!(summary.visited_event_count, 1);
    assert_eq!(summary.recomputed_event_count, 1);
    assert_eq!(summary.meaningful_change_event_count, 1);

    let classified = tx.classified_observation_summaries();
    assert_eq!(classified.len(), 1);
    assert_eq!(classified[0].observer_id, handle.observer_id());
    assert_eq!(classified[0].handle_id, handle.handle_id());
    assert_eq!(classified[0].policy, ObservationPolicy::meaningful_change());
    assert!(classified[0].visited);
    assert!(classified[0].recomputed);
    assert!(classified[0].meaningful_change);
    assert!(classified[0].trigger_matched);
    assert_eq!(classified[0].matched_nodes.len(), 1);
    assert_eq!(
        classified[0].matched_nodes.iter().collect::<Vec<_>>(),
        vec![source]
    );
}

#[test]
fn observation_phase2_distinguishes_output_suppressed_from_meaningful_change() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let source = graph
        .node()
        .output_identity()
        .authority_policy(AuthorityPolicy::AuthoritativeOnly)
        .build();
    let mut runtime = build_runtime(graph);

    let mut ctx = ();
    let mut baseline = runtime.begin(&mut ctx);
    baseline
        .evaluate_with_plan(
            source,
            &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("stable-output"),
                ))
            },
            EvaluationRequestMode::Default,
        )
        .unwrap();
    baseline.commit().unwrap();

    let handle = runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [source],
        Box::new(NoopObservationListener),
    );

    let mut tx = runtime.begin(&mut ctx);
    tx.mark_dirty(source, ASPECT_A).unwrap();
    tx.evaluate_with_plan(
        source,
        &|view| {
            Ok(view.finish(
                NodeEvaluationResult::from_version(version_ab(2, 0))
                    .with_output_identity("stable-output"),
            ))
        },
        EvaluationRequestMode::Default,
    )
    .unwrap();

    let summary = tx.observation_scratch_summary();
    assert_eq!(summary.classified_event_count, 1);
    assert_eq!(summary.recomputed_event_count, 1);
    assert_eq!(summary.meaningful_change_event_count, 0);

    let classified = tx.classified_observation_summaries();
    assert_eq!(classified.len(), 1);
    assert_eq!(classified[0].observer_id, handle.observer_id());
    assert_eq!(classified[0].handle_id, handle.handle_id());
    assert_eq!(classified[0].policy, ObservationPolicy::meaningful_change());
    assert!(classified[0].visited);
    assert!(classified[0].recomputed);
    assert!(!classified[0].meaningful_change);
    assert!(!classified[0].trigger_matched);
    assert_eq!(
        classified[0].matched_nodes.iter().collect::<Vec<_>>(),
        vec![source]
    );
}

#[test]
fn observation_phase2_coalesces_multiple_matching_nodes_into_one_classified_event() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let source = graph
        .node()
        .authority_policy(AuthorityPolicy::AuthoritativeOnly)
        .build();
    let derived = graph.node().build();
    graph.append_dependency(derived, source, ASPECT_A).unwrap();
    let mut runtime = build_runtime(graph);

    let handle = runtime.observe_nodes(
        ObservationPolicy::visited(),
        [source, derived],
        Box::new(NoopObservationListener),
    );

    let mut ctx = ();
    let mut tx = runtime.begin(&mut ctx);
    tx.mark_dirty(source, ASPECT_A).unwrap();
    tx.evaluate_dirty(&|view| {
        if view.node() == source {
            Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
        } else {
            Ok(view.finish(NodeEvaluationResult::from_version(version_ab(10, 0))))
        }
    })
    .unwrap();

    let summary = tx.observation_scratch_summary();
    assert_eq!(summary.staged_candidate_observer_count, 1);
    assert_eq!(summary.staged_candidate_match_count, 2);
    assert_eq!(summary.classified_event_count, 1);

    let classified = tx.classified_observation_summaries();
    assert_eq!(classified.len(), 1);
    assert_eq!(classified[0].observer_id, handle.observer_id());
    assert_eq!(classified[0].handle_id, handle.handle_id());
    assert_eq!(classified[0].policy, ObservationPolicy::visited());
    assert_eq!(classified[0].matched_nodes.len(), 2);
    assert!(classified[0].trigger_matched);
    assert_eq!(
        classified[0].matched_nodes.iter().collect::<Vec<_>>(),
        vec![source, derived]
    );
}

#[test]
fn observation_phase2_prepared_plan_execution_stages_and_classifies_observers() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let source = graph
        .node()
        .authority_policy(AuthorityPolicy::AuthoritativeOnly)
        .build();
    let mut runtime = build_runtime(graph);

    let handle = runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [source],
        Box::new(NoopObservationListener),
    );

    let plan = runtime
        .build_evaluation_plan(&[source], EvaluationRequestMode::Default)
        .unwrap();

    let mut ctx = ();
    let mut tx = runtime.begin(&mut ctx);
    tx.execute_prepared_plan(&plan, &|view| {
        Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
    })
    .unwrap();

    let summary = tx.observation_scratch_summary();
    assert_eq!(summary.staged_candidate_observer_count, 1);
    assert_eq!(summary.classified_event_count, 1);
    assert_eq!(summary.meaningful_change_event_count, 1);
    let classified = tx.classified_observation_summaries();
    assert_eq!(classified.len(), 1);
    assert_eq!(classified[0].observer_id, handle.observer_id());
    assert_eq!(classified[0].handle_id, handle.handle_id());
    assert_eq!(
        classified[0].matched_nodes.iter().collect::<Vec<_>>(),
        vec![source]
    );
}

#[test]
fn observation_phase2_telemetry_counters_accumulate_across_transactions() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let source = graph
        .node()
        .authority_policy(AuthorityPolicy::AuthoritativeOnly)
        .build();
    let mut runtime = build_runtime(graph);

    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [source],
        Box::new(NoopObservationListener),
    );

    for version in [1_u64, 2_u64] {
        let mut ctx = ();
        let mut tx = runtime.begin(&mut ctx);
        tx.mark_dirty(source, ASPECT_A).unwrap();
        tx.evaluate_with_plan(
            source,
            &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(version, 0)))),
            EvaluationRequestMode::Default,
        )
        .unwrap();
        tx.commit().unwrap();
    }

    let telemetry = runtime.telemetry().transaction;
    assert_eq!(telemetry.staged_observation_candidate_count, 2);
    assert_eq!(telemetry.classified_observation_count, 2);
    assert!(telemetry.staged_observation_match_count >= 2);
    assert!(telemetry.observation_classification_breadth >= 2);
}
