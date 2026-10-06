use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex, OnceLock},
};

use worth_execution::{
    Bisection, CancellationSource, CancellationToken, ComponentPartitioner, ExecutionAuthority,
    ExecutionAuthorityConfig, ExecutionMap, LeaseRequest, MapKernelFailure, MapOutcome,
    MapPartition, MapStop, PartitionItemId, PartitionUpdateDenial, SourceFactId, WeightedEdge,
    WeightedItem,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

fn item(id: u64) -> PartitionItemId {
    PartitionItemId(id)
}
fn fact(id: u64) -> SourceFactId {
    SourceFactId(id + 100)
}

static CHECKED_TEST_LOCK: Mutex<()> = Mutex::new(());

fn checked_test_guard() -> std::sync::MutexGuard<'static, ()> {
    CHECKED_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

fn authority() -> &'static ExecutionAuthority {
    static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: 10_000_000,
        })
        .unwrap()
    })
}

fn map() -> ExecutionMap<u64, u64> {
    ExecutionMap::try_from_declared_partitions(
        vec![PartitionIdentity::new(1)],
        vec![MapPartition {
            identity: PartitionIdentity::new(1),
            value: 1,
            read_keys: vec![],
            write_keys: vec![],
            kernel_scratch_bytes: 512,
            max_result_bytes: 0,
        }],
    )
    .unwrap()
}

fn lease<'a>(
    authority: &'a ExecutionAuthority,
    memory: u64,
    ceiling: u64,
    cancellation: CancellationToken,
) -> worth_execution::ExecutionResourceLease<'a> {
    authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), memory, ceiling),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}

fn component_chain(count: u64) -> ComponentPartitioner {
    let mut components = ComponentPartitioner::new();
    for id in 1..=count {
        components.upsert_item(item(id), fact(id));
    }
    for id in 1..count {
        components.add_edge(item(id), item(id + 1)).unwrap();
    }
    components
}

#[path = "partitioner_checked/merge.rs"]
mod merge;

fn weighted_root(count: u64, max_leaf_weight: u64) -> Bisection {
    let mut bisection = Bisection::new(max_leaf_weight, 2).unwrap();
    for id in 1..=count {
        bisection
            .upsert_item(WeightedItem {
                item: item(id),
                source_fact: fact(id),
                weight: 1,
            })
            .unwrap();
    }
    bisection
}

