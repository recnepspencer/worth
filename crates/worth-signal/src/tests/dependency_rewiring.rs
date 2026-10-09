use crate::facade::*;
use crate::logic::prepared::{PreparedDependencyCapture, PreparedEvaluation};
use crate::tests::support::*;

#[test]
fn rollback_after_dependency_rewiring_restores_original_topology() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let source_a = graph.node().build();
    let source_b = graph.node().build();
    let dependent = graph.node().build();
    graph
        .append_dependency(dependent, source_a, ASPECT_A)
        .unwrap();

    let before = graph.dependencies_of(dependent).unwrap().to_vec();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let mut ctx = ();

    let err = runtime
        .transaction(request_execution, &mut ctx, |_tx| {
            Err(SignalError::invalid_input("force rollback"))
        })
        .unwrap_err();
    assert!(format!("{err}").contains("force rollback"));

    let after = runtime.graph().dependencies_of(dependent).unwrap().to_vec();
    assert_eq!(before, after);
    assert!(!after.iter().any(|edge| edge.source() == source_b));
    runtime.graph().assert_bidirectional_consistency().unwrap();
}

#[test]
fn same_stage_dependency_rewiring_updates_dependency_edges_and_subscriber_sets_together() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::development());
    let source_a = graph.node().build();
    let source_b = graph.node().build();
    let source_c = graph.node().build();
    let left = graph.node().build();
    let right = graph.node().build();

    let bootstrap = graph
        .build_evaluation_plan(&[left, right], EvaluationRequestMode::Default)
        .unwrap();
    graph
        .execute_prepared_plan_with_precompute(&bootstrap, &|node, _view| {
            let mut capture = PreparedDependencyCapture::new();
            match node {
                n if n == left => capture.record(source_a, ASPECT_A, None),
                n if n == right => capture.record(source_b, ASPECT_A, None),
                _ => {}
            }
            Ok(
                PreparedEvaluation::from_result(NodeEvaluationResult::from_version(version_ab(
                    1, 0,
                )))
                .with_dependencies(capture),
            )
        })
        .unwrap();
    let metrics_before = graph.observe().metrics();
    let rewiring_apply_count_before = metrics_before.execution.rewiring_apply_count;

    mark_dirty(&mut graph, left, ASPECT_A).unwrap();
    mark_dirty(&mut graph, right, ASPECT_A).unwrap();
    let rewire = graph
        .build_evaluation_plan(&[left, right], EvaluationRequestMode::Default)
        .unwrap();
    let report = graph
        .execute_prepared_plan_with_precompute(&rewire, &|node, _view| {
            let mut capture = PreparedDependencyCapture::new();
            match node {
                n if n == left => capture.record(source_c, ASPECT_A, None),
                n if n == right => capture.record(source_a, ASPECT_A, None),
                _ => {}
            }
            Ok(
                PreparedEvaluation::from_result(NodeEvaluationResult::from_version(version_ab(
                    2, 0,
                )))
                .with_dependencies(capture),
            )
        })
        .unwrap();

    let left_deps = graph.dependencies_of(left).unwrap().to_vec();
    let right_deps = graph.dependencies_of(right).unwrap().to_vec();
    let a_subs = graph.subscribers_of(source_a).unwrap().to_vec();
    let b_subs = graph.subscribers_of(source_b).unwrap().to_vec();
    let c_subs = graph.subscribers_of(source_c).unwrap().to_vec();

    assert_eq!(left_deps.len(), 1);
    assert_eq!(left_deps[0].source(), source_c);
    assert_eq!(right_deps.len(), 1);
    assert_eq!(right_deps[0].source(), source_a);
    assert_eq!(a_subs, vec![right]);
    assert!(b_subs.is_empty());
    assert_eq!(c_subs, vec![left]);
    assert!(!report.stages.is_empty());
    assert!(report
        .stages
        .iter()
        .filter(|stage| stage.task_records.len() > 1)
        .flat_map(|stage| stage.task_records.windows(2))
        .all(|pair| pair[0].id.0 < pair[1].id.0));
    assert!(report
        .stages
        .iter()
        .filter(|stage| matches!(
            stage.outcome,
            crate::logic::planner::StageExecutionOutcome::CompletedSerial
        ))
        .all(|stage| stage.semantic_segment_count == stage.task_records.len() as u32));
    assert!(report
        .stages
        .iter()
        .filter(|stage| matches!(
            stage.outcome,
            crate::logic::planner::StageExecutionOutcome::CompletedSerial
        ))
        .all(|stage| {
            let mut ids = stage
                .task_records
                .iter()
                .map(|record| record.semantic_segment_id.0)
                .collect::<Vec<_>>();
            ids.sort_unstable();
            ids.dedup();
            ids.len() == stage.semantic_segment_count as usize
        }));
    assert_eq!(
        graph.observe().metrics().execution.rewiring_apply_count - rewiring_apply_count_before,
        2
    );
    let left_explanation = graph.observe().explain(left).unwrap();
    assert!(left_explanation.rewiring.is_some());
    let left_provenance = graph
        .observe()
        .materialize()
        .reconstruct_provenance_artifact(left)
        .unwrap();
    assert!(left_provenance.rewiring.is_some());
    assert!(left_provenance
        .causal_links
        .iter()
        .any(|link| matches!(link.disposition, CausalDisposition::Topology)));
    graph.assert_bidirectional_consistency().unwrap();
}
