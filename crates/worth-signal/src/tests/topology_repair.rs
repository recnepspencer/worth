use crate::facade::*;
use crate::tests::support::*;

mod checkpoint_cause_restore;

#[test]
fn observation_reads_stay_pure_while_runtime_topology_reads_prune_stale_edges() {
    let mut graph = SignalGraph::new();
    let upstream = graph.node().build();
    let downstream = graph.node().build();

    graph
        .append_dependency(downstream, upstream, ASPECT_A)
        .unwrap();
    graph.unregister_node(upstream).unwrap();
    let stale_edge = graph.build_dependency_edge(upstream, ASPECT_A, None);
    graph
        .set_dependency_edges_sorted(downstream, &[stale_edge])
        .unwrap();

    let observed_dependencies = graph.dependencies_of(downstream).unwrap();
    assert_eq!(observed_dependencies.len(), 1);
    assert_eq!(observed_dependencies[0].source(), upstream);

    let runtime_dependencies = graph.runtime_dependencies_of(downstream).unwrap();
    assert!(runtime_dependencies.is_empty());
    assert!(graph.dependencies_of(downstream).unwrap().is_empty());
}

#[test]
fn runtime_dead_edge_repair_advances_revision_and_invalidates_causes() {
    use crate::data::proof::invalidation::binding::{OutputCommitOrdinal, ResolvedDependencyCause};

    let mut graph = SignalGraph::new();
    let upstream = graph.node().build();
    let downstream = graph.node().build();
    graph
        .set_dependencies(downstream, [DependencyEdge::new(upstream, ASPECT_A)])
        .unwrap();
    let revision = graph.dependency_revision(downstream).unwrap();
    graph
        .inject_pending_causes_unchecked_for_test(
            downstream,
            [ResolvedDependencyCause::new(
                graph.runtime_instance_id(),
                downstream,
                revision,
                upstream,
                ASPECT_A,
                None,
                0,
                OutputCommitOrdinal(1),
                1,
                PartitionScopeSet::default(),
            )],
        )
        .unwrap();
    graph.unregister_node(upstream).unwrap();
    graph
        .inject_retired_dependency_for_test(downstream, upstream, ASPECT_A)
        .unwrap();
    let revision = graph.dependency_revision(downstream).unwrap();
    graph
        .inject_pending_causes_unchecked_for_test(
            downstream,
            [ResolvedDependencyCause::new(
                graph.runtime_instance_id(),
                downstream,
                revision,
                upstream,
                ASPECT_A,
                None,
                0,
                OutputCommitOrdinal(2),
                2,
                PartitionScopeSet::default(),
            )],
        )
        .unwrap();

    assert!(graph
        .runtime_dependencies_of(downstream)
        .unwrap()
        .is_empty());
    assert_eq!(
        graph.dependency_revision(downstream).unwrap().0,
        revision.0 + 1
    );
    assert!(graph.pending_causes(downstream).unwrap().is_empty());
    assert_eq!(graph.get_state(downstream).unwrap(), NodeState::MaybeStale);
    assert!(graph.node_dirty_aspects(downstream).unwrap().is_empty());
}

#[test]
fn retirement_severs_dead_edges_before_gc_resets_tombstone_pressure() {
    let mut graph = SignalGraph::with_gc_threshold(1);
    let upstream = graph.node().build();
    let downstream = graph.node().build();
    graph
        .set_dependencies(downstream, [DependencyEdge::new(upstream, ASPECT_A)])
        .unwrap();
    graph.unregister_node(upstream).unwrap();
    assert!(graph.dependencies_of(downstream).unwrap().is_empty());
    graph.run_gc_epoch();

    assert!(graph
        .runtime_dependencies_of(downstream)
        .unwrap()
        .is_empty());
}

#[test]
fn retirement_uses_structural_revalidation_without_forging_aspect_dirtiness() {
    let mut graph = SignalGraph::new();
    let upstream = graph.node().build();
    let downstream = graph.node().build();
    graph
        .set_dependencies(downstream, [DependencyEdge::new(upstream, ASPECT_A)])
        .unwrap();
    let mut compute = |_id, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut graph, upstream, &mut compute).unwrap();
    evaluate(&mut graph, downstream, &mut compute).unwrap();
    let revision = graph.dependency_revision(downstream).unwrap();

    graph.unregister_node(upstream).unwrap();

    assert_eq!(graph.get_state(downstream).unwrap(), NodeState::MaybeStale);
    assert_eq!(
        graph.dependency_revision(downstream).unwrap().0,
        revision.0 + 1
    );
    assert!(graph.pending_causes(downstream).unwrap().is_empty());
    assert!(graph.node_dirty_aspects(downstream).unwrap().is_empty());
    assert!(graph
        .node_dirty_scoped_aspects(downstream)
        .unwrap()
        .is_empty());
}

