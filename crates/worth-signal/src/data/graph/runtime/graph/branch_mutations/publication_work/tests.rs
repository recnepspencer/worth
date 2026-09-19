use super::*;
use crate::data::aspect::Aspect;
use crate::data::graph::signal_graph::RuntimeArtifactStructuralDelta;
use crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition;
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use crate::data::reuse::ArtifactFamilyId;

fn fixture(payload: &str) -> (SignalGraph, NodeId, ReuseBasis) {
    let mut graph = SignalGraph::new();
    let node = graph.create_node();
    let source = graph.create_node();
    graph
        .set_dependencies(
            node,
            [DependencyEdge::partition_detail(
                source,
                Aspect::new(0),
                payload,
                payload,
            )],
        )
        .unwrap();
    let mut basis = ReuseBasis::default();
    basis.artifact_family_basis = Some(ArtifactFamilyId::new(payload));
    let mut runtime = crate::data::trace::RuntimeArtifactState::default();
    runtime.warm_mut().reuse_basis = crate::data::trace::ReuseOperationalBasis::new(basis.clone());
    graph
        .apply_node_artifact_write_delta(
            node,
            crate::data::trace::ArtifactWriteDelta {
                runtime: Some(runtime),
                retained: None,
            },
        )
        .unwrap();
    // Populate the existing mutation log through its owner. This is a storage
    // copy-cost fixture, not evidence of an externally performed artifact.
    graph.record_branch_mutation_runtime_artifact(
        node,
        RuntimeArtifactStructuralDelta {
            previous_artifact_id: None,
            next_artifact_id: None,
            previous_output_hash: None,
            next_output_hash: None,
            previous_reuse_basis: Some(basis.clone()),
            next_reuse_basis: Some(basis.clone()),
        },
    );
    (graph, node, basis)
}

#[test]
fn producer_branch_records_admit_payload_copies_before_publication() {
    let (mut graph, node, basis) = fixture(&"scope".repeat(1000));
    let _retained = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let prior_view = graph.observation.branch_mutation_view.fork_persistent();
    let prior_pending = graph.observation.branch_mutation_records.fork_persistent();
    let mut measured = Work::new(100_000_000);
    graph
        .admit_effect_branch_record_work(
            node,
            Some(&basis),
            &mut EvaluationWork::Conditional(&mut measured),
        )
        .unwrap();
    let cost = measured.visits();
    assert!(cost >= 5_000 * 20);
    for available in [cost - 1, cost] {
        let mut work = Work::new(cost + 29);
        work.reserve_visits(cost + 29 - available).unwrap();
        let result = graph.admit_effect_branch_record_work(
            node,
            Some(&basis),
            &mut EvaluationWork::Conditional(&mut work),
        );
        if available == cost {
            result.unwrap();
        } else {
            assert_eq!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted {
                    maximum_visits: cost + 29
                })
            );
        }
        assert_eq!(graph.observation.branch_mutation_view, prior_view);
        assert_eq!(graph.observation.branch_mutation_records, prior_pending);
    }
    // Consume the covered producer record operations on the forked source.
    let before = prior_view.get(&node).unwrap().structural_deltas.len();
    graph.record_branch_mutation_causality(node);
    graph.record_branch_mutation_runtime_artifact(
        node,
        RuntimeArtifactStructuralDelta {
            previous_artifact_id: None,
            next_artifact_id: None,
            previous_output_hash: None,
            next_output_hash: None,
            previous_reuse_basis: Some(basis.clone()),
            next_reuse_basis: Some(basis),
        },
    );
    graph.record_branch_mutation_retained_artifact(node);
    graph.record_branch_mutation_state(node);
    graph.record_branch_mutation_snapshot(
        node,
        crate::data::graph::signal_graph::DependencySnapshotStructuralDelta {
            previous_entry_count: 1,
            next_entry_count: 1,
            changed_entry_count: 1,
        },
    );
    assert_eq!(
        graph
            .observation
            .branch_mutation_view
            .get(&node)
            .unwrap()
            .structural_deltas
            .len(),
        before + 5
    );
    assert_eq!(
        prior_view.get(&node).unwrap().structural_deltas.len(),
        before
    );
    assert_eq!(
        prior_pending.get(&node).unwrap().structural_deltas.len(),
        before
    );
}

#[test]
fn producer_branch_record_work_depends_on_stored_payload_bytes() {
    let (short, node, basis) = fixture("x");
    let mut measured = Work::new(100_000_000);
    short
        .admit_effect_branch_record_work(
            node,
            Some(&basis),
            &mut EvaluationWork::Conditional(&mut measured),
        )
        .unwrap();
    let (long, node, _) = fixture(&"x".repeat(10000));
    // No new basis: the excess cost must come from the stored runtime/log data.
    assert!(matches!(
        long.admit_effect_branch_record_work(
            node,
            None,
            &mut EvaluationWork::Conditional(&mut Work::new(measured.visits()))
        ),
        Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
}