#[test]
fn checked_split_and_recut_denials_preserve_retained_topology() {
    let _serial = checked_test_guard();
    let authority = authority();
    let map = map();

    let components = Arc::new(Mutex::new(component_chain(4)));
    let tight = lease(authority, 100_000, 2, CancellationToken::new());
    let outcome = map.run(Some(&tight), |_, context| {
        let denial =
            components
                .lock()
                .unwrap()
                .remove_edge_checked(&tight, context, item(2), item(3));
        assert_eq!(
            denial,
            Err(PartitionUpdateDenial::Stop(
                worth_execution::MapKernelStop::WorkCeiling
            ))
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(outcome, MapOutcome::Stopped { .. }));
    assert_eq!(
        components.lock().unwrap().route(item(4)).unwrap().partition,
        PartitionIdentity::new(1)
    );

    let components = Arc::new(Mutex::new(component_chain(16)));
    let memory_tight = lease(authority, 8_192, 10_000, CancellationToken::new());
    let outcome = map.run(Some(&memory_tight), |_, context| {
        let denial = components.lock().unwrap().remove_edge_checked(
            &memory_tight,
            context,
            item(8),
            item(9),
        );
        assert!(
            matches!(
                denial,
                Err(PartitionUpdateDenial::Admission(
                    worth_execution::LeaseDenial::MemoryExhausted(_)
                ))
            ),
            "{denial:?}"
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(
        outcome,
        MapOutcome::Stopped {
            reason: MapStop::Failure {
                cause: MapKernelFailure::Stop(worth_execution::MapKernelStop::NestedStopped),
                ..
            },
            ..
        }
    ));
    assert_eq!(
        components
            .lock()
            .unwrap()
            .route(item(16))
            .unwrap()
            .partition,
        PartitionIdentity::new(1)
    );

    let cancellation = CancellationSource::new();
    let cancelled = lease(authority, 100_000, 10_000, cancellation.token());
    let outcome = map.run(Some(&cancelled), |_, context| {
        cancellation.cancel();
        let denial = components
            .lock()
            .unwrap()
            .remove_item_checked(&cancelled, context, item(1));
        assert_eq!(
            denial,
            Err(PartitionUpdateDenial::Stop(
                worth_execution::MapKernelStop::Cancelled
            ))
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(outcome, MapOutcome::Stopped { .. }));
    assert_eq!(
        components.lock().unwrap().route(item(1)).unwrap().partition,
        PartitionIdentity::new(1)
    );

    let bisection = Arc::new(Mutex::new(weighted_root(4, 4)));
    let tight = lease(authority, 100_000, 2, CancellationToken::new());
    let outcome = map.run(Some(&tight), |_, context| {
        let denial = bisection.lock().unwrap().upsert_item_checked(
            &tight,
            context,
            WeightedItem {
                item: item(5),
                source_fact: fact(5),
                weight: 1,
            },
        );
        assert_eq!(
            denial,
            Err(PartitionUpdateDenial::Stop(
                worth_execution::MapKernelStop::WorkCeiling
            ))
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(outcome, MapOutcome::Stopped { .. }));
    assert!(bisection.lock().unwrap().route(item(5)).is_none());

    let admitted = lease(authority, 100_000, 100_000, CancellationToken::new());
    let outcome = map.run(Some(&admitted), |_, context| {
        let work = components
            .lock()
            .unwrap()
            .remove_edge_checked(&admitted, context, item(8), item(9))
            .unwrap();
        assert_eq!(work.items_rerouted, 8);
        Ok::<_, MapKernelFailure<()>>(work.edges_visited)
    });
    match outcome {
        MapOutcome::Complete { report, .. } => {
            assert!(report.charged_work() > 0);
            assert!(report.physical().peak_charged_memory_bytes() > 0);
        }
        MapOutcome::Stopped { reason, .. } => panic!("checked split stopped: {reason:?}"),
    }
    assert_eq!(
        components
            .lock()
            .unwrap()
            .route(item(16))
            .unwrap()
            .partition,
        PartitionIdentity::new(9)
    );
    assert_eq!(bisection.lock().unwrap().leaves().len(), 1);

    let memory_tight = lease(authority, 4_096, 10_000, CancellationToken::new());
    let outcome = map.run(Some(&memory_tight), |_, context| {
        let denial = bisection.lock().unwrap().upsert_item_checked(
            &memory_tight,
            context,
            WeightedItem {
                item: item(5),
                source_fact: fact(5),
                weight: 1,
            },
        );
        assert!(
            matches!(
                denial,
                Err(PartitionUpdateDenial::Admission(
                    worth_execution::LeaseDenial::MemoryExhausted(_)
                ))
            ),
            "{denial:?}"
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(
        outcome,
        MapOutcome::Stopped {
            reason: MapStop::Failure {
                cause: MapKernelFailure::Stop(worth_execution::MapKernelStop::NestedStopped),
                ..
            },
            ..
        }
    ));
    assert!(bisection.lock().unwrap().route(item(5)).is_none());
}

#[test]
fn checked_local_recut_ignores_edges_inside_unrelated_sibling() {
    let _serial = checked_test_guard();
    let authority = authority();
    let map = map();
    let plain = Arc::new(Mutex::new(weighted_root(4, 2)));
    let with_sibling_edge = Arc::new(Mutex::new(weighted_root(4, 2)));
    assert_eq!(
        with_sibling_edge.lock().unwrap().leaves()[&PartitionIdentity::new(3)],
        std::collections::BTreeSet::from([item(2), item(3)])
    );
    with_sibling_edge
        .lock()
        .unwrap()
        .upsert_edge(WeightedEdge {
            a: item(2),
            b: item(3),
            weight: 99,
        })
        .unwrap();
    let lease = lease(authority, 200_000, 100_000, CancellationToken::new());
    let outcome = map.run(Some(&lease), |_, context| {
        let next = WeightedItem {
            item: item(5),
            source_fact: fact(5),
            weight: 1,
        };
        let plain_work = plain
            .lock()
            .unwrap()
            .upsert_item_checked(&lease, context, next)
            .unwrap();
        let sibling_work = with_sibling_edge
            .lock()
            .unwrap()
            .upsert_item_checked(&lease, context, next)
            .unwrap();
        assert_eq!(plain_work.subtrees_recut, 1);
        assert_eq!(sibling_work.subtrees_recut, 1);
        assert_eq!(plain_work.edges_visited, sibling_work.edges_visited);
        Ok::<_, MapKernelFailure<()>>(plain_work.edges_visited)
    });
    match outcome {
        MapOutcome::Complete { report, .. } => {
            assert!(report.physical().peak_charged_memory_bytes() > 0);
        }
        MapOutcome::Stopped { reason, .. } => panic!("local recut stopped: {reason:?}"),
    }
    assert_eq!(
        plain.lock().unwrap().route(item(3)),
        with_sibling_edge.lock().unwrap().route(item(3))
    );
}
