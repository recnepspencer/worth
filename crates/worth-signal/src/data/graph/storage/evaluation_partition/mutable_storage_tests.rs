use super::*;
use crate::data::dependency::DependencyEdge;
use crate::data::handle::NodeId;
use crate::data::retained_storage::{
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial,
};
use crate::facade::{NodeEvaluationResult, SignalRuntimePolicy};
use crate::tests::support::{evaluate, version_ab, ASPECT_A};

fn evaluated_slot() -> (
    SignalGraph,
    SignalExecutionBasis,
    SignalEvaluationPartition,
    [NodeId; 2],
) {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let nodes = [graph.node().output_identity().build(), graph.node().build()];
    let basis = SignalExecutionBasis::capture(&mut graph, &mut Work::new(100_000)).unwrap();
    let mut slot = basis.new_evaluation_partition();
    slot.execute(&mut graph, |selected| {
        evaluate(selected, nodes[0], &mut |_, _| {
            Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
                .with_output_identity("retained artifact")
                .with_label("retained diagnostic payload"))
        })
        .unwrap();
    })
    .unwrap();
    (graph, basis, slot, nodes)
}

#[test]
fn mutable_evaluation_fork_preserves_history_and_isolates_canonical_node_and_topology_edits() {
    let (mut graph, basis, mut slot, nodes) = evaluated_slot();
    let draft = slot.evaluation.fork_persistent();
    let (before, hot, warm, cold) = slot
        .execute(&mut graph, |selected| {
            (
                serde_json::to_value(selected.diagnostics_state()).unwrap(),
                selected.arena.hot.clone(),
                selected.arena.warm.clone(),
                selected.arena.cold.clone(),
            )
        })
        .unwrap();
    assert!(!before["recent_history"].as_array().unwrap().is_empty());
    assert!(before["latest_flow"].is_object());
    let mut draft = SignalEvaluationPartition::from_retained(slot.definitions.clone(), draft);
    draft
        .execute(&mut graph, |selected| {
            assert!(selected.arena.hot.shares_storage_with(&hot));
            assert!(selected.arena.warm.shares_storage_with(&warm));
            assert!(selected.arena.cold.shares_storage_with(&cold));
            assert_eq!(
                serde_json::to_value(selected.diagnostics_state()).unwrap(),
                before
            );
            evaluate(&mut *selected, nodes[1], &mut |_, _| Ok(version_ab(7, 0))).unwrap();
            selected
                .set_dependencies(nodes[0], [DependencyEdge::new(nodes[1], ASPECT_A)])
                .unwrap();
            assert_eq!(
                selected.dependencies_of(nodes[0]).unwrap()[0].source(),
                nodes[1]
            );
            assert_eq!(
                selected
                    .node_version_for_scope(nodes[1], ASPECT_A, None)
                    .unwrap(),
                7
            );
            let after = serde_json::to_value(selected.diagnostics_state()).unwrap();
            assert!(
                after["recent_history"].as_array().unwrap().len()
                    > before["recent_history"].as_array().unwrap().len()
            );
        })
        .unwrap();
    slot.execute(&mut graph, |selected| {
        assert!(selected.dependencies_of(nodes[0]).unwrap().is_empty());
        assert_eq!(
            selected
                .node_version_for_scope(nodes[0], ASPECT_A, None)
                .unwrap(),
            1
        );
        assert_eq!(
            selected
                .node_version_for_scope(nodes[1], ASPECT_A, None)
                .unwrap(),
            0
        );
        assert_eq!(
            serde_json::to_value(selected.diagnostics_state()).unwrap(),
            before
        );
    })
    .unwrap();
    basis
        .new_evaluation_partition()
        .execute(&mut graph, |selected| {
            assert_eq!(
                selected
                    .node_version_for_scope(nodes[0], ASPECT_A, None)
                    .unwrap(),
                0
            );
            assert!(
                serde_json::to_value(selected.diagnostics_state()).unwrap()["recent_history"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
        })
        .unwrap();
}

#[test]
fn mutable_evaluation_preparation_bounds_full_diagnostics_without_changing_semantics() {
    let (mut graph, _basis, mut slot, _) = evaluated_slot();
    let (mut short_graph, _short_basis, mut short, _) = evaluated_slot();
    let (_exact_graph, _exact_basis, mut exact, _) = evaluated_slot();
    // Independent native fixtures prevent one preparation from warming the
    // other fixture's shared history metadata. Match their fork representation.
    slot.evaluation = slot.evaluation.fork_persistent();
    short.evaluation = short.evaluation.fork_persistent();
    exact.evaluation = exact.evaluation.fork_persistent();
    let before = slot
        .execute(&mut graph, |selected| {
            serde_json::to_value(selected.diagnostics_state()).unwrap()
        })
        .unwrap();
    let short_before = short
        .execute(&mut short_graph, |selected| {
            serde_json::to_value(selected.diagnostics_state()).unwrap()
        })
        .unwrap();
    let mut work = Work::new(100_000);
    let charge = slot
        .evaluation
        .prepare_mutable_heap_charge(&mut work)
        .unwrap();
    assert!(charge.bytes() > 0);
    assert_eq!(
        exact
            .evaluation
            .prepare_mutable_heap_charge(&mut Work::new(work.visits()))
            .unwrap(),
        charge
    );
    let limit = work.visits() - 1;
    assert_eq!(
        short
            .evaluation
            .prepare_mutable_heap_charge(&mut Work::new(limit)),
        Err(
            super::super::execution_basis::SignalExecutionBasisChargeDenial::Storage(
                RetainedStoragePreparationDenial::WorkExhausted {
                    maximum_visits: limit
                }
            )
        )
    );
    assert_eq!(
        short
            .evaluation
            .prepare_mutable_heap_charge(&mut Work::new(100_000))
            .unwrap(),
        charge
    );
    short
        .execute(&mut short_graph, |selected| {
            assert_eq!(
                serde_json::to_value(selected.diagnostics_state()).unwrap(),
                short_before
            );
        })
        .unwrap();
    slot.execute(&mut graph, |selected| {
        assert_eq!(
            serde_json::to_value(selected.diagnostics_state()).unwrap(),
            before
        );
    })
    .unwrap();
}

#[test]
fn native_output_publication_preserves_carried_node_lane_charges() {
    use crate::data::retained_storage::RetainedStorageMeasurement;

    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let node = graph.node().output_identity().build();
    let basis = SignalExecutionBasis::capture(&mut graph, &mut Work::new(100_000)).unwrap();
    let mut slot = basis.new_evaluation_partition();
    slot.execute(&mut graph, |selected| {
        // Cold capture has already established the facts. The evaluation must
        // carry updates, not recover them by scanning all retained nodes later.
        assert!(selected.arena.hot.prepared_retained_charge().is_ok());
        assert!(selected.arena.warm.prepared_retained_charge().is_ok());
        assert!(selected.arena.cold.prepared_retained_charge().is_ok());
        evaluate(&mut *selected, node, &mut |_, _| {
            Ok(NodeEvaluationResult::from_version(version_ab(7, 0))
                .with_output_identity("published identity")
                .with_label("published diagnostic payload"))
        })
        .unwrap();
        assert_eq!(
            selected
                .node_version_for_scope(node, ASPECT_A, None)
                .unwrap(),
            7
        );
        let carried = [
            selected.arena.hot.prepared_retained_charge(),
            selected.arena.warm.prepared_retained_charge(),
            selected.arena.cold.prepared_retained_charge(),
        ];
        let measured = [
            selected
                .arena
                .hot
                .retained_heap_charge(&mut Work::new(100_000))
                .unwrap(),
            selected
                .arena
                .warm
                .retained_heap_charge(&mut Work::new(100_000))
                .unwrap(),
            selected
                .arena
                .cold
                .retained_heap_charge(&mut Work::new(100_000))
                .unwrap(),
        ];
        assert_eq!(
            carried,
            measured.map(Ok),
            "native publication must preserve each admitted lane charge"
        );
    })
    .unwrap();
}
