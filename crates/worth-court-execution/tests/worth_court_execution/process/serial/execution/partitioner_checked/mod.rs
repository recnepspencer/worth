#[allow(unused_imports)]
use crate::partitioner_checked::isolated_context::*;
use {std::num::NonZeroUsize, std::sync::Arc, std::sync::Mutex, std::sync::OnceLock};

use {
    worth_execution::Bisection, worth_execution::CancellationSource,
    worth_execution::CancellationToken, worth_execution::ComponentPartitioner,
    worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig,
    worth_execution::ExecutionMap, worth_execution::LeaseRequest,
    worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition,
    worth_execution::MapStop, worth_execution::PartitionItemId,
    worth_execution::PartitionUpdateDenial, worth_execution::WeightedEdge,
    worth_execution::WeightedItem,
};
use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    worth_foundational::PartitionIdentity,
};

#[test]
fn checked_split_and_recut_denials_preserve_retained_topology() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = checked_test_guard();
    let authority = authority();
    let map = map();

    let components = std::sync::Arc::new(std::sync::Mutex::new(component_chain(4)));
    let tight = lease(
        authority,
        100_000,
        2,
        worth_execution::CancellationToken::new(),
    );
    let outcome = map.run(Some(&tight), |_, context| {
        let denial =
            components
                .lock()
                .unwrap()
                .remove_edge_checked(&tight, context, item(2), item(3));
        assert_eq!(
            denial,
            Err(worth_execution::PartitionUpdateDenial::Stop(
                worth_execution::MapKernelStop::WorkCeiling
            ))
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(
        outcome,
        worth_execution::MapOutcome::Stopped { .. }
    ));
    assert_eq!(
        components.lock().unwrap().route(item(4)).unwrap(),
        worth_foundational::PartitionIdentity::new(1)
    );

    let components = std::sync::Arc::new(std::sync::Mutex::new(component_chain(16)));
    let memory_tight = lease(
        authority,
        8_192,
        10_000,
        worth_execution::CancellationToken::new(),
    );
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
                Err(worth_execution::PartitionUpdateDenial::Admission(
                    worth_execution::LeaseDenial::MemoryExhausted(_)
                ))
            ),
            "{denial:?}"
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(
        outcome,
        worth_execution::MapOutcome::Stopped {
            reason: worth_execution::MapStop::Failure {
                cause: worth_execution::MapKernelFailure::Stop(
                    worth_execution::MapKernelStop::NestedStopped
                ),
                ..
            },
            ..
        }
    ));
    assert_eq!(
        components.lock().unwrap().route(item(16)).unwrap(),
        worth_foundational::PartitionIdentity::new(1)
    );

    let cancellation = worth_execution::CancellationSource::new();
    let cancelled = lease(authority, 100_000, 10_000, cancellation.token());
    let outcome = map.run(Some(&cancelled), |_, context| {
        cancellation.cancel();
        let denial = components
            .lock()
            .unwrap()
            .remove_item_checked(&cancelled, context, item(1));
        assert_eq!(
            denial,
            Err(worth_execution::PartitionUpdateDenial::Stop(
                worth_execution::MapKernelStop::Cancelled
            ))
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(
        outcome,
        worth_execution::MapOutcome::Stopped { .. }
    ));
    assert_eq!(
        components.lock().unwrap().route(item(1)).unwrap(),
        worth_foundational::PartitionIdentity::new(1)
    );

    let bisection = std::sync::Arc::new(std::sync::Mutex::new(weighted_root(4, 4)));
    let tight = lease(
        authority,
        100_000,
        2,
        worth_execution::CancellationToken::new(),
    );
    let outcome = map.run(Some(&tight), |_, context| {
        let denial = bisection.lock().unwrap().upsert_item_checked(
            &tight,
            context,
            WeightedItem {
                item: item(5),
                weight: 1,
            },
        );
        assert_eq!(
            denial,
            Err(worth_execution::PartitionUpdateDenial::Stop(
                worth_execution::MapKernelStop::WorkCeiling
            ))
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(
        outcome,
        worth_execution::MapOutcome::Stopped { .. }
    ));
    assert!(bisection.lock().unwrap().route(item(5)).is_none());

    let admitted = lease(
        authority,
        100_000,
        100_000,
        worth_execution::CancellationToken::new(),
    );
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
        worth_execution::MapOutcome::Complete { report, .. } => {
            assert!(report.charged_work() > 0);
            assert!(report.physical().peak_charged_memory_bytes() > 0);
        }
        worth_execution::MapOutcome::Stopped { reason, .. } => {
            panic!("checked split stopped: {reason:?}")
        }
    }
    assert_eq!(
        components.lock().unwrap().route(item(16)).unwrap(),
        worth_foundational::PartitionIdentity::new(9)
    );
    assert_eq!(bisection.lock().unwrap().leaves().len(), 1);

    let memory_tight = lease(
        authority,
        4_096,
        10_000,
        worth_execution::CancellationToken::new(),
    );
    let outcome = map.run(Some(&memory_tight), |_, context| {
        let denial = bisection.lock().unwrap().upsert_item_checked(
            &memory_tight,
            context,
            WeightedItem {
                item: item(5),
                weight: 1,
            },
        );
        assert!(
            matches!(
                denial,
                Err(worth_execution::PartitionUpdateDenial::Admission(
                    worth_execution::LeaseDenial::MemoryExhausted(_)
                ))
            ),
            "{denial:?}"
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(
        outcome,
        worth_execution::MapOutcome::Stopped {
            reason: worth_execution::MapStop::Failure {
                cause: worth_execution::MapKernelFailure::Stop(
                    worth_execution::MapKernelStop::NestedStopped
                ),
                ..
            },
            ..
        }
    ));
    assert!(bisection.lock().unwrap().route(item(5)).is_none());
}

#[test]
fn checked_local_recut_ignores_edges_inside_unrelated_sibling() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = checked_test_guard();
    let authority = authority();
    let map = map();
    let plain = std::sync::Arc::new(std::sync::Mutex::new(weighted_root(4, 2)));
    let with_sibling_edge = std::sync::Arc::new(std::sync::Mutex::new(weighted_root(4, 2)));
    assert_eq!(
        with_sibling_edge.lock().unwrap().leaves()[&worth_foundational::PartitionIdentity::new(3)],
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
    let lease = lease(
        authority,
        200_000,
        100_000,
        worth_execution::CancellationToken::new(),
    );
    let outcome = map.run(Some(&lease), |_, context| {
        let next = WeightedItem {
            item: item(5),
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
        worth_execution::MapOutcome::Complete { report, .. } => {
            assert!(report.physical().peak_charged_memory_bytes() > 0);
        }
        worth_execution::MapOutcome::Stopped { reason, .. } => {
            panic!("local recut stopped: {reason:?}")
        }
    }
    assert_eq!(
        plain.lock().unwrap().route(item(3)),
        with_sibling_edge.lock().unwrap().route(item(3))
    );
}
#[path = "merge/mod.rs"]
pub(crate) mod merge;
