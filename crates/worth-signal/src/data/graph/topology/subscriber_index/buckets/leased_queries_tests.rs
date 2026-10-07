//! Leased candidate discovery is checked against current authoritative edges.
use crate::data::aspect::{Aspect, AspectVersion};
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::{PartitionSubscription, ScopeCoverage, ScopePath};
use crate::data::proof::invalidation::binding::OutputCommitOrdinal;
use crate::data::proof::invalidation::output_commit::{
    NonEmptyCanonicalAspectChangeSet, ProducedAspectChange, ProducedAspectDelta, ScopePrecision,
};
use crate::data::proof::PartitionScopeSet;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::logic::evaluation::EvaluationWork;
use crate::tests::leased_execution::support::{authority, request};
use worth_execution::{ExecutionScan, MapKernelFailure, MapStop, ScanOutcome};
use worth_foundational::{ExecutionPosture, PartitionIdentity};

#[test]
fn checked_candidate_preparation_rejects_a_destroyed_reverse_index() {
    let mut graph = SignalGraph::new();
    let producer = graph.create_node();
    graph.destroy_reverse_subscription_index_for_test();
    let lease = authority().request_lease(request(1, 1_000_000)).unwrap();
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).unwrap();
    let mut budget = SignalPreparationBudget::new(4 * 1024 * 1024);
    let outcome = scan.run(
        Some(&lease),
        (),
        0,
        0,
        4 * 1024 * 1024,
        4 * 1024 * 1024,
        |_, _, work| {
            let error = graph
                .prepare_candidate_epoch(
                    std::iter::once(producer),
                    0,
                    ExecutionPosture::Serial,
                    &lease,
                    work,
                    &mut budget,
                )
                .err()
                .expect("destroyed index must deny before evaluator work");
            Err::<((), ()), _>(MapKernelFailure::Domain(error))
        },
    );
    let ScanOutcome::Stopped {
        completed_prefix,
        reason:
            MapStop::Failure {
                cause: MapKernelFailure::Domain(error),
                ..
            },
        ..
    } = outcome
    else {
        panic!("owner guard must return its original Signal error before publication")
    };
    assert!(completed_prefix.is_empty());
    assert!(matches!(
        &error,
        crate::data::error::SignalError::Internal { message, .. }
        if message == "reverse subscription index requires authority rebuild"
    ));
}

#[test]
fn leased_hierarchy_candidates_match_current_edges_after_scope_replacement() {
    for depth in [1, 2, 4, 8] {
        for workers in [1, 2, 4] {
            let mut graph = SignalGraph::new();
            let producer = graph.create_node();
            let path = ScopePath::new((0..depth).map(|level| format!("level{level}"))).unwrap();
            let mut sibling = path.segments().to_vec();
            sibling[depth - 1] = "sibling".into();
            let sibling = ScopePath::new(sibling).unwrap();
            let scopes = [
                None,
                Some(PartitionSubscription::exact(path.clone())),
                Some(PartitionSubscription::subtree(path.clone())),
                Some(PartitionSubscription::subtree(
                    ScopePath::one("level0").unwrap(),
                )),
                Some(PartitionSubscription::exact(sibling.clone())),
            ];
            let consumers = scopes
                .iter()
                .map(|scope| {
                    let consumer = graph.create_node();
                    graph
                        .set_dependencies(consumer, [edge(producer, scope.clone())])
                        .unwrap();
                    consumer
                })
                .collect::<Vec<_>>();
            // Retire the exact membership through the real topology owner.
            graph
                .set_dependencies(
                    consumers[1],
                    [edge(producer, Some(PartitionSubscription::exact(sibling)))],
                )
                .unwrap();
            let mut queries = vec![
                PartitionSubscription::exact(path.clone()),
                PartitionSubscription::subtree(path.clone()),
            ];
            if depth < ScopePath::MAX_DEPTH {
                queries.push(PartitionSubscription::exact(
                    path.with_segment("unknown-leaf").unwrap(),
                ));
            }
            for query in queries {
                let expected = oracle(&graph, producer, &consumers, &query);
                let delta = candidate_delta(producer, query);
                let lease = authority()
                    .request_lease(request(workers, 2_000_000))
                    .unwrap();
                let identity = PartitionIdentity::new(1);
                let scan =
                    ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).unwrap();
                let mut budget = SignalPreparationBudget::new(4 * 1024 * 1024);
                let outcome = scan.run(
                    Some(&lease),
                    (),
                    0,
                    4 * 1024 * 1024,
                    4 * 1024 * 1024,
                    4 * 1024 * 1024,
                    |_, _, work| {
                        let queries = graph
                            .collect_reverse_subscription_queries(
                                &delta,
                                &mut EvaluationWork::Ordinary,
                                Some(&lease),
                                Some(work),
                                Some(&mut budget),
                            )
                            .map_err(MapKernelFailure::Domain)?;
                        Ok(((), queries))
                    },
                );
                let ScanOutcome::Complete {
                    prefixes, report, ..
                } = outcome
                else {
                    let ScanOutcome::Stopped { reason, .. } = outcome else {
                        unreachable!()
                    };
                    panic!("leased candidate discovery stopped at depth={depth}, workers={workers}: {reason:?}");
                };
                assert_eq!(
                    prefixes[0][0].candidates, expected,
                    "depth={depth}, workers={workers}"
                );
                assert!(report.physical().active_workers_high_watermark() <= workers);
            }
        }
    }
}

fn edge(producer: NodeId, scope: Option<PartitionSubscription>) -> DependencyEdge {
    match scope {
        Some(scope) => DependencyEdge::with_partition_scope(producer, Aspect::new(0), scope),
        None => DependencyEdge::new(producer, Aspect::new(0)),
    }
}

// These are non-authoritative lookup inputs. This owner test does not claim a
// publication receipt or bypass the subsequent causal edge admission.
fn candidate_delta(producer: NodeId, scope: PartitionSubscription) -> ProducedAspectDelta {
    ProducedAspectDelta {
        producer,
        output_commit_ordinal: OutputCommitOrdinal(1),
        committed_output_version: AspectVersion::zero().with(Aspect::new(0), 1),
        changes: NonEmptyCanonicalAspectChangeSet::new(vec![ProducedAspectChange {
            aspect: Aspect::new(0),
            previous_version: 0,
            committed_version: 1,
            changed_scopes: PartitionScopeSet::new([scope]),
        }])
        .unwrap(),
        scope_precision: ScopePrecision::ExactAspectScopes,
    }
}

fn oracle(
    graph: &SignalGraph,
    producer: NodeId,
    consumers: &[NodeId],
    query: &PartitionSubscription,
) -> Vec<NodeId> {
    let mut result = consumers
        .iter()
        .copied()
        .filter(|&consumer| {
            graph.dependencies_of(consumer).unwrap().iter().any(|edge| {
                if edge.source() != producer || edge.aspect() != Aspect::new(0) {
                    return false;
                }
                let Some(observed) = edge.scope_ref() else {
                    return true;
                };
                let left = observed.path().segments();
                let right = query.path().segments();
                match (observed.coverage(), query.coverage()) {
                    (ScopeCoverage::Exact, ScopeCoverage::Exact) => left == right,
                    (ScopeCoverage::Subtree, ScopeCoverage::Exact) => right.starts_with(left),
                    (ScopeCoverage::Exact, ScopeCoverage::Subtree) => left.starts_with(right),
                    (ScopeCoverage::Subtree, ScopeCoverage::Subtree) => {
                        left.starts_with(right) || right.starts_with(left)
                    }
                }
            })
        })
        .collect::<Vec<_>>();
    result.sort_unstable();
    result
}
