use std::num::NonZeroUsize;

use worth_execution::{
    Bisection, BisectionDenial, CancellationToken, ComponentPartitioner, ExecutionAuthority,
    ExecutionAuthorityConfig, ExecutionMap, KeyedDenial, KeyedItem, KeyedPartitioner, LeaseRequest,
    MapKernelFailure, MapOutcome, MapPartition, MapStop, PartitionItemId, SourceFactId,
    WeightedEdge, WeightedItem,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

fn item(id: u64) -> PartitionItemId {
    PartitionItemId(id)
}
fn fact(id: u64) -> SourceFactId {
    SourceFactId(id)
}

#[test]
fn keyed_routes_source_facts_and_denies_identity_aliases_without_mutation() {
    let mut keyed = KeyedPartitioner::new();
    let first = KeyedItem {
        item: item(1),
        source_fact: fact(51),
        key: "east",
        partition: PartitionIdentity::new(7),
    };
    assert_eq!(keyed.upsert(first.clone()).unwrap().items_rerouted, 1);
    assert_eq!(keyed.upsert(first.clone()).unwrap().members_visited, 0);
    let fact_only = KeyedItem {
        source_fact: fact(52),
        ..first
    };
    assert_eq!(keyed.upsert(fact_only).unwrap().items_rerouted, 0);
    assert_eq!(keyed.route(item(1)).unwrap().source_fact, fact(52));
    assert_eq!(
        keyed.members(PartitionIdentity::new(7)).unwrap(),
        &std::collections::BTreeSet::from([item(1)])
    );

    let denied = keyed.upsert(KeyedItem {
        item: item(2),
        source_fact: fact(60),
        key: "west",
        partition: PartitionIdentity::new(7),
    });
    assert_eq!(
        denied,
        Err(KeyedDenial::IdentityCollision {
            partition: PartitionIdentity::new(7)
        })
    );
    assert!(keyed.route(item(2)).is_none());
    assert_eq!(keyed.members(PartitionIdentity::new(7)).unwrap().len(), 1);
    assert_eq!(keyed.remove(item(1)).items_rerouted, 1);
    assert!(keyed.members(PartitionIdentity::new(7)).is_none());
}

#[test]
fn candidate_partition_work_is_charged_by_enclosing_map_lease() {
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(1).unwrap(),
        charged_memory_bytes: Some(8_192),
    })
    .unwrap();
    let map = ExecutionMap::try_from_declared_partitions(
        vec![PartitionIdentity::new(20)],
        vec![MapPartition {
            identity: PartitionIdentity::new(20),
            value: 1_u64,
            read_keys: Vec::<u64>::new(),
            write_keys: Vec::<u64>::new(),
            kernel_scratch_bytes: 256,
            max_result_bytes: 0,
        }],
    )
    .unwrap();
    for (ceiling, accepted) in [(1, false), (2, true)] {
        let lease = authority
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Serial,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), 8_192, ceiling),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .unwrap();
        let outcome = map.run(Some(&lease), |value, context| {
            let mut candidate = KeyedPartitioner::new();
            let work = candidate
                .upsert(KeyedItem {
                    item: item(*value),
                    source_fact: fact(40),
                    key: 7_u64,
                    partition: PartitionIdentity::new(7),
                })
                .unwrap();
            work.charge(context)?;
            Ok::<_, MapKernelFailure<()>>(candidate.route(item(*value)).unwrap().partition.value())
        });
        match (accepted, outcome) {
            (true, MapOutcome::Complete { values, report }) => {
                assert_eq!(values, vec![7]);
                assert_eq!(report.charged_work(), 2);
            }
            (
                false,
                MapOutcome::Stopped {
                    reason: MapStop::WorkExhausted { .. },
                    ..
                },
            ) => {}
            (_, other) => panic!(
                "unexpected charged partition outcome: {}",
                other.report().charged_work()
            ),
        }
    }
}

