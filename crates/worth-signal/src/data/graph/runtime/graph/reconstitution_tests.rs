use super::SignalGraph;
use crate::data::aspect::Aspect;
use crate::data::proof::invalidation::output_commit::{ProducedAspectChange, ScopePrecision};
use crate::data::proof::PartitionScopeSet;
use crate::facade::DependencyEdge;

#[test]
fn runtime_reconstitution_rebuilds_destroyed_reverse_index_under_fresh_identity() {
    let mut graph = SignalGraph::new();
    let producer = graph.create_node();
    let consumer = graph.create_node();
    let aspect = Aspect::new(3);
    graph
        .set_dependencies(consumer, [DependencyEdge::new(producer, aspect)])
        .unwrap();
    let change = ProducedAspectChange {
        aspect,
        previous_version: 0,
        committed_version: 1,
        changed_scopes: PartitionScopeSet::default(),
    };
    graph.destroy_reverse_subscription_index_for_test();
    assert!(graph
        .query_reverse_subscriptions(
            producer,
            &change,
            ScopePrecision::ExactAspectScopes,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary
        )
        .is_err());

    let prior_graph = graph.installed_graph_capability().graph_instance_id();
    let (mut restored, report) = graph
        .reconstitute_for_runtime_rebind()
        .unwrap()
        .into_parts();

    assert_eq!(report.previous_graph_instance_id(), prior_graph);
    assert_ne!(report.restored_graph_instance_id(), prior_graph);
    assert_eq!(report.reconstructed_node_count(), 2);
    assert_eq!(report.checkpoint_reconstruction_count(), 1);
    assert_eq!(
        restored
            .query_reverse_subscriptions(
                producer,
                &change,
                ScopePrecision::ExactAspectScopes,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary
            )
            .unwrap()
            .candidates,
        vec![consumer]
    );
}
