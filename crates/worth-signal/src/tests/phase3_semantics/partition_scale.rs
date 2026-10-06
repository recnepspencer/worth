use crate::facade::{
    mark_dirty_with_regions, ChangedRegion, NodeEvaluationResult, NodeId, NodeState,
    PartitionSubscription, SignalGraph, SignalRuntime,
};
use crate::tests::support::{evaluate, version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn transaction_partition_invalidations_union_dirty_scopes_until_runtime_evaluation() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let source = runtime.graph_mut().node().partitioned_output().build();
    let dependent = runtime.graph_mut().node().build();
    runtime
        .graph_mut()
        .append_partition_detail_dependency(dependent, source, ASPECT_A, "wing", "rib-12")
        .unwrap();
    runtime
        .graph_mut()
        .append_partition_detail_dependency(dependent, source, ASPECT_A, "wing", "rib-13")
        .unwrap();

    runtime
        .transaction(&mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12"))
                        .with_changed_region(ChangedRegion::new("wing").with_detail("rib-13")),
                ))
            })?;
            tx.read(dependent, &|view| {
                let _ = view.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::partition_and_detail("wing", "rib-12"),
                )?;
                let _ = view.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::partition_and_detail("wing", "rib-13"),
                )?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(10, 0))))
            })?;
            Ok(())
        })
        .unwrap();
    let dependent_before = runtime.graph().get_state(dependent).unwrap();

    runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty_with_regions(
                source,
                ASPECT_A,
                &[ChangedRegion::new("wing").with_detail("rib-12")],
            )?;
            tx.mark_dirty_with_regions(
                source,
                ASPECT_A,
                &[ChangedRegion::new("wing").with_detail("rib-13")],
            )?;
            Ok(())
        })
        .unwrap();

    let scopes = runtime.graph().node_dirty_scoped_aspects(source).unwrap();
    assert_eq!(runtime.graph().get_state(source).unwrap(), NodeState::Dirty);
    assert_eq!(
        runtime.graph().get_state(dependent).unwrap(),
        dependent_before
    );
    assert!(scopes.iter().any(|(_, scope)| scope
        .path()
        .segments()
        .get(1)
        .is_some_and(|part| part == "rib-12")));
    assert!(scopes.iter().any(|(_, scope)| scope
        .path()
        .segments()
        .get(1)
        .is_some_and(|part| part == "rib-13")));
}

#[test]
fn sparse_partition_fanout_keeps_most_subscribers_out_of_dirty_state() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    let mut source_v1 = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_changed_region(ChangedRegion::new("partition-7")))
    };
    evaluate(&mut graph, source, &mut source_v1).unwrap();

    let mut subscribers = Vec::new();
    for index in 0..128 {
        let subscriber = graph.node().build();
        graph
            .append_partition_dependency(subscriber, source, ASPECT_A, format!("partition-{index}"))
            .unwrap();
        subscribers.push(subscriber);
    }

    let mut source_v2 = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
            .with_changed_region(ChangedRegion::new("partition-7")))
    };
    let mut subscriber_compute = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(10, 0)))
    };

    for &subscriber in &subscribers {
        evaluate(&mut graph, subscriber, &mut subscriber_compute).unwrap();
    }

    mark_dirty_with_regions(
        &mut graph,
        source,
        ASPECT_A,
        &[ChangedRegion::new("partition-7")],
    )
    .unwrap();

    assert!(subscribers
        .iter()
        .all(|&subscriber| graph.get_state(subscriber).unwrap() == NodeState::Clean));

    evaluate(&mut graph, source, &mut source_v2).unwrap();

    let dirty_count = subscribers
        .iter()
        .filter(|&&subscriber| graph.get_state(subscriber).unwrap() == NodeState::Dirty)
        .count();
    let clean_count = subscribers
        .iter()
        .filter(|&&subscriber| graph.get_state(subscriber).unwrap() == NodeState::Clean)
        .count();
    assert_eq!(dirty_count, 1);
    assert_eq!(clean_count, 127);
    assert_eq!(graph.pending_causes(subscribers[7]).unwrap().len(), 1);
}
