use crate::facade::{
    EvaluationRequestMode, NodeEvaluationResult, SignalError, SignalGraph, SignalRuntime,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A, ASPECT_B};

#[test]
fn repeated_edge_churn_preserves_dependency_and_subscriber_integrity() {
    let mut graph = SignalGraph::new();
    let hub = graph.node().build();
    let leaves: Vec<_> = (0..48).map(|_| graph.node().build()).collect();

    for round in 0..40 {
        for &leaf in &leaves {
            let aspect = if round % 2 == 0 { ASPECT_A } else { ASPECT_B };
            graph.append_dependency(leaf, hub, aspect).unwrap();
            assert!(graph
                .dependencies_of(leaf)
                .unwrap()
                .iter()
                .any(|dependency| dependency.source() == hub && dependency.aspect() == aspect));
        }

        for &leaf in leaves.iter().step_by(3) {
            let aspect = if round % 2 == 0 { ASPECT_A } else { ASPECT_B };
            graph.drop_dependency(leaf, hub, aspect).unwrap();
            graph.append_dependency(leaf, hub, aspect).unwrap();
        }

        let subscribers = graph.subscribers_of(hub).unwrap();
        assert_eq!(subscribers.len(), leaves.len());
        for &leaf in &leaves {
            let deps = graph.dependencies_of(leaf).unwrap();
            assert!(!deps.is_empty());
            assert!(deps.iter().all(|dependency| dependency.source() == hub));
            let distinct_aspects = deps
                .iter()
                .map(|dependency| dependency.aspect().id())
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(distinct_aspects.len(), deps.len());
        }
    }
}

#[test]
fn rollback_after_dynamic_dependency_churn_restores_original_dependencies() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let source_a = runtime.graph_mut().node().build();
    let source_b = runtime.graph_mut().node().build();
    let dependent = runtime.graph_mut().node().build();
    let mut ctx = ();

    runtime
        .transaction(request_execution, &mut ctx, |tx| {
            tx.evaluate_with_plan(
                dependent,
                &|view| {
                    let result = if view.node() == source_a {
                        crate::logic::evaluation::EvaluationOutput::from_result(version_ab(1, 0))
                    } else if view.node() == source_b {
                        crate::logic::evaluation::EvaluationOutput::from_result(version_ab(2, 0))
                    } else {
                        let version = view.read_aspect_version(source_a, ASPECT_A)?;
                        view.finish(version)
                    };
                    Ok(result)
                },
                EvaluationRequestMode::ForceOnDemand,
            )?;
            Ok(())
        })
        .unwrap();

    let err = runtime.transaction(request_execution, &mut ctx, |tx| {
        tx.mark_dirty(source_b, ASPECT_A)?;
        tx.evaluate_with_plan(
            dependent,
            &|view| {
                let result = if view.node() == source_a {
                    crate::logic::evaluation::EvaluationOutput::from_result(version_ab(1, 0))
                } else if view.node() == source_b {
                    crate::logic::evaluation::EvaluationOutput::from_result(version_ab(3, 0))
                } else {
                    let version = view.read_aspect_version(source_b, ASPECT_A)?;
                    view.finish(version)
                };
                Ok(result)
            },
            EvaluationRequestMode::Default,
        )?;
        Err(SignalError::invalid_input(
            "force rollback after dependency churn",
        ))
    });
    assert!(err.is_err());

    let dependencies = runtime.graph().dependencies_of(dependent).unwrap();
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].source(), source_a);
    let explanation = runtime.observe().explain(dependent).unwrap();
    assert!(explanation
        .upstream
        .iter()
        .all(|cause| format!("{cause:?}").contains(&source_a.to_string())
            || !format!("{cause:?}").contains(&source_b.to_string())));
}

#[test]
fn unregister_and_slot_reuse_after_churn_leave_no_ghost_edges() {
    let mut graph = SignalGraph::new();
    let upstream = graph.node().build();
    let middle = graph.node().build();
    let downstreams: Vec<_> = (0..24).map(|_| graph.node().build()).collect();

    graph.append_dependency(middle, upstream, ASPECT_A).unwrap();
    for &downstream in &downstreams {
        graph
            .append_dependency(downstream, middle, ASPECT_B)
            .unwrap();
    }

    for &downstream in downstreams.iter().step_by(2) {
        graph.drop_dependency(downstream, middle, ASPECT_B).unwrap();
        graph
            .append_dependency(downstream, middle, ASPECT_B)
            .unwrap();
    }

    graph.unregister_node(middle).unwrap();
    let replacement = graph.node().build();

    assert!(graph.subscribers_of(upstream).unwrap().is_empty());
    for &downstream in &downstreams {
        assert!(
            graph
                .runtime_dependencies_of(downstream)
                .unwrap()
                .is_empty(),
            "runtime cleanup should clear stale edges left behind by retirement"
        );
    }
    assert_ne!(replacement, middle);
}

#[test]
fn snapshot_churn_reorders_dependencies_without_ghost_snapshots() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let a = runtime.graph_mut().node().build();
    let b = runtime.graph_mut().node().build();
    let dependent = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(b, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(2, 0))))
            })?;
            tx.evaluate_with_plan(
                dependent,
                &|view| {
                    let result = if view.node() == a {
                        view.finish(version_ab(1, 0))
                    } else if view.node() == b {
                        view.finish(version_ab(2, 0))
                    } else {
                        let version = view.read_aspect_version(a, ASPECT_A)?;
                        view.finish(NodeEvaluationResult::from_version(version))
                    };
                    Ok(result)
                },
                EvaluationRequestMode::ForceOnDemand,
            )?;
            Ok(())
        })
        .unwrap();

    assert_eq!(
        runtime
            .graph()
            .get_dep_snapshot(dependent)
            .unwrap()
            .entries()
            .len(),
        1
    );

    runtime
        .graph_mut()
        .drop_dependency(dependent, a, ASPECT_A)
        .unwrap();
    runtime
        .graph_mut()
        .append_dependency(dependent, b, ASPECT_A)
        .unwrap();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(b, ASPECT_A)?;
            tx.read(b, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(3, 0))))
            })?;
            Ok(())
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.evaluate_with_plan(
                dependent,
                &|view| {
                    let version = view.read_aspect_version(b, ASPECT_A)?;
                    let result = view.finish(NodeEvaluationResult::from_version(version));
                    Ok(result)
                },
                EvaluationRequestMode::ForceOnDemand,
            )?;
            Ok(())
        })
        .unwrap();

    let snapshot = runtime.graph().get_dep_snapshot(dependent).unwrap();
    assert_eq!(snapshot.entries().len(), 1);
    assert_eq!(snapshot.entries()[0].source, b);
    assert_eq!(snapshot.entries()[0].aspect, ASPECT_A);
    assert_eq!(snapshot.entries()[0].cached_version, 3);
    assert!(snapshot.entries()[0].scope.is_none());
    let operational_dependencies = runtime.graph().dependencies_of(dependent).unwrap();
    assert_eq!(operational_dependencies.len(), 1);
    assert_eq!(operational_dependencies[0].source(), b);
}
