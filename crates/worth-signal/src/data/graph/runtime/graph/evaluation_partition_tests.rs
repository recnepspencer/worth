use super::cause_sets_tests::{graph_with_edge, publish_delta};
use crate::data::dependency::{DependencyEdge, DependencySnapshot};
use crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition;

mod conditional_execution;
mod retained_basis;

#[test]
fn retained_storage_isolates_versions_causes_and_dynamic_dependencies() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    let alternate = graph.create_node();
    let original_ledger = graph.pending_branch_mutation_records();
    let mut a = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let mut b = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    a.execute(&mut graph, |graph| {
        publish_delta(graph, producer, aspect, 0, 1, 1);
        assert_eq!(graph.pending_causes(consumer).unwrap().len(), 1);
    })
    .unwrap();
    let b_capacity = b
        .execute(&mut graph, |graph| {
            assert!(graph.pending_causes(consumer).unwrap().is_empty());
            graph
                .set_dependencies(consumer, [DependencyEdge::new(alternate, aspect)])
                .unwrap();
            let mut snapshot = DependencySnapshot::empty();
            snapshot.record(alternate, aspect, 0, None);
            graph.set_dep_snapshot(consumer, snapshot).unwrap();
            publish_delta(graph, alternate, aspect, 0, 3, 1);
            assert_eq!(graph.pending_causes(consumer).unwrap().len(), 1);
            graph.traversal.topology_node_buffer.capacity()
        })
        .unwrap();
    a.execute(&mut graph, |graph| {
        assert_eq!(
            graph.dependencies_of(consumer).unwrap()[0].source(),
            producer
        );
        assert_eq!(
            graph.get_dep_snapshot(consumer).unwrap().entries()[0].source,
            producer
        );
        assert_eq!(
            graph
                .node_version_for_scope(producer, aspect, None)
                .unwrap(),
            1
        );
        assert_eq!(graph.pending_causes(consumer).unwrap().len(), 1);
        graph.transition_node_clean(consumer).unwrap();
    })
    .unwrap();
    assert!(
        b_capacity > 0,
        "rewiring must exercise reusable topology scratch"
    );
    b.execute(&mut graph, |graph| {
        assert_eq!(graph.traversal.topology_node_buffer.capacity(), b_capacity);
        assert_eq!(
            graph.dependencies_of(consumer).unwrap()[0].source(),
            alternate
        );
        assert_eq!(
            graph.get_dep_snapshot(consumer).unwrap().entries()[0].cached_version,
            0
        );
        assert_eq!(
            graph
                .node_version_for_scope(alternate, aspect, None)
                .unwrap(),
            3
        );
        // Releasing A's cause handle must not release B's same-shaped handle.
        assert_eq!(graph.pending_causes(consumer).unwrap().len(), 1);
    })
    .unwrap();
    assert_eq!(
        graph
            .node_version_for_scope(producer, aspect, None)
            .unwrap(),
        0
    );
    assert_eq!(
        graph.dependencies_of(consumer).unwrap()[0].source(),
        producer
    );
    assert!(graph.pending_causes(consumer).unwrap().is_empty());
    assert_eq!(graph.pending_branch_mutation_records(), original_ledger);
}

#[test]
fn unwind_restores_graph_and_retains_performed_partition_work() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    let original_ledger = graph.pending_branch_mutation_records();
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        partition
            .execute(&mut graph, |graph| {
                publish_delta(graph, producer, aspect, 0, 7, 1);
                panic!("provider failed after derived propagation");
            })
            .unwrap();
    }));
    assert!(unwind.is_err());
    assert_eq!(
        graph
            .node_version_for_scope(producer, aspect, None)
            .unwrap(),
        0
    );
    assert!(graph.pending_causes(consumer).unwrap().is_empty());
    assert_eq!(graph.pending_branch_mutation_records(), original_ledger);
    partition
        .execute(&mut graph, |graph| {
            assert_eq!(
                graph
                    .node_version_for_scope(producer, aspect, None)
                    .unwrap(),
                7
            );
            assert_eq!(graph.pending_causes(consumer).unwrap().len(), 1);
            graph.transition_node_clean(consumer).unwrap();
        })
        .unwrap();
    assert!(graph.pending_causes(consumer).unwrap().is_empty());
}

#[test]
fn retained_definition_unwind_restores_current_membership_and_rejects_foreign_graph() {
    let (mut graph, _, _, _) = graph_with_edge();
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let added = graph.create_node();
    let current_ledger = graph.pending_branch_mutation_records();
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        partition
            .execute(&mut graph, |selected| {
                assert!(selected.get_entry(added).is_err());
                panic!("provider unwinds while old definitions are selected");
            })
            .unwrap();
    }));
    assert!(unwind.is_err());
    assert!(graph.get_entry(added).is_ok());
    assert_eq!(graph.pending_branch_mutation_records(), current_ledger);
    partition
        .execute(&mut graph, |selected| {
            assert!(selected.get_entry(added).is_err());
        })
        .unwrap();

    let (mut foreign, _, _, _) = graph_with_edge();
    let mut contacted = false;
    let denial = partition.execute(&mut foreign, |_| {
        contacted = true;
    });
    assert!(denial.is_err());
    assert!(!contacted);
}