#[test]
fn retirement_preserves_an_unevaluated_consumers_first_compute_obligation() {
    let mut graph = SignalGraph::new();
    let upstream = graph.node().build();
    let downstream = graph.node().build();
    graph
        .set_dependencies(downstream, [DependencyEdge::new(upstream, ASPECT_A)])
        .unwrap();

    graph.unregister_node(upstream).unwrap();

    assert_eq!(graph.get_state(downstream).unwrap(), NodeState::Dirty);
    let mut evaluations = 0_u32;
    evaluate(&mut graph, downstream, &mut |_id, _graph| {
        evaluations += 1;
        Ok(version_ab(1, 0))
    })
    .unwrap();
    assert_eq!(evaluations, 1);
}

#[test]
fn checkpoint_restore_repairs_serialized_dead_topology_before_runtime_reads() {
    let mut graph = SignalGraph::new();
    let retired_source = graph.node().build();
    let consumer = graph.node().build();
    let live_source = graph.node().build();
    let retired_subscriber = graph.node().build();
    graph.unregister_node(retired_source).unwrap();
    graph.unregister_node(retired_subscriber).unwrap();
    graph
        .inject_retired_dependency_for_test(consumer, retired_source, ASPECT_A)
        .unwrap();
    graph
        .inject_retired_subscriber_for_test(live_source, retired_subscriber)
        .unwrap();

    let authority = graph.capture_checkpoint_authority();
    let mut restored = SignalGraph::restore_from_checkpoint_authority(&authority).unwrap();

    assert_eq!(restored.get_state(consumer).unwrap(), NodeState::Dirty);
    assert!(restored.dependencies_of(consumer).unwrap().is_empty());
    assert!(restored.subscribers_of(live_source).unwrap().is_empty());
    assert!(restored
        .runtime_dependencies_of(consumer)
        .unwrap()
        .is_empty());
    assert!(restored
        .runtime_subscribers_of(live_source)
        .unwrap()
        .is_empty());
    restored.assert_bidirectional_consistency().unwrap();
    let mut evaluations = 0_u32;
    evaluate(&mut restored, consumer, &mut |_id, _graph| {
        evaluations += 1;
        Ok(version_ab(1, 0))
    })
    .unwrap();
    assert_eq!(evaluations, 1);
}

#[test]
fn checkpoint_repair_recomputes_an_evaluated_consumer_after_snapshot_discard() {
    let mut graph = SignalGraph::new();
    let retired_source = graph.node().build();
    let consumer = graph.node().build();
    let mut baseline = |_id, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut graph, consumer, &mut baseline).unwrap();
    graph.unregister_node(retired_source).unwrap();
    graph
        .inject_retired_dependency_for_test(consumer, retired_source, ASPECT_A)
        .unwrap();

    let authority = graph.capture_checkpoint_authority();
    let restored = SignalGraph::restore_from_checkpoint_authority(&authority).unwrap();
    assert_eq!(restored.get_state(consumer).unwrap(), NodeState::MaybeStale);
    let pending = restored
        .pending_dependency_revalidation(consumer)
        .unwrap()
        .expect("checkpoint topology repair must retain structural revalidation");
    assert!(pending.is_resolved());

    let evaluations = std::sync::atomic::AtomicU32::new(0);
    let mut runtime = SignalRuntime::builder(restored)
        .with_kernel_defaults()
        .build();
    runtime
        .read(consumer, &(), &|view| {
            evaluations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok(view.finish(version_ab(2, 0)))
        })
        .unwrap();
    assert_eq!(evaluations.load(std::sync::atomic::Ordering::Relaxed), 1);
}

