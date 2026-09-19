use super::*;
use crate::data::comparator::DefaultComparatorPolicyResolver;
use crate::data::node::NodeState;
use crate::data::output::OutputChange;
use crate::facade::{mark_dirty, SignalRuntimePolicy};
use crate::tests::support::{evaluate, version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn self_replacement_is_rejected_by_graph_admission_before_publication() {
    let mut graph = SignalGraph::new();
    let producer = graph.node().produces_aspects(ASPECT_A).build();
    evaluate(&mut graph, producer, &mut |_, _| Ok(version_ab(1, 0))).unwrap();
    let ordinal = graph.cause_sets.output_commit_ordinal_for_test();
    let error = graph
        .set_dependencies(
            producer,
            [crate::data::dependency::DependencyEdge::new(
                producer, ASPECT_A,
            )],
        )
        .unwrap_err();
    assert!(matches!(error, SignalError::CycleDetected { path } if path == [producer, producer]));
    assert!(graph
        .current_runtime_dependencies_of(producer)
        .unwrap()
        .is_empty());
    assert!(graph.pending_causes(producer).unwrap().is_empty());
    assert_eq!(
        graph.node_aspect_version(producer).unwrap(),
        version_ab(1, 0)
    );
    assert_eq!(graph.get_state(producer).unwrap(), NodeState::Clean);
    assert_eq!(graph.cause_sets.output_commit_ordinal_for_test(), ordinal);
}

#[test]
fn direct_consumer_basis_survives_prepared_waiter_publication() {
    let mut graph = SignalGraph::new();
    let producer = graph.node().produces_aspects(ASPECT_A).build();
    let consumer = graph.node().build();
    graph
        .append_dependency(consumer, producer, ASPECT_A)
        .unwrap();
    for node in [producer, consumer] {
        evaluate(&mut graph, node, &mut |_, _| Ok(version_ab(1, 0))).unwrap();
    }
    mark_dirty(&mut graph, producer, ASPECT_A).unwrap();
    mark_dirty(&mut graph, consumer, crate::tests::support::ASPECT_B).unwrap();
    let basis = graph
        .node_direct_invalidation_basis(consumer)
        .unwrap()
        .cloned();
    let mut effect = super::super::tests::test_effect_with_labels(Vec::new());
    effect.operational.node = producer;
    effect.operational.aspect_version = version_ab(2, 0);
    effect.operational.output_change = OutputChange::Replaced;
    graph
        .apply_effect(
            effect,
            OutputEquivalencePolicy::ExactAspectVersion,
            &mut DefaultComparatorPolicyResolver::default(),
            false,
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    assert_eq!(
        graph.node_direct_invalidation_basis(consumer).unwrap(),
        basis.as_ref()
    );
    assert_eq!(graph.get_state(consumer).unwrap(), NodeState::Dirty);
    assert!(graph.pending_causes(consumer).unwrap().is_empty());
    assert_eq!(
        graph.node_dirty_aspects(consumer).unwrap(),
        crate::tests::support::ASPECT_B.into()
    );
}

#[test]
fn exhausted_waiter_preparation_preserves_live_packet_state_and_retry_succeeds() {
    let mut graph = SignalGraph::new();
    let producer = graph.node().produces_aspects(ASPECT_A).build();
    let consumer = graph.node().build();
    graph
        .append_dependency(consumer, producer, ASPECT_A)
        .unwrap();
    for node in [producer, consumer] {
        evaluate(&mut graph, node, &mut |_, _| Ok(version_ab(1, 0))).unwrap();
    }
    mark_dirty(&mut graph, producer, ASPECT_A).unwrap();
    let waiters = graph.topology.pending_revalidation_waiters.clone();
    let pending = graph.node_pending_revalidation(consumer).unwrap().cloned();
    let snapshot = graph.get_dep_snapshot(producer).unwrap().clone();
    let ordinal = graph.cause_sets.output_commit_ordinal_for_test();
    let artifacts = graph.telemetry().storage.hot_write_runtime_artifact_count;
    graph.set_runtime_policy(
        SignalRuntimePolicy::development().with_maximum_waiter_resolution_visits(1),
    );
    let mut effect = super::super::tests::test_effect_with_labels(Vec::new());
    effect.operational.node = producer;
    effect.operational.aspect_version = version_ab(2, 0);
    effect.operational.output_change = OutputChange::Replaced;
    let apply = graph
        .build_apply_commit_packet(
            effect.clone(),
            OutputEquivalencePolicy::ExactAspectVersion,
            false,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
        .unwrap();
    let error = graph
        .prepare_output_commit_packet(
            apply,
            &mut DefaultComparatorPolicyResolver::default(),
            &mut EvaluationWork::Ordinary,
        )
        .unwrap_err();
    assert_eq!(
        error,
        SignalError::WaiterResolutionWorkExhausted { maximum_visits: 1 }
    );
    assert_eq!(
        graph.node_aspect_version(producer).unwrap(),
        version_ab(1, 0)
    );
    assert_eq!(graph.get_state(producer).unwrap(), NodeState::Dirty);
    assert_eq!(graph.topology.pending_revalidation_waiters, waiters);
    assert_eq!(
        graph.node_pending_revalidation(consumer).unwrap(),
        pending.as_ref()
    );
    assert_eq!(graph.get_dep_snapshot(producer).unwrap(), &snapshot);
    assert!(graph.pending_causes(consumer).unwrap().is_empty());
    assert_eq!(graph.cause_sets.output_commit_ordinal_for_test(), ordinal);
    assert_eq!(
        graph.telemetry().storage.hot_write_runtime_artifact_count,
        artifacts
    );
    graph.set_runtime_policy(
        SignalRuntimePolicy::development().with_maximum_waiter_resolution_visits(10_000),
    );
    graph
        .apply_effect(
            effect,
            OutputEquivalencePolicy::ExactAspectVersion,
            &mut DefaultComparatorPolicyResolver::default(),
            false,
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    assert_eq!(
        graph.node_aspect_version(producer).unwrap(),
        version_ab(2, 0)
    );
    assert_eq!(graph.get_state(consumer).unwrap(), NodeState::Dirty);
    assert_eq!(graph.pending_causes(consumer).unwrap().len(), 1);
    assert!(graph.node_pending_revalidation(consumer).unwrap().is_none());
    assert_eq!(
        graph.cause_sets.output_commit_ordinal_for_test(),
        ordinal + 1
    );
}

#[test]
fn cause_publication_merges_consumers_around_a_preserved_direct_basis() {
    let mut graph = SignalGraph::new();
    let first = graph.node().build();
    let direct = graph.node().build();
    let last = graph.node().build();
    // Producer order differs from dependency order. Insert subscriptions in
    // reverse order to exercise the native candidate normalization boundary.
    let producer = graph.node().produces_aspects(ASPECT_A).build();
    for consumer in [last, direct, first] {
        graph
            .append_dependency(consumer, producer, ASPECT_A)
            .unwrap();
    }
    for node in [producer, first, direct, last] {
        evaluate(&mut graph, node, &mut |_, _| Ok(version_ab(1, 0))).unwrap();
    }
    mark_dirty(&mut graph, producer, ASPECT_A).unwrap();
    mark_dirty(&mut graph, direct, crate::tests::support::ASPECT_B).unwrap();
    let direct_basis = graph
        .node_direct_invalidation_basis(direct)
        .unwrap()
        .cloned();
    assert!(direct_basis.is_some());
    for consumer in [first, last] {
        assert!(graph.pending_causes(consumer).unwrap().is_empty());
    }
    evaluate(&mut graph, producer, &mut |_, _| Ok(version_ab(2, 0))).unwrap();
    for consumer in [first, last] {
        assert_eq!(graph.get_state(consumer).unwrap(), NodeState::Dirty);
        assert_eq!(graph.node_dirty_aspects(consumer).unwrap(), ASPECT_A.into());
        assert_eq!(graph.pending_causes(consumer).unwrap().len(), 1);
        assert!(graph.node_pending_revalidation(consumer).unwrap().is_none());
    }
    assert_eq!(
        graph.node_direct_invalidation_basis(direct).unwrap(),
        direct_basis.as_ref()
    );
    assert_eq!(
        graph.node_dirty_aspects(direct).unwrap(),
        crate::tests::support::ASPECT_B.into()
    );
    assert!(graph.pending_causes(direct).unwrap().is_empty());
    assert!(graph
        .pending_revalidation_waiters(producer)
        .unwrap()
        .is_empty());
    assert_eq!(graph.get_state(producer).unwrap(), NodeState::Clean);
}
