use super::runtime_world::{build_runtime, Ev};
use crate::facade::{
    mark_dirty_with_regions, ChangedRegion, CheckpointBarrier, DiagnosticsTier,
    EvaluationRequestMode, EventEpochOutcome, NodeEvaluationResult, PartitionSubscription,
    SignalGraph, SignalRuntimePolicy,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn flow_diagnostics_attach_event_epochs_after_successful_commit() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().build();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();
    let mut runtime = build_runtime(graph);

    runtime
        .transaction(&mut (), |tx| {
            tx.evaluate_with_plan(
                source,
                &|view| Ok(view.finish(version_ab(1, 0))),
                EvaluationRequestMode::Default,
            )?;
            tx.evaluate_with_plan(
                dependent,
                &|view| {
                    let version = view.read_aspect_version(source, ASPECT_A)?;
                    Ok(view.finish(NodeEvaluationResult::from_version(version)))
                },
                EvaluationRequestMode::Default,
            )?;
            tx.emit_event(Ev::Tick);
            tx.flush_events(CheckpointBarrier::PerOperation)?;
            Ok(())
        })
        .unwrap();

    let flow = runtime.observe().latest_flow_diagnostics().unwrap();
    assert_eq!(flow.event_epochs.len(), 1);
    assert_eq!(flow.event_epochs[0].outcome, EventEpochOutcome::Committed);
    assert_eq!(
        flow.event_epochs[0].barrier,
        CheckpointBarrier::PerOperation
    );
    assert_eq!(flow.event_epochs[0].committed_subscriber_count, 0);
    assert_eq!(flow.event_epochs[0].failed_subscriber_position, None);
}

#[test]
fn fillet_style_explanation_stays_local_to_the_changed_partition_scope() {
    let mut graph = SignalGraph::new();
    let feature_edit = graph.node().partitioned_output().build();
    let unrelated_region = graph.node().partitioned_output().build();
    let fillet = graph.node().build();
    graph
        .append_partition_detail_dependency(
            fillet,
            feature_edit,
            ASPECT_A,
            "surface",
            "fillet-band",
        )
        .unwrap();

    let bootstrap = graph
        .build_evaluation_plan(
            &[feature_edit, unrelated_region, fillet],
            EvaluationRequestMode::ForceOnDemand,
        )
        .unwrap();
    graph
        .execute_prepared_plan_with_precompute(&bootstrap, &|node, view| {
            let result = if node == fillet {
                let version = view.read_partitioned_aspect_version(
                    feature_edit,
                    ASPECT_A,
                    PartitionSubscription::partition_and_detail("surface", "fillet-band"),
                )?;
                view.finish(NodeEvaluationResult::from_version(version))
            } else {
                view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))
            };
            Ok(result)
        })
        .unwrap();

    mark_dirty_with_regions(
        &mut graph,
        feature_edit,
        ASPECT_A,
        &[ChangedRegion::new("surface").with_detail("fillet-band")],
    )
    .unwrap();
    let feature_update = graph
        .build_evaluation_plan(&[feature_edit], EvaluationRequestMode::Default)
        .unwrap();
    graph
        .execute_prepared_plan_with_precompute(&feature_update, &|_node, view| {
            Ok(view.finish(
                NodeEvaluationResult::from_version(version_ab(2, 0))
                    .with_changed_region(ChangedRegion::new("surface").with_detail("fillet-band")),
            ))
        })
        .unwrap();

    let explanation = graph.observe().explain(fillet).unwrap();
    let summary = explanation.diagnostics_summary(DiagnosticsTier::Development);
    assert!(explanation.causal_links.iter().any(|link| {
        link.source == Some(feature_edit)
            && link
                .scope
                .validation_scope
                .as_ref()
                .is_some_and(|scope| scope.path().segments() == ["surface", "fillet-band"])
    }));
    assert!(!explanation
        .causal_links
        .iter()
        .any(|link| link.source == Some(unrelated_region)));
    assert!(summary.triage_classes.contains(&"locality".to_string()));
    assert_eq!(summary.discarded_scope_count, 0);
    assert!(summary
        .scope_provenance_kinds
        .iter()
        .any(|kind| kind == "Direct"));
}

#[test]
fn flow_cause_samples_surface_locality_triage_without_false_rewiring() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    let fillet = graph.node().build();
    graph
        .append_partition_detail_dependency(fillet, source, ASPECT_A, "surface", "fillet-band")
        .unwrap();
    let mut runtime = build_runtime(graph);
    runtime.set_runtime_policy(SignalRuntimePolicy::development());

    runtime
        .transaction(&mut (), |tx| {
            tx.evaluate_with_plan(
                source,
                &|view| Ok(view.finish(version_ab(1, 0))),
                EvaluationRequestMode::Default,
            )?;
            tx.evaluate_with_plan(
                fillet,
                &|view| {
                    let version = view.read_partitioned_aspect_version(
                        source,
                        ASPECT_A,
                        PartitionSubscription::partition_and_detail("surface", "fillet-band"),
                    )?;
                    Ok(view.finish(NodeEvaluationResult::from_version(version)))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap();

    let flow = runtime.observe().latest_flow_diagnostics().unwrap();
    assert!(flow.cause_samples.iter().any(|sample| {
        sample.node == fillet
            && sample.suspect_classes.contains(&"locality".to_string())
            && !sample.suspect_classes.contains(&"rewiring".to_string())
            && sample.scope_kinds.iter().any(|kind| kind == "Direct")
    }));
}

#[test]
fn operational_flow_diagnostics_do_not_sample_explanations_by_default() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    let fillet = graph.node().build();
    graph
        .append_partition_detail_dependency(fillet, source, ASPECT_A, "surface", "fillet-band")
        .unwrap();
    let mut runtime = build_runtime(graph);
    runtime.set_runtime_policy(SignalRuntimePolicy::operational());
    let observation = runtime
        .begin_observation_session(crate::facade::SignalObservationRequest::telemetry())
        .unwrap();

    runtime
        .transaction(&mut (), |tx| {
            tx.evaluate_with_plan(
                source,
                &|view| Ok(view.finish(version_ab(1, 0))),
                EvaluationRequestMode::Default,
            )?;
            tx.evaluate_with_plan(
                fillet,
                &|view| {
                    let version = view.read_partitioned_aspect_version(
                        source,
                        ASPECT_A,
                        PartitionSubscription::partition_and_detail("surface", "fillet-band"),
                    )?;
                    Ok(view.finish(NodeEvaluationResult::from_version(version)))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap();
    runtime.finish_observation_session(&observation).unwrap();

    let flow = runtime.observe().latest_flow_diagnostics().unwrap();
    assert!(
        flow.cause_samples.is_empty(),
        "operational flow diagnostics should not pay sampled explanation cost by default"
    );
}
