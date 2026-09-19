use super::*;
use crate::data::retained_storage::RetainedStoragePreparation as Preparation;
use crate::logic::evaluation::EvaluationWork;

#[test]
fn conditional_reverse_query_budget_preserves_sparse_retirement_and_source() {
    let producer = NodeId::new(0, 0);
    let aspect = Aspect::new(0);
    let membership = IndexedSubscriptionMembership::from_edge(producer, aspect, None).unwrap();
    for count in [64, 4096] {
        let mut source = ReverseSubscriptionIndex::default();
        for n in 1..=count {
            source.replace_consumer(NodeId::new(n, 0), vec![membership.clone()]);
        }
        let mut fork = source.fork_persistent();
        for n in 1..count {
            fork.replace_consumer(NodeId::new(n, 0), Vec::new());
        }
        let mut work = Preparation::new(30_000);
        let expected = vec![NodeId::new(count, 0)];
        assert_eq!(
            fork.query_whole_aspect(
                producer,
                aspect,
                &mut EvaluationWork::Conditional(&mut work)
            )
            .unwrap()
            .candidates,
            expected
        );
        let cost = work.visits();
        assert!(cost < 30_000, "retired population must be skipped");
        assert_eq!(
            fork.query_whole_aspect(
                producer,
                aspect,
                &mut EvaluationWork::Conditional(&mut Preparation::new(cost))
            )
            .unwrap()
            .candidates,
            expected
        );
        assert!(matches!(
            fork.query_whole_aspect(
                producer,
                aspect,
                &mut EvaluationWork::Conditional(&mut Preparation::new(cost - 1))
            ),
            Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
        ));
        assert_eq!(
            source
                .query_whole_aspect(producer, aspect, &mut EvaluationWork::Ordinary)
                .unwrap()
                .candidates
                .len(),
            count as usize
        );
    }
}

#[test]
fn conditional_scoped_discovery_charges_query_bytes_and_preserves_exact_candidates() {
    use crate::data::dependency::DependencyEdge;
    use crate::data::output::PartitionSubscription;
    use crate::data::proof::invalidation::output_commit::{ProducedAspectChange, ScopePrecision};
    use crate::data::proof::PartitionScopeSet;
    let mut graph = SignalGraph::new();
    let producer = graph.create_node();
    let consumer = graph.create_node();
    let aspect = Aspect::new(0);
    graph
        .set_dependencies(
            consumer,
            [DependencyEdge::partition_detail(
                producer, aspect, "rates", "5y",
            )],
        )
        .unwrap();
    let mut change = ProducedAspectChange {
        aspect,
        previous_version: 0,
        committed_version: 1,
        changed_scopes: PartitionScopeSet::new([PartitionSubscription::partition_and_detail(
            "rates", "5y",
        )]),
    };
    let mut work = Preparation::new(100_000);
    assert_eq!(
        graph
            .query_reverse_subscriptions(
                producer,
                &change,
                ScopePrecision::ExactAspectScopes,
                &mut EvaluationWork::Conditional(&mut work)
            )
            .unwrap()
            .candidates,
        [consumer]
    );
    let cost = work.visits();
    assert_eq!(
        graph
            .query_reverse_subscriptions(
                producer,
                &change,
                ScopePrecision::ExactAspectScopes,
                &mut EvaluationWork::Conditional(&mut Preparation::new(cost))
            )
            .unwrap()
            .candidates,
        [consumer]
    );
    assert!(matches!(
        graph.query_reverse_subscriptions(
            producer,
            &change,
            ScopePrecision::ExactAspectScopes,
            &mut EvaluationWork::Conditional(&mut Preparation::new(cost - 1))
        ),
        Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
    change.changed_scopes = PartitionScopeSet::new([PartitionSubscription::partition_and_detail(
        "λ".repeat(10_000),
        "5y",
    )]);
    assert!(matches!(
        graph.query_reverse_subscriptions(
            producer,
            &change,
            ScopePrecision::ExactAspectScopes,
            &mut EvaluationWork::Conditional(&mut Preparation::new(cost))
        ),
        Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
    assert!(graph
        .query_reverse_subscriptions(
            producer,
            &change,
            ScopePrecision::ExactAspectScopes,
            &mut EvaluationWork::Ordinary
        )
        .unwrap()
        .candidates
        .is_empty());
}
