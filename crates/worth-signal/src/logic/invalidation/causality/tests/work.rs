use super::*;
use crate::data::error::SignalError;
use crate::data::retained_storage::RetainedStoragePreparation as Preparation;
use crate::logic::evaluation::EvaluationWork;

#[test]
fn conditional_cause_preparation_consumes_one_allowance_and_preserves_published_causes() {
    let (mut graph, producer, consumer, aspect) = evaluated_graph_with_edge(|node, aspect| {
        DependencyEdge::whole_partition(node, aspect, "rates")
    });
    publish_scoped_delta(
        &mut graph,
        producer,
        aspect,
        ChangedRegion::new("rates").with_detail("5y"),
    );
    let before = graph.pending_causes(consumer).unwrap().to_vec();
    assert_eq!(before.len(), 1);
    let delta = ProducedAspectDelta::from_committed_result(
        producer,
        graph.cause_sets.reserve_output_commit_ordinal(),
        AspectVersion::from_updates([(aspect, 1)]),
        AspectVersion::from_updates([(aspect, 2)]),
        AspectMask::from_aspect(aspect),
        &[(aspect, ChangedRegion::new("rates").with_detail("10y"))],
        &[],
    )
    .unwrap();
    let mut work = Preparation::new(100_000);
    graph
        .prepare_direct_output_causes(
            &delta,
            &mut DefaultComparatorPolicyResolver::default(),
            &mut EvaluationWork::Conditional(&mut work),
        )
        .unwrap();
    let cost = work.visits();
    for available in [cost - 1, cost] {
        let mut work = Preparation::new(cost + 7);
        work.reserve_visits(cost + 7 - available).unwrap();
        let result = graph.prepare_direct_output_causes(
            &delta,
            &mut DefaultComparatorPolicyResolver::default(),
            &mut EvaluationWork::Conditional(&mut work),
        );
        if available == cost {
            assert!(result.is_ok());
        } else {
            assert!(matches!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
            ));
        }
        assert_eq!(graph.pending_causes(consumer).unwrap(), before);
    }
}

#[test]
fn cause_copy_charges_both_key_and_binding_scope_payloads() {
    use crate::logic::invalidation::causality::preparation_work::admit_causes_copy;
    let (mut graph, producer, consumer, aspect) = evaluated_graph_with_edge(|node, aspect| {
        DependencyEdge::whole_partition(node, aspect, "rates")
    });
    publish_scoped_delta(&mut graph, producer, aspect, ChangedRegion::new("rates"));
    let mut causes = graph.pending_causes(consumer).unwrap().to_vec();
    let mut work = Preparation::new(100_000);
    admit_causes_copy(&causes, &mut EvaluationWork::Conditional(&mut work)).unwrap();
    let short_cost = work.visits();
    // Explicit malformed-payload seam: key and binding are distinct owned
    // fields. Accounting must not infer the binding length from its key.
    causes[0].binding_axes.edge_scope =
        Some(PartitionSubscription::whole_partition("λ".repeat(10_000)));
    assert!(matches!(
        admit_causes_copy(
            &causes,
            &mut EvaluationWork::Conditional(&mut Preparation::new(short_cost))
        ),
        Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
}

#[test]
fn bounded_cause_scope_normalization_matches_independent_set() {
    use crate::logic::invalidation::causality::scope_normalization::normalize;
    for count in 0..64 {
        let scopes: Vec<_> = (0..count)
            .map(|n| {
                PartitionSubscription::partition_and_detail(
                    format!("rates-{}", n % 7),
                    format!("λ-{}", n % 3),
                )
            })
            .collect();
        let expected: Vec<_> = scopes
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut work = Preparation::new(1_000_000);
        let result =
            normalize(scopes.clone(), &mut EvaluationWork::Conditional(&mut work)).unwrap();
        assert_eq!(result.as_slice(), expected);
        let cost = work.visits();
        assert_eq!(
            normalize(
                scopes.clone(),
                &mut EvaluationWork::Conditional(&mut Preparation::new(cost))
            )
            .unwrap(),
            result
        );
        if cost > 0 {
            assert!(matches!(
                normalize(
                    scopes,
                    &mut EvaluationWork::Conditional(&mut Preparation::new(cost - 1))
                ),
                Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
            ));
        }
    }
    let a = PartitionSubscription::whole_partition("a");
    let b = PartitionSubscription::whole_partition("b");
    assert!(PartitionScopeSet::from_canonical_scopes(vec![b, a.clone()]).is_none());
    assert!(PartitionScopeSet::from_canonical_scopes(vec![a.clone(), a]).is_none());
}
