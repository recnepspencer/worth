use std::num::NonZeroUsize;

use worth_execution::{
    Bisection, BisectionDenial, CancellationToken, ChargedBytes, ComponentPartitioner,
    ExecutionAuthority, ExecutionAuthorityConfig, ExecutionMap, KeyedDenial, KeyedEditDenial,
    KeyedItem, KeyedPartitioner, LeaseRequest, MapKernelFailure, MapOutcome, MapPartition, MapStop,
    PartitionItemId, WeightedEdge, WeightedItem,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

fn item(id: u64) -> PartitionItemId {
    PartitionItemId(id)
}
fn admit(_: u64) -> Result<(), ()> {
    Ok(())
}

#[test]
fn keyed_routes_items_and_denies_identity_aliases_without_mutation() {
    let mut keyed = KeyedPartitioner::new();
    let first = KeyedItem {
        item: item(1),
        key: "east",
        partition: PartitionIdentity::new(7),
    };
    assert_eq!(
        keyed.upsert(first.clone(), admit).unwrap().items_rerouted,
        1
    );
    assert_eq!(
        keyed.upsert(first.clone(), admit).unwrap().members_visited,
        0
    );
    assert_eq!(keyed.route(item(1)), Some(PartitionIdentity::new(7)));
    assert_eq!(
        keyed.members(PartitionIdentity::new(7)).unwrap(),
        &std::collections::BTreeSet::from([item(1)])
    );

    let denied = keyed.upsert(
        KeyedItem {
            item: item(2),
            key: "west",
            partition: PartitionIdentity::new(7),
        },
        admit,
    );
    assert_eq!(
        denied,
        Err(KeyedEditDenial::Keyed(KeyedDenial::IdentityCollision {
            partition: PartitionIdentity::new(7)
        }))
    );
    assert!(keyed.route(item(2)).is_none());
    assert_eq!(keyed.members(PartitionIdentity::new(7)).unwrap().len(), 1);
    assert_eq!(keyed.remove(item(1)).items_rerouted, 1);
    assert!(keyed.members(PartitionIdentity::new(7)).is_none());
}

#[test]
fn keyed_offers_the_retained_bound_before_an_edit_and_a_refusal_changes_nothing() {
    let mut keyed = KeyedPartitioner::new();
    let first = KeyedItem {
        item: item(1),
        key: "east",
        partition: PartitionIdentity::new(7),
    };
    let one = KeyedPartitioner::<&str>::retained_bytes(1, 1, 0).unwrap();
    let mut offered = 0;
    keyed
        .upsert(first.clone(), |bound| {
            offered = bound;
            admit(bound)
        })
        .unwrap();
    assert_eq!(offered, one);
    let second = KeyedItem {
        item: item(2),
        key: "west",
        partition: PartitionIdentity::new(8),
    };
    let two = KeyedPartitioner::<&str>::retained_bytes(2, 2, 0).unwrap();
    assert!(two > one);
    assert_eq!(
        keyed.upsert(second, |bound| if bound > one {
            Err(bound)
        } else {
            Ok(())
        }),
        Err(KeyedEditDenial::Admission(two))
    );
    assert!(keyed.route(item(2)).is_none());
    assert!(keyed.members(PartitionIdentity::new(8)).is_none());
}

/// A key that owns 100 bytes of heap.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct OwnsHeap(&'static str);

impl ChargedBytes for OwnsHeap {
    fn additional_charged_bytes(&self) -> u64 {
        100
    }
}

/// Each retained copy of a key is charged its heap: one on its item, two on
/// a new key's group. A removal releases what it retained.
#[test]
fn keyed_charges_the_heap_each_retained_key_owns() {
    let entry = |id, key| KeyedItem {
        item: item(id),
        key: OwnsHeap(key),
        partition: PartitionIdentity::new(if key == "east" { 7 } else { 8 }),
    };
    let bound = |items, keys, heap: u64| {
        KeyedPartitioner::<OwnsHeap>::retained_bytes(items, keys, 0).unwrap() + heap
    };
    let mut keyed = KeyedPartitioner::new();
    let mut offered = Vec::new();
    for (id, key) in [(1, "east"), (2, "east"), (3, "west")] {
        keyed
            .upsert(entry(id, key), |offer| {
                offered.push(offer);
                admit(offer)
            })
            .unwrap();
    }
    assert_eq!(
        offered,
        [bound(1, 1, 300), bound(2, 1, 400), bound(3, 2, 700)]
    );
    keyed.remove(item(3));
    keyed.remove(item(2));
    keyed
        .upsert(entry(2, "east"), |offer| {
            offered.push(offer);
            admit(offer)
        })
        .unwrap();
    assert_eq!(offered.last().copied(), Some(bound(2, 1, 400)));
}

/// A kept copy is the partitioner that upserting only the kept items
/// leaves, offered first at what that partitioner charges; a refusal builds
/// nothing.
#[test]
fn keyed_keeps_a_copy_of_the_named_items_at_their_own_bound() {
    let entry = |id, key| KeyedItem {
        item: item(id),
        key: OwnsHeap(key),
        partition: PartitionIdentity::new(if key == "east" { 7 } else { 8 }),
    };
    let mut keyed = KeyedPartitioner::new();
    let mut fresh = KeyedPartitioner::new();
    for (id, key) in [(1, "east"), (2, "west"), (3, "east"), (4, "west")] {
        keyed.upsert(entry(id, key), admit).unwrap();
        if id != 2 {
            fresh.upsert(entry(id, key), admit).unwrap();
        }
    }
    assert_eq!(
        keyed.kept(|item| item.0 != 2, |_| Err("refused")).err(),
        Some(KeyedEditDenial::Admission("refused"))
    );
    let mut offered = 0;
    let kept = keyed
        .kept(
            |item| item.0 != 2,
            |bound| {
                offered = bound;
                admit(bound)
            },
        )
        .unwrap();
    assert_eq!(Some(offered), fresh.charged_bytes());
    assert_eq!(kept.charged_bytes(), fresh.charged_bytes());
    assert_eq!(
        kept.partitions().collect::<Vec<_>>(),
        fresh.partitions().collect::<Vec<_>>()
    );
    for id in 1..=4 {
        assert_eq!(kept.route(item(id)), fresh.route(item(id)));
    }
    for partition in [7, 8] {
        let partition = PartitionIdentity::new(partition);
        assert_eq!(kept.members(partition), fresh.members(partition));
    }
    let none = keyed.kept(|_| false, admit).unwrap();
    assert_eq!(none.partitions().count(), 0);
    assert_eq!(none.charged_bytes(), Some(0));
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
                .upsert(
                    KeyedItem {
                        item: item(*value),
                        key: 7_u64,
                        partition: PartitionIdentity::new(7),
                    },
                    admit,
                )
                .unwrap();
            work.charge(context)?;
            Ok::<_, MapKernelFailure<()>>(candidate.route(item(*value)).unwrap().value())
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
        components.upsert_item(item(id));
    }
    components.add_edge(item(1), item(2)).unwrap();
    components.add_edge(item(10), item(11)).unwrap();
    let merge = components.add_edge(item(2), item(10)).unwrap();
    assert_eq!(merge.items_rerouted, 2);
    assert_eq!(
        components.route(item(11)).unwrap(),
        PartitionIdentity::new(1)
    );
    let split = components.remove_edge(item(2), item(10)).unwrap();
    assert_eq!(split.members_visited, 4);
    assert_eq!(split.items_rerouted, 2);
    assert_eq!(
        components.route(item(10)).unwrap(),
        PartitionIdentity::new(10)
    );
    assert_eq!(
        components.route(item(100)).unwrap(),
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
        components.route(item(2)).unwrap(),
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
                weight: 1,
            })
            .unwrap();
    }
    let unaffected_path = bisection.route(item(3)).unwrap();
    assert_eq!(unaffected_path, PartitionIdentity::new(3));
    let unchanged = bisection
        .upsert_item(WeightedItem {
            item: item(3),
            weight: 1,
        })
        .unwrap();
    assert_eq!(unchanged, worth_execution::PartitionWork::default());
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
            weight: 1,
        })
        .unwrap();
    assert_eq!(recut.subtrees_recut, 1);
    assert_eq!(bisection.route(item(3)).unwrap(), unaffected_path);
    assert_eq!(
        bisection
            .leaves()
            .values()
            .map(|members| members.len())
            .sum::<usize>(),
        5
    );
    assert!(bisection.route(item(5)).is_some());

    let removal = bisection.remove_item(item(5)).unwrap();
    assert_eq!(removal.subtrees_recut, 0);
    assert_eq!(bisection.route(item(3)).unwrap(), unaffected_path);
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
            weight: 0
        }),
        Err(BisectionDenial::ZeroWeight)
    );
    assert_eq!(bisection.route(item(1)), route);
    assert!(bisection.route(item(2)).is_none());
    assert!(bisection.cut_interfaces().is_empty());
}

#[path = "partitioners/fresh_route.rs"]
mod fresh_route;
