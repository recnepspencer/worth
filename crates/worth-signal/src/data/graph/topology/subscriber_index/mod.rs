//! Rebuildable producer-local reverse subscription narrowing.
//!
//! Dependency edges remain authoritative. This index only discovers candidate
//! consumers for one performed producer/aspect/scope change.

mod buckets;
pub(crate) mod candidates;
mod membership;
mod rebuild;

pub(crate) use buckets::ReverseSubscriptionIndex;

#[cfg(test)]
mod tests {
    use crate::data::aspect::{Aspect, AspectMask, AspectVersion};
    use crate::data::comparator::DefaultComparatorPolicyResolver;
    use crate::data::dependency::DependencyEdge;
    use crate::data::graph::SignalGraph;
    use crate::data::output::PartitionSubscription;
    use crate::data::proof::invalidation::output_commit::{
        ProducedAspectChange, ProducedAspectDelta, ScopePrecision,
    };
    use crate::data::proof::PartitionScopeSet;

    #[test]
    fn exact_detail_query_returns_unscoped_whole_and_detail_consumers_only() {
        let mut graph = SignalGraph::new();
        let producer = graph.create_node();
        let unscoped = graph.create_node();
        let whole = graph.create_node();
        let detail = graph.create_node();
        let disjoint = graph.create_node();
        let aspect = Aspect::new(2);
        graph
            .set_dependencies(unscoped, [DependencyEdge::new(producer, aspect)])
            .unwrap();
        graph
            .set_dependencies(
                whole,
                [DependencyEdge::whole_partition(producer, aspect, "rates")],
            )
            .unwrap();
        graph
            .set_dependencies(
                detail,
                [DependencyEdge::partition_detail(
                    producer, aspect, "rates", "5y",
                )],
            )
            .unwrap();
        graph
            .set_dependencies(
                disjoint,
                [DependencyEdge::partition_detail(
                    producer, aspect, "rates", "10y",
                )],
            )
            .unwrap();

        let query = graph
            .query_reverse_subscriptions(
                producer,
                &ProducedAspectChange {
                    aspect,
                    previous_version: 0,
                    committed_version: 1,
                    changed_scopes: PartitionScopeSet::new([
                        PartitionSubscription::partition_and_detail("rates", "5y"),
                    ]),
                },
                ScopePrecision::ExactAspectScopes,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary,
            )
            .unwrap();

        assert_eq!(query.candidates, vec![unscoped, whole, detail]);
        assert_eq!(query.bucket_probes, 3);
    }

    #[test]
    fn destroyed_index_fails_closed_until_authority_rebuild() {
        let mut graph = SignalGraph::new();
        let producer = graph.create_node();
        let consumer = graph.create_node();
        let aspect = Aspect::new(2);
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
        let later_consumer = graph.create_node();
        graph
            .set_dependencies(later_consumer, [DependencyEdge::new(producer, aspect)])
            .unwrap();
        assert!(
            graph
                .query_reverse_subscriptions(
                    producer,
                    &change,
                    ScopePrecision::ExactAspectScopes,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .is_err(),
            "a partial membership update must not certify a destroyed index as rebuilt"
        );
        graph
            .rebuild_reverse_subscription_index_from_dependencies()
            .unwrap();
        assert_eq!(
            graph
                .query_reverse_subscriptions(
                    producer,
                    &change,
                    ScopePrecision::ExactAspectScopes,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap()
                .candidates,
            vec![consumer, later_consumer]
        );
    }

    #[test]
    fn drift_candidate_cannot_mutate_or_resolve_a_non_subscriber() {
        let mut graph = SignalGraph::new();
        let changed_producer = graph.create_node();
        let actual_producer = graph.create_node();
        let consumer = graph.create_node();
        let aspect = Aspect::new(2);
        graph
            .set_dependencies(consumer, [DependencyEdge::new(actual_producer, aspect)])
            .unwrap();
        graph
            .topology
            .reverse_subscriptions
            .inject_candidate_drift_for_test(changed_producer, aspect, consumer);
        let state_before = graph.get_state(consumer).unwrap();
        let rejection_before = graph.telemetry().invalidation.direct_causality_rejections;
        let delta = ProducedAspectDelta::from_committed_result(
            changed_producer,
            graph.cause_sets.reserve_output_commit_ordinal(),
            AspectVersion::zero(),
            AspectVersion::from_updates([(aspect, 1)]),
            AspectMask::from_aspect(aspect),
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

        assert_eq!(graph.get_state(consumer).unwrap(), state_before);
        assert!(graph.pending_causes(consumer).unwrap().is_empty());
        assert_eq!(
            graph.telemetry().invalidation.direct_causality_rejections - rejection_before,
            1
        );
    }
}
