use super::*;
use crate::data::comparator::{DefaultComparatorPolicyResolver, VersionComparatorResolver};
use crate::data::node::NodeState;
use crate::data::output::OutputChange;
use crate::facade::mark_dirty;
use crate::tests::support::{
    evaluate, evaluate_with_resolver, version_ab, GraphDependencyBatchExt, ASPECT_A,
};

struct CountingOutputResolver {
    calls: usize,
    decision: bool,
}

impl VersionComparatorResolver for CountingOutputResolver {
    fn resolve(
        &mut self,
        _key: &str,
        _aspect: crate::facade::Aspect,
        _cached: u64,
        _current: u64,
    ) -> Result<bool, SignalError> {
        self.calls += 1;
        Ok(self.decision)
    }
}

#[test]
fn every_prepublication_seam_leaves_semantic_state_untouched() {
    let seams = [
        OutputCommitPreparationSeam::SemanticDecision,
        OutputCommitPreparationSeam::ProducedDelta,
        OutputCommitPreparationSeam::DirectCauseAdmission,
        OutputCommitPreparationSeam::WaiterResolution,
        OutputCommitPreparationSeam::ArtifactStorage,
        OutputCommitPreparationSeam::PacketPrevalidation,
    ];
    for seam in seams {
        assert_prepublication_failure_is_atomic(seam);
    }
}

#[test]
fn changed_output_publication_commits_the_whole_authority_packet() {
    let mut graph = SignalGraph::new();
    let producer = graph.node().produces_aspects(ASPECT_A).build();
    let consumer = graph.node().build();
    graph
        .append_dependency(consumer, producer, ASPECT_A)
        .unwrap();
    evaluate(&mut graph, producer, &mut |_node, _graph| {
        Ok(version_ab(1, 0))
    })
    .unwrap();
    evaluate(&mut graph, consumer, &mut |_node, _graph| {
        Ok(version_ab(10, 0))
    })
    .unwrap();
    mark_dirty(&mut graph, producer, ASPECT_A).unwrap();

    let ordinal_before = graph.cause_sets.output_commit_ordinal_for_test();
    let recomputed_before = graph.telemetry().evaluation.nodes_recomputed;
    evaluate(&mut graph, producer, &mut |_node, _graph| {
        Ok(version_ab(2, 0))
    })
    .unwrap();

    assert_eq!(
        graph.node_aspect_version(producer).unwrap(),
        version_ab(2, 0)
    );
    assert_eq!(graph.get_state(producer).unwrap(), NodeState::Clean);
    assert!(graph.node_dirty_aspects(producer).unwrap().is_empty());
    assert_eq!(graph.get_state(consumer).unwrap(), NodeState::Dirty);
    assert!(graph
        .node_dirty_aspects(consumer)
        .unwrap()
        .contains(ASPECT_A.into()));
    assert!(graph
        .node_dirty_scoped_aspects(consumer)
        .unwrap()
        .is_empty());
    let causes = graph.pending_causes(consumer).unwrap();
    assert_eq!(causes.len(), 1);
    assert_eq!(causes[0].binding_axes.producer, producer);
    assert_eq!(causes[0].binding_axes.aspect, ASPECT_A);
    assert_eq!(causes[0].binding_axes.cached_version, 1);
    assert_eq!(causes[0].binding_axes.committed_version, 2);
    assert_eq!(
        graph.cause_sets.output_commit_ordinal_for_test(),
        ordinal_before + 1
    );
    assert_eq!(
        graph.telemetry().evaluation.nodes_recomputed,
        recomputed_before + 1
    );
    assert_eq!(
        graph.observe().explain(producer).unwrap().output_change,
        Some(OutputChange::Replaced)
    );
}

#[test]
fn mutable_output_resolver_is_consulted_once_by_canonical_commit() {
    let mut graph = SignalGraph::new();
    let producer = graph
        .node()
        .produces_aspects(ASPECT_A)
        .output_equivalence(OutputEquivalencePolicy::Custom {
            key: "stateful-output".to_string(),
        })
        .build();
    let mut resolver = CountingOutputResolver {
        calls: 0,
        decision: true,
    };
    evaluate_with_resolver(
        &mut graph,
        producer,
        &mut |_node, _graph| Ok(version_ab(1, 0)),
        &mut resolver,
    )
    .unwrap();
    resolver.calls = 0;
    resolver.decision = false;
    mark_dirty(&mut graph, producer, ASPECT_A).unwrap();
    evaluate_with_resolver(
        &mut graph,
        producer,
        &mut |_node, _graph| Ok(version_ab(2, 0)),
        &mut resolver,
    )
    .unwrap();

    assert_eq!(resolver.calls, 1);
    assert_eq!(
        graph.node_aspect_version(producer).unwrap(),
        version_ab(1, 0)
    );
}

#[test]
fn output_tolerance_cannot_suppress_a_nodes_first_committed_truth() {
    let mut graph = SignalGraph::new();
    let producer = graph
        .node()
        .produces_aspects(ASPECT_A)
        .output_equivalence(OutputEquivalencePolicy::AspectVersionTolerance { epsilon: 5 })
        .build();

    evaluate(&mut graph, producer, &mut |_node, _graph| {
        Ok(version_ab(1, 0))
    })
    .unwrap();

    assert_eq!(
        graph.node_aspect_version(producer).unwrap(),
        version_ab(1, 0)
    );
    assert!(graph.node_runtime_artifact_state_present(producer).unwrap());
}

