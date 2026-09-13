use super::*;
use crate::data::graph::PendingRevalidationNodeProjection;
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use std::collections::BTreeMap;

#[test]
fn downstream_warm_copy_and_waiter_retirement_are_admitted_before_publication() {
    let mut source = SignalGraph::new();
    let producer = source.create_node();
    let consumer = source.create_node();
    source.transition_node_clean(producer).unwrap();
    source.transition_node_clean(consumer).unwrap();
    source
        .install_node_dependency_revalidation(consumer, [producer], false)
        .unwrap();
    source.replace_pending_revalidation_waiters(consumer, &[], &[producer]);
    let payload = "downstream-token".repeat(1000);
    let mut runtime = crate::data::trace::RuntimeArtifactState::default();
    runtime.warm_mut().output_identity = Some(crate::data::output::OutputIdentity::new(&payload));
    // Retained payload fixture, independent of semantic output production.
    source
        .apply_node_artifact_write_delta(
            consumer,
            crate::data::trace::ArtifactWriteDelta {
                runtime: Some(runtime),
                retained: None,
            },
        )
        .unwrap();
    let (mut graph, _) = source.fork_persistent();
    let mut preparation = Work::new(usize::MAX);
    let projection =
        PendingRevalidationNodeProjection::capture(&graph, producer, &mut preparation).unwrap();
    let prepared = graph
        .prepare_pending_revalidation_resolution(
            producer,
            BTreeMap::from([(producer, projection)]),
            &mut preparation,
        )
        .unwrap();
    assert!(prepared.nodes.contains_key(&consumer));
    assert!(prepared.buckets.get(&producer).unwrap().is_empty());
    let mut measured = Work::new(usize::MAX);
    graph
        .admit_pending_resolution_publication_work(
            &prepared,
            producer,
            &mut EvaluationWork::Conditional(&mut measured),
        )
        .unwrap();
    let cost = measured.visits();
    let mut node_writes = Work::new(usize::MAX);
    graph
        .admit_downstream_node_mutation_work(&mut EvaluationWork::Conditional(&mut node_writes))
        .unwrap();
    assert!(node_writes.visits() > 6);
    assert!(cost >= payload.len() + node_writes.visits());
    for available in [cost - 1, cost] {
        let mut work = Work::new(cost + 7);
        work.reserve_visits(cost + 7 - available).unwrap();
        let result = graph.admit_pending_resolution_publication_work(
            &prepared,
            producer,
            &mut EvaluationWork::Conditional(&mut work),
        );
        if available == cost {
            result.unwrap();
            assert_eq!(work.visits(), cost + 7);
        } else {
            assert_eq!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted {
                    maximum_visits: cost + 7
                })
            );
        }
        assert!(graph.node_pending_revalidation(consumer).unwrap().is_some());
        assert_eq!(
            graph
                .topology
                .pending_revalidation_waiters
                .get(&producer)
                .unwrap()
                .len(),
            1
        );
    }
    graph
        .publish_pending_revalidation_resolution(prepared)
        .unwrap();
    assert!(graph.node_pending_revalidation(consumer).unwrap().is_none());
    assert!(graph
        .topology
        .pending_revalidation_waiters
        .get(&producer)
        .is_none());
    assert!(source
        .node_pending_revalidation(consumer)
        .unwrap()
        .is_some());
    assert_eq!(
        source
            .topology
            .pending_revalidation_waiters
            .get(&producer)
            .unwrap()
            .len(),
        1
    );
}