#[test]
fn checkpoint_repair_schedules_retained_dirty_producer_before_structural_recompute() {
    let mut graph = SignalGraph::new();
    let live_source = graph.node().build();
    let consumer = graph.node().build();
    let retired_source = graph.node().build();
    graph
        .set_dependencies(consumer, [DependencyEdge::new(live_source, ASPECT_A)])
        .unwrap();
    let mut baseline = |_id, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut graph, live_source, &mut baseline).unwrap();
    evaluate(&mut graph, consumer, &mut baseline).unwrap();
    graph.unregister_node(retired_source).unwrap();
    mark_dirty(&mut graph, live_source, ASPECT_A).unwrap();
    let live_edge = graph.build_dependency_edge(live_source, ASPECT_A, None);
    let dead_edge = graph.build_dependency_edge(retired_source, ASPECT_A, None);
    graph
        .set_dependency_edges_sorted(consumer, &[live_edge, dead_edge])
        .unwrap();

    let authority = graph.capture_checkpoint_authority();
    let restored = SignalGraph::restore_from_checkpoint_authority(&authority).unwrap();
    let pending = restored
        .pending_dependency_revalidation(consumer)
        .unwrap()
        .expect("checkpoint repair must retain structural authority");
    assert!(pending.requires_structural_recompute());
    assert_eq!(pending.unresolved_producers(), &[live_source]);

    let consumer_evaluations = std::sync::atomic::AtomicU32::new(0);
    let mut runtime = SignalRuntime::builder(restored)
        .with_kernel_defaults()
        .build();
    runtime
        .read(consumer, &(), &|view| {
            if view.node() == consumer {
                consumer_evaluations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            Ok(view.finish(version_ab(2, 0)))
        })
        .unwrap();
    assert_eq!(
        consumer_evaluations.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn direct_dirty_seed_supersedes_dependency_causes_before_topology_repair() {
    use crate::data::proof::invalidation::binding::{OutputCommitOrdinal, ResolvedDependencyCause};

    let mut graph = SignalGraph::new();
    let upstream = graph.node().build();
    let downstream = graph.node().build();
    graph
        .set_dependencies(downstream, [DependencyEdge::new(upstream, ASPECT_A)])
        .unwrap();
    let revision = graph.dependency_revision(downstream).unwrap();
    graph
        .inject_pending_causes_unchecked_for_test(
            downstream,
            [ResolvedDependencyCause::new(
                graph.runtime_instance_id(),
                downstream,
                revision,
                upstream,
                ASPECT_A,
                None,
                0,
                OutputCommitOrdinal(1),
                1,
                PartitionScopeSet::default(),
            )],
        )
        .unwrap();
    graph.set_node_state(downstream, NodeState::Dirty).unwrap();

    mark_dirty(&mut graph, downstream, ASPECT_B).unwrap();
    graph.unregister_node(upstream).unwrap();

    let dirty = graph.node_dirty_aspects(downstream).unwrap();
    assert_eq!(graph.get_state(downstream).unwrap(), NodeState::Dirty);
    assert!(graph.pending_causes(downstream).unwrap().is_empty());
    assert!(dirty.contains(AspectMask::from_aspect(ASPECT_B)));
    assert!(!dirty.contains(AspectMask::from_aspect(ASPECT_A)));
}

#[test]
fn dependency_output_cannot_overwrite_an_active_direct_dirty_obligation() {
    use crate::data::comparator::DefaultComparatorPolicyResolver;
    use crate::data::proof::invalidation::binding::OutputCommitOrdinal;
    use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;

    let mut graph = SignalGraph::new();
    let upstream = graph.node().build();
    let downstream = graph.node().build();
    graph
        .set_dependencies(downstream, [DependencyEdge::new(upstream, ASPECT_A)])
        .unwrap();
    let mut baseline = |_id, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut graph, upstream, &mut baseline).unwrap();
    evaluate(&mut graph, downstream, &mut baseline).unwrap();
    mark_dirty(&mut graph, downstream, ASPECT_B).unwrap();

    let delta = ProducedAspectDelta::from_committed_result(
        upstream,
        OutputCommitOrdinal(1),
        version_ab(1, 0),
        version_ab(2, 0),
        AspectMask::from_aspect(ASPECT_A),
        &[],
        &[],
    )
    .unwrap();
    let prepared = graph
        .prepare_direct_output_causes(
            &delta,
            &mut DefaultComparatorPolicyResolver::default(),
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
        .unwrap();
    let mut work = crate::data::retained_storage::RetainedStoragePreparation::new(10_000);
    let projection = crate::data::graph::PendingRevalidationNodeProjection::capture(
        &graph,
        delta.producer,
        &mut work,
    )
    .unwrap();
    let prepared = graph
        .prepare_direct_cause_publication(prepared, projection, false, &mut work)
        .unwrap();
    graph.publish_direct_output_causes(prepared).unwrap();

    let dirty = graph.node_dirty_aspects(downstream).unwrap();
    assert_eq!(graph.get_state(downstream).unwrap(), NodeState::Dirty);
    assert!(graph.pending_causes(downstream).unwrap().is_empty());
    assert!(dirty.contains(AspectMask::from_aspect(ASPECT_B)));
    assert!(!dirty.contains(AspectMask::from_aspect(ASPECT_A)));
}

#[test]
fn observation_subscriber_reads_stay_pure_while_runtime_reads_prune_stale_subscribers() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let subscriber = graph.node().build();

    graph
        .append_dependency(subscriber, source, ASPECT_A)
        .unwrap();
    graph.unregister_node(subscriber).unwrap();
    graph
        .inject_retired_subscriber_for_test(source, subscriber)
        .unwrap();

    let observed_subscribers = graph.subscribers_of(source).unwrap();
    assert_eq!(observed_subscribers, &[subscriber]);

    let runtime_subscribers = graph.runtime_subscribers_of(source).unwrap();
    assert!(runtime_subscribers.is_empty());
    assert!(graph.subscribers_of(source).unwrap().is_empty());
}