fn assert_prepublication_failure_is_atomic(seam: OutputCommitPreparationSeam) {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(crate::facade::SignalRuntimePolicy::development());
    let producer = graph.node().produces_aspects(ASPECT_A).build();
    let consumer = graph.node().build();
    graph
        .append_dependency(consumer, producer, ASPECT_A)
        .unwrap();
    evaluate(&mut graph, producer, &mut |_node, _graph| {
        Ok(version_ab(1, 0))
    })
    .unwrap();
    evaluate(&mut graph, consumer, &mut |_node, _graph| {
        Ok(version_ab(10, 0))
    })
    .unwrap();
    mark_dirty(&mut graph, producer, ASPECT_A).unwrap();

    let before_version = graph.node_aspect_version(producer).unwrap();
    let before_producer_state = graph.get_state(producer).unwrap();
    let before_consumer_state = graph.get_state(consumer).unwrap();
    let before_snapshot = graph.get_dep_snapshot(producer).unwrap().clone();
    let before_causes = graph.pending_causes(consumer).unwrap().to_vec();
    let before_ordinal = graph.cause_sets.output_commit_ordinal_for_test();
    let before_observation = graph.observe().explain(producer).unwrap().output_change;

    let materializations = graph
        .telemetry()
        .storage
        .hot_write_cold_record_materialization_count;
    let installations = graph.telemetry().storage.hot_write_runtime_artifact_count;
    let mut effect = super::super::tests::test_effect_with_labels(vec!["prepared-cold".into()]);
    effect.operational.node = producer;
    effect.operational.aspect_version = version_ab(2, 0);
    effect.operational.output_change = OutputChange::Replaced;
    let apply = graph
        .build_apply_commit_packet(
            effect,
            OutputEquivalencePolicy::ExactAspectVersion,
            false,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
        .unwrap();
    let error = graph
        .prepare_output_commit_packet_with_probe(
            apply,
            &mut DefaultComparatorPolicyResolver::default(),
            |candidate| {
                if candidate == seam {
                    return Err(SignalError::internal("injected prepublication failure"));
                }
                Ok(())
            },
            &mut EvaluationWork::Ordinary,
        )
        .expect_err("injected seam must reject preparation");

    assert!(error
        .to_string()
        .contains("injected prepublication failure"));
    let materialized = matches!(
        seam,
        OutputCommitPreparationSeam::ArtifactStorage
            | OutputCommitPreparationSeam::PacketPrevalidation
    );
    assert_eq!(
        graph
            .telemetry()
            .storage
            .hot_write_cold_record_materialization_count,
        materializations + u64::from(materialized)
    );
    assert_eq!(
        graph.telemetry().storage.hot_write_runtime_artifact_count,
        installations
    );
    assert_eq!(graph.node_aspect_version(producer).unwrap(), before_version);
    assert_eq!(graph.get_state(producer).unwrap(), before_producer_state);
    assert_eq!(graph.get_state(consumer).unwrap(), before_consumer_state);
    assert_eq!(graph.get_dep_snapshot(producer).unwrap(), &before_snapshot);
    assert_eq!(graph.pending_causes(consumer).unwrap(), before_causes);
    assert_eq!(
        graph.cause_sets.output_commit_ordinal_for_test(),
        before_ordinal
    );
    assert_eq!(
        graph.observe().explain(producer).unwrap().output_change,
        before_observation
    );
}

#[test]
fn canonical_packet_owns_materialized_artifact_before_publication() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(crate::facade::SignalRuntimePolicy::development());
    let node = graph.node().produces_aspects(ASPECT_A).build();
    let mut effect = super::super::tests::test_effect_with_labels(vec!["prepared-cold".into()]);
    effect.operational.node = node;
    effect.operational.aspect_version = version_ab(2, 0);
    let apply = graph
        .build_apply_commit_packet(
            effect,
            OutputEquivalencePolicy::ExactAspectVersion,
            false,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
        .unwrap();
    let packet = graph
        .prepare_output_commit_packet(
            apply,
            &mut DefaultComparatorPolicyResolver::default(),
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    assert_eq!(
        packet
            .storage
            .artifact_write
            .retained
            .as_ref()
            .unwrap()
            .labels,
        vec!["prepared-cold".to_string()]
    );
    assert!(graph
        .get_entry(node)
        .unwrap()
        .retained_diagnostic_artifact()
        .is_none());
    assert_eq!(
        graph
            .telemetry()
            .storage
            .hot_write_cold_record_materialization_count,
        1
    );
    assert_eq!(
        graph.telemetry().storage.hot_write_runtime_artifact_count,
        0
    );
    graph.publish_output_commit_packet(packet);
    assert_eq!(
        graph
            .get_entry(node)
            .unwrap()
            .retained_diagnostic_artifact()
            .unwrap()
            .labels,
        vec!["prepared-cold".to_string()]
    );
    assert_eq!(
        graph
            .telemetry()
            .storage
            .hot_write_cold_record_materialization_count,
        1
    );
    assert_eq!(
        graph.telemetry().storage.hot_write_runtime_artifact_count,
        1
    );
}