#[test]
fn components_merge_and_split_only_the_affected_island() {
    let mut components = ComponentPartitioner::new();
    for id in [1, 2, 10, 11, 100] {
        components.upsert_item(item(id), fact(id + 1000));
    }
    components.add_edge(item(1), item(2)).unwrap();
    components.add_edge(item(10), item(11)).unwrap();
    let merge = components.add_edge(item(2), item(10)).unwrap();
    assert_eq!(merge.items_rerouted, 2);
    assert_eq!(
        components.route(item(11)).unwrap().partition,
        PartitionIdentity::new(1)
    );
    let split = components.remove_edge(item(2), item(10)).unwrap();
    assert_eq!(split.members_visited, 4);
    assert_eq!(split.items_rerouted, 2);
    assert_eq!(
        components.route(item(10)).unwrap().partition,
        PartitionIdentity::new(10)
    );
    assert_eq!(
        components.route(item(100)).unwrap().partition,
        PartitionIdentity::new(100)
    );
    assert_eq!(
        components
            .add_edge(item(1), item(2))
            .unwrap()
            .members_visited,
        0
    );
    let removal = components.remove_item(item(1));
    assert_eq!(removal.members_visited, 2);
    assert_eq!(
        components.route(item(2)).unwrap().partition,
        PartitionIdentity::new(2)
    );
}

#[test]
fn bisection_retains_cut_paths_and_reports_local_recuts_and_interface_quality() {
    let mut bisection = Bisection::new(2, 2).unwrap();
    for id in 1..=4 {
        bisection
            .upsert_item(WeightedItem {
                item: item(id),
                source_fact: fact(id + 20),
                weight: 1,
            })
            .unwrap();
    }
    let unaffected_path = bisection.route(item(3)).unwrap().partition;
    assert_eq!(unaffected_path, PartitionIdentity::new(3));
    let fact_change = bisection
        .upsert_item(WeightedItem {
            item: item(3),
            source_fact: fact(99),
            weight: 1,
        })
        .unwrap();
    assert_eq!(fact_change.subtrees_recut, 0);
    assert_eq!(fact_change.items_rerouted, 0);
    assert_eq!(bisection.route(item(3)).unwrap().source_fact, fact(99));
    let edge_work = bisection
        .upsert_edge(WeightedEdge {
            a: item(1),
            b: item(3),
            weight: 7,
        })
        .unwrap();
    assert_eq!(edge_work.subtrees_recut, 0);
    assert_eq!(bisection.quality().cut_weight, 7);
    assert!(bisection.cut_interfaces()[&PartitionIdentity::new(1)].contains(&(item(1), item(3))));

    let recut = bisection
        .upsert_item(WeightedItem {
            item: item(5),
            source_fact: fact(25),
            weight: 1,
        })
        .unwrap();
    assert_eq!(recut.subtrees_recut, 1);
    assert_eq!(bisection.route(item(3)).unwrap().partition, unaffected_path);
    assert_eq!(
        bisection
            .leaves()
            .values()
            .map(|members| members.len())
            .sum::<usize>(),
        5
    );
    assert_eq!(bisection.route(item(5)).unwrap().source_fact, fact(25));

    let removal = bisection.remove_item(item(5)).unwrap();
    assert_eq!(removal.subtrees_recut, 0);
    assert_eq!(bisection.route(item(3)).unwrap().partition, unaffected_path);
    assert!(bisection.route(item(5)).is_none());
    assert_eq!(
        bisection
            .leaves()
            .values()
            .map(|members| members.len())
            .sum::<usize>(),
        4
    );
}

#[test]
fn bisection_denies_invalid_edges_before_changing_routes() {
    let mut bisection = Bisection::new(1, 0).unwrap();
    bisection
        .upsert_item(WeightedItem {
            item: item(1),
            source_fact: fact(9),
            weight: 1,
        })
        .unwrap();
    let route = bisection.route(item(1));
    assert_eq!(
        bisection.upsert_edge(WeightedEdge {
            a: item(1),
            b: item(2),
            weight: 1
        }),
        Err(BisectionDenial::UnknownItem(item(2)))
    );
    assert_eq!(
        bisection.upsert_item(WeightedItem {
            item: item(2),
            source_fact: fact(10),
            weight: 0
        }),
        Err(BisectionDenial::ZeroWeight)
    );
    assert_eq!(bisection.route(item(1)), route);
    assert!(bisection.route(item(2)).is_none());
    assert!(bisection.cut_interfaces().is_empty());
}
