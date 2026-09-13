use super::*;
use crate::data::aspect::AspectVersion;
use crate::data::comparator::DefaultComparatorPolicyResolver;
use crate::data::output::ChangedRegion;
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use crate::logic::evaluation::DiagnosticEnvelope;
use crate::tests::support::{evaluate, GraphDependencyBatchExt};

#[test]
fn replaced_consumer_cause_handle_denies_before_combined_node_publication() {
    let (mut graph, first) = prepared("scope");
    let producer = first.storage.apply.effect.operational.node;
    let consumer = graph
        .live_node_ids()
        .into_iter()
        .find(|node| *node != producer)
        .unwrap();
    let mut next_effect = first.storage.apply.effect.clone();
    graph.publish_output_commit_packet(first);
    assert!(!graph.pending_causes(consumer).unwrap().is_empty());
    next_effect.operational.aspect_version = AspectVersion::from_updates([(Aspect::new(0), 10)]);
    let apply = graph
        .build_apply_commit_packet(
            next_effect,
            OutputEquivalencePolicy::ExactAspectVersion,
            false,
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    let packet = graph
        .prepare_output_commit_packet(
            apply,
            &mut DefaultComparatorPolicyResolver::default(),
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    graph.release_pending_causes(consumer).unwrap();
    assert!(graph.pending_causes(consumer).unwrap().is_empty());
    let before = graph.node_aspect_version(producer).unwrap();
    let error = graph
        .prevalidate_output_commit_packet(&packet, &mut EvaluationWork::Ordinary)
        .unwrap_err();
    assert_eq!(
        error,
        SignalError::invalid_input("prepared cause slot belongs to another node state")
    );
    assert_eq!(graph.node_aspect_version(producer).unwrap(), before);
    assert!(graph.pending_causes(consumer).unwrap().is_empty());
}

fn prepared(scope: &str) -> (SignalGraph, OutputCommitPacket) {
    let mut graph = SignalGraph::new();
    let node = graph.node().produces_aspects(AspectMask::ALL).build();
    // A real settled consumer retains the published commit through its cause.
    // Unreferenced commits are intentionally not retained by the ledger.
    let consumer = graph.node().build();
    graph
        .append_dependency(consumer, node, Aspect::new(0))
        .unwrap();
    evaluate(&mut graph, consumer, &mut |_, _| Ok(AspectVersion::zero())).unwrap();

    let mut effect = super::super::tests::test_effect_with_labels(Vec::new());
    effect.operational.node = node;
    effect.operational.aspect_version = AspectVersion::from_updates([(Aspect::new(0), 9)]);
    effect.diagnostics = DiagnosticEnvelope::from_parts(
        None,
        None,
        vec![ChangedRegion::new(scope).with_detail(scope)],
        Vec::new(),
    );
    let apply = graph
        .build_apply_commit_packet(
            effect,
            OutputEquivalencePolicy::ExactAspectVersion,
            false,
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    let packet = graph
        .prepare_output_commit_packet(
            apply,
            &mut DefaultComparatorPolicyResolver::default(),
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    (graph, packet)
}

#[test]
fn final_packet_admits_comparison_and_both_publication_copies_before_movement() {
    let scope = "scope-λ".repeat(1_000);
    let (mut graph, packet) = prepared(&scope);
    let delta = packet
        .storage
        .prepared_direct
        .as_ref()
        .unwrap()
        .delta()
        .clone();
    let before = graph.node_aspect_version(delta.producer).unwrap();
    assert!(graph
        .cause_sets
        .published_output_commit(delta.output_commit_ordinal)
        .is_none());
    let mut measured = Work::new(usize::MAX);
    graph
        .prevalidate_output_commit_packet(&packet, &mut EvaluationWork::Conditional(&mut measured))
        .unwrap();
    let cost = measured.visits();
    // Two strings per scope; equality reads both operands and publication
    // creates two owned copies. Payload bytes alone require this much work.
    assert!(cost >= 8 * scope.len());
    for available in [8 * scope.len() - 1, cost - 1, cost] {
        let mut work = Work::new(cost + 7);
        work.reserve_visits(cost + 7 - available).unwrap();
        let result = graph
            .prevalidate_output_commit_packet(&packet, &mut EvaluationWork::Conditional(&mut work));
        if available == cost {
            result.unwrap();
            assert_eq!(work.visits(), cost + 7);
        } else {
            assert_eq!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted {
                    maximum_visits: cost + 7,
                })
            );
        }
        assert_eq!(graph.node_aspect_version(delta.producer).unwrap(), before);
        assert!(graph
            .cause_sets
            .published_output_commit(delta.output_commit_ordinal)
            .is_none());
    }
    let (report, _) = graph.publish_output_commit_packet(packet);
    assert!(matches!(report.verdict, EvaluationVerdict::Recomputed));
    assert_eq!(
        graph.node_aspect_version(delta.producer).unwrap(),
        delta.committed_output_version
    );
    assert_eq!(
        graph
            .cause_sets
            .published_output_commit(delta.output_commit_ordinal),
        Some(&delta)
    );
}

#[test]
fn final_packet_work_depends_on_scope_bytes_and_rejects_mismatched_delta() {
    let (short_graph, short_packet) = prepared("x");
    let mut short_work = Work::new(1_000_000);
    short_graph
        .prevalidate_output_commit_packet(
            &short_packet,
            &mut EvaluationWork::Conditional(&mut short_work),
        )
        .unwrap();
    let (graph, packet) = prepared(&"x".repeat(1_000));
    assert!(matches!(
        graph.prevalidate_output_commit_packet(
            &packet,
            &mut EvaluationWork::Conditional(&mut Work::new(short_work.visits())),
        ),
        Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
    // Corrupt only the comparison query, keeping the actual prepared admission.
    // This fixture tests packet validation, not public construction authority.
    let mut other = packet
        .storage
        .prepared_direct
        .as_ref()
        .unwrap()
        .delta()
        .clone();
    other.changes.first_mut_for_test().committed_version += 1;
    assert!(packet
        .storage
        .direct_causes
        .as_ref()
        .unwrap()
        .validate_packet(other.producer, Some(&other), &mut EvaluationWork::Ordinary,)
        .is_err());
}

#[test]
fn cause_store_gather_admits_retained_handle_lookups() {
    let (mut graph, packet) = prepared("scope");
    let extras: Vec<_> = (0..256).map(|_| graph.create_node()).collect();
    let _retained = crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition::retain_basis_storage(&mut graph);
    for node in extras.into_iter().step_by(64) {
        graph
            .set_node_state(node, crate::data::node::NodeState::Dirty)
            .unwrap();
    }
    let causes = packet.storage.direct_causes.as_ref().unwrap();
    let mut measured = Work::new(usize::MAX);
    causes
        .admit_cause_store_work(&graph, &mut EvaluationWork::Conditional(&mut measured))
        .unwrap();
    let cost = measured.visits();
    assert!(cost >= 2 * (graph.arena.nodes.lookup_steps() + graph.arena.hot.lookup_steps()));
    for available in [0, cost - 1, cost] {
        let mut work = Work::new(cost + 13);
        work.reserve_visits(cost + 13 - available).unwrap();
        let result =
            causes.admit_cause_store_work(&graph, &mut EvaluationWork::Conditional(&mut work));
        if available == cost {
            result.unwrap();
            assert_eq!(work.visits(), cost + 13);
        } else {
            assert!(matches!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
            ));
        }
        assert!(!graph.cause_sets.has_occupied_sets());
    }
}

#[test]
fn projected_cause_version_drift_is_denied_before_producer_write() {
    let (graph, mut packet) = prepared("scope");
    let producer = packet.storage.apply.effect.operational.node;
    let before = graph.node_aspect_version(producer).unwrap();
    graph
        .prevalidate_output_commit_packet(&packet, &mut EvaluationWork::Ordinary)
        .unwrap();
    packet.storage.apply.effect.operational.aspect_version = before;
    assert!(graph
        .prevalidate_output_commit_packet(&packet, &mut EvaluationWork::Ordinary)
        .is_err());
    assert_eq!(graph.node_aspect_version(producer).unwrap(), before);
    assert!(!graph.cause_sets.has_occupied_sets());
}

#[test]
fn scoped_cause_validation_admits_both_checks_before_publication() {
    let mut graph = SignalGraph::new();
    let producer = graph.node().produces_aspects(AspectMask::ALL).build();
    let consumer = graph.create_node();
    let aspect = Aspect::new(0);
    let text = "validation-key".repeat(1000);
    graph
        .set_dependencies(
            consumer,
            [crate::data::dependency::DependencyEdge::partition_detail(
                producer,
                aspect,
                text.as_str(),
                text.as_str(),
            )],
        )
        .unwrap();
    evaluate(&mut graph, consumer, &mut |_, _| Ok(AspectVersion::zero())).unwrap();
    let _retained = crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let mut effect = super::super::tests::test_effect_with_labels(Vec::new());
    effect.operational.node = producer;
    effect.operational.aspect_version = AspectVersion::from_updates([(aspect, 9)]);
    effect.diagnostics = DiagnosticEnvelope::from_parts(
        None,
        None,
        vec![ChangedRegion::new(text.as_str()).with_detail(text.as_str())],
        Vec::new(),
    );
    let apply = graph
        .build_apply_commit_packet(
            effect,
            OutputEquivalencePolicy::ExactAspectVersion,
            false,
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    let packet = graph
        .prepare_output_commit_packet(
            apply,
            &mut DefaultComparatorPolicyResolver::default(),
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    let mut measured = Work::new(usize::MAX);
    graph
        .prevalidate_output_commit_packet(&packet, &mut EvaluationWork::Conditional(&mut measured))
        .unwrap();
    let cost = measured.visits();
    assert!(cost > text.len() * 8);
    for available in [cost - 1, cost] {
        let mut work = Work::new(cost + 23);
        work.reserve_visits(cost + 23 - available).unwrap();
        let result = graph
            .prevalidate_output_commit_packet(&packet, &mut EvaluationWork::Conditional(&mut work));
        if available == cost {
            result.unwrap();
            assert_eq!(work.visits(), cost + 23);
        } else {
            assert!(matches!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
            ));
        }
        assert!(graph.pending_causes(consumer).unwrap().is_empty());
        assert_eq!(
            graph.node_aspect_version(producer).unwrap(),
            AspectVersion::zero()
        );
    }
    graph.publish_output_commit_packet(packet);
    let causes = graph.pending_causes(consumer).unwrap();
    assert_eq!(causes.len(), 1);
    assert_eq!(causes[0].changed_scopes.as_slice()[0].partition.0, text);
    assert_eq!(graph.node_aspect_version(producer).unwrap().get(aspect), 9);
}

#[test]
fn retained_packet_reserves_node_containers_before_selected_reads_or_writes() {
    let (mut graph, packet) = prepared("container-work");
    let producer = packet.storage.apply.effect.operational.node;
    for _ in 0..256 {
        graph.create_node();
    }
    let _retained = crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let sibling_hot = graph.arena.hot.fork_persistent();
    let before = graph.node_aspect_version(producer).unwrap();
    let mut measured = Work::new(usize::MAX);
    graph
        .admit_effect_node_mutation_work(&mut EvaluationWork::Conditional(&mut measured))
        .unwrap();
    let container_cost = measured.visits();
    assert!(container_cost > 20);
    let mut short = Work::new(container_cost + 11);
    short.reserve_visits(12).unwrap();
    assert_eq!(
        graph.prevalidate_output_commit_packet(
            &packet,
            &mut EvaluationWork::Conditional(&mut short),
        ),
        Err(SignalError::ConditionalEvaluationWorkExhausted {
            maximum_visits: container_cost + 11,
        }),
    );
    // The first atomic reservation denies without even consuming a partial
    // allowance, and the publication's node storage remains shared.
    assert_eq!(short.visits(), 12);
    assert!(graph.arena.hot.shares_storage_with(&sibling_hot));
    assert_eq!(graph.node_aspect_version(producer).unwrap(), before);
    let mut admitted = Work::new(usize::MAX);
    graph
        .prevalidate_output_commit_packet(&packet, &mut EvaluationWork::Conditional(&mut admitted))
        .unwrap();
    graph.publish_output_commit_packet(packet);
    assert_eq!(
        graph
            .node_aspect_version(producer)
            .unwrap()
            .get(Aspect::new(0)),
        9
    );
    assert_eq!(
        sibling_hot[producer.index() as usize]
            .as_ref()
            .unwrap()
            .aspect_version_header
            .global(),
        before,
    );
}
