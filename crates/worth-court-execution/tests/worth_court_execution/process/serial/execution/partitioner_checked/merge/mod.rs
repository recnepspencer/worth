#[allow(unused_imports)] use crate::partitioner_checked::isolated_context::merge::*;
use {std::sync::Arc, worth_execution::CancellationSource, worth_execution::CancellationToken, worth_execution::ComponentPartitioner, worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapStop, std::sync::Mutex, std::num::NonZeroUsize, worth_foundational::PartitionIdentity, worth_execution::PartitionUpdateDenial, worth_execution::WeightedEdge, crate::partitioner_checked::isolated_context::authority, crate::partitioner_checked::isolated_context::checked_test_guard, crate::partitioner_checked::isolated_context::item, crate::partitioner_checked::isolated_context::lease, crate::partitioner_checked::isolated_context::map, crate::partitioner_checked::isolated_context::weighted_root};

#[test]
fn checked_merge_denial_preserves_both_islands_and_success_moves_only_loser() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = checked_test_guard();
    let authority = authority();
    let map = map();
    let components = std::sync::Arc::new(std::sync::Mutex::new(two_component_islands()));

    let tight = lease(authority, 100_000, 2, worth_execution::CancellationToken::new());
    let outcome = map.run(Some(&tight), |_, context| {
        let denial =
            components
                .lock()
                .unwrap()
                .add_edge_checked(&tight, context, item(2), item(10));
        assert_eq!(
            denial,
            Err(worth_execution::PartitionUpdateDenial::Stop(
                worth_execution::MapKernelStop::WorkCeiling
            ))
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(outcome, worth_execution::MapOutcome::Stopped { .. }));
    assert_eq!(
        components.lock().unwrap().route(item(21)).unwrap(),
        worth_foundational::PartitionIdentity::new(10)
    );

    let memory_tight = lease(authority, 4_096, 10_000, worth_execution::CancellationToken::new());
    let outcome = map.run(Some(&memory_tight), |_, context| {
        let denial =
            components
                .lock()
                .unwrap()
                .add_edge_checked(&memory_tight, context, item(2), item(10));
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
                cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::NestedStopped),
                ..
            },
            ..
        }
    ));
    assert_eq!(
        components.lock().unwrap().route(item(21)).unwrap(),
        worth_foundational::PartitionIdentity::new(10)
    );

    let admitted = lease(authority, 100_000, 100_000, worth_execution::CancellationToken::new());
    let outcome = map.run(Some(&admitted), |_, context| {
        let work = components
            .lock()
            .unwrap()
            .add_edge_checked(&admitted, context, item(2), item(10))
            .unwrap();
        assert_eq!(work.items_rerouted, 12);
        assert_eq!(work.members_visited, 36);
        Ok::<_, MapKernelFailure<()>>(work.items_rerouted)
    });
    match outcome {
        worth_execution::MapOutcome::Complete { report, .. } => {
            assert!(report.physical().peak_charged_memory_bytes() > 0)
        }
        worth_execution::MapOutcome::Stopped { reason, .. } => panic!("checked merge stopped: {reason:?}"),
    }
    assert_eq!(
        components.lock().unwrap().route(item(21)).unwrap(),
        worth_foundational::PartitionIdentity::new(1)
    );
    assert_eq!(
        components.lock().unwrap().route(item(100)).unwrap(),
        worth_foundational::PartitionIdentity::new(100)
    );
}

#[test]
fn checked_item_and_edge_edits_run_under_the_lease() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = checked_test_guard();
    let authority = authority();
    let admitted = lease(authority, 100_000, 100_000, worth_execution::CancellationToken::new());
    let components = std::sync::Arc::new(std::sync::Mutex::new(worth_execution::ComponentPartitioner::new()));
    let bisection = std::sync::Arc::new(std::sync::Mutex::new(weighted_root(2, 10)));
    let outcome = map().run(Some(&admitted), |_, context| {
        let mut components = components.lock().unwrap();
        let work = components
            .upsert_item_checked(&admitted, context, item(7))
            .unwrap();
        assert_eq!(work.items_rerouted, 1);
        let mut bisection = bisection.lock().unwrap();
        assert_eq!(
            bisection
                .upsert_edge_checked(
                    &admitted,
                    context,
                    WeightedEdge {
                        a: item(1),
                        b: item(2),
                        weight: 3
                    },
                )
                .unwrap()
                .edges_visited,
            1
        );
        assert_eq!(
            bisection
                .remove_edge_checked(&admitted, context, item(1), item(2))
                .unwrap()
                .edges_visited,
            1
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(outcome, worth_execution::MapOutcome::Complete { .. }));
    assert_eq!(
        components.lock().unwrap().route(item(7)),
        Some(worth_foundational::PartitionIdentity::new(7))
    );
}

#[test]
fn unrelated_and_cancelled_child_leases_cannot_edit_retained_routes() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = checked_test_guard();
    let authority = authority();
    let parent = lease(authority, 100_000, 100_000, worth_execution::CancellationToken::new());
    let unrelated = lease(authority, 100_000, 100_000, worth_execution::CancellationToken::new());
    let components = std::sync::Arc::new(std::sync::Mutex::new(worth_execution::ComponentPartitioner::new()));
    let outcome = map().run(Some(&parent), |_, context| {
        assert_eq!(
            components
                .lock()
                .unwrap()
                .upsert_item_checked(&unrelated, context, item(1)),
            Err(worth_execution::PartitionUpdateDenial::Admission(
                worth_execution::LeaseDenial::UnrelatedNestedLease
            ))
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(outcome, worth_execution::MapOutcome::Stopped { .. }));
    assert!(components.lock().unwrap().route(item(1)).is_none());

    let cancellation = worth_execution::CancellationSource::new();
    let child = parent
        .child(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), Some(100_000), Some(100_000)),
            ),
            deadline: None,
            cancellation: cancellation.token(),
        })
        .unwrap();
    cancellation.cancel();
    let outcome = map().run(Some(&parent), |_, context| {
        assert_eq!(
            components
                .lock()
                .unwrap()
                .upsert_item_checked(&child, context, item(2)),
            Err(worth_execution::PartitionUpdateDenial::Stop(
                worth_execution::MapKernelStop::Cancelled
            ))
        );
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(outcome, worth_execution::MapOutcome::Stopped { .. }));
    assert!(components.lock().unwrap().route(item(2)).is_none());
}

#[test]
fn persistent_graph_charge_stops_growth_and_shrinks_after_removal() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = checked_test_guard();
    let authority = authority();
    let tight = lease(authority, 4_096, 100_000, worth_execution::CancellationToken::new());
    let components = std::sync::Arc::new(std::sync::Mutex::new(worth_execution::ComponentPartitioner::new()));
    let outcome = map().run(Some(&tight), |_, context| {
        let mut count = 0_u64;
        for id in 1..=1_000 {
            match components
                .lock()
                .unwrap()
                .upsert_item_checked(&tight, context, item(id))
            {
                Ok(_) => count += 1,
                Err(worth_execution::PartitionUpdateDenial::Admission(
                    worth_execution::LeaseDenial::MemoryExhausted(_),
                )) => break,
                Err(other) => panic!("unexpected checked denial: {other:?}"),
            }
        }
        assert!(count > 0 && count < 1_000);
        assert!(components.lock().unwrap().route(item(1_000)).is_none());
        Ok::<_, MapKernelFailure<()>>(count)
    });
    assert!(matches!(outcome, worth_execution::MapOutcome::Stopped { .. }));

    let roomy = lease(authority, 100_000, 100_000, worth_execution::CancellationToken::new());
    let outcome = map().run(Some(&roomy), |_, context| {
        let mut components = components.lock().unwrap();
        for id in 1..=6 {
            components
                .remove_item_checked(&roomy, context, item(id))
                .unwrap();
        }
        for id in 20..=30 {
            components
                .upsert_item_checked(&roomy, context, item(id))
                .unwrap();
        }
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(outcome, worth_execution::MapOutcome::Complete { .. }));
    assert!(components.lock().unwrap().route(item(30)).is_some());
}

#[test]
fn report_peak_includes_two_simultaneously_retained_graphs() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = checked_test_guard();
    let authority = authority();
    let admitted = lease(authority, 100_000, 100_000, worth_execution::CancellationToken::new());
    let first = std::sync::Arc::new(std::sync::Mutex::new(worth_execution::ComponentPartitioner::new()));
    let second = std::sync::Arc::new(std::sync::Mutex::new(worth_execution::ComponentPartitioner::new()));
    let outcome = map().run(Some(&admitted), |_, context| {
        for id in 1..=8 {
            first
                .lock()
                .unwrap()
                .upsert_item_checked(&admitted, context, item(id))
                .unwrap();
            second
                .lock()
                .unwrap()
                .upsert_item_checked(&admitted, context, item(id + 100))
                .unwrap();
        }
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    match outcome {
        worth_execution::MapOutcome::Complete { report, .. } => {
            // Each eight-item graph retains at least 8 * 384 charged bytes.
            assert!(report.physical().peak_charged_memory_bytes() >= 2 * 8 * 384);
        }
        worth_execution::MapOutcome::Stopped { reason, .. } => panic!("two retained graphs stopped: {reason:?}"),
    }
}

#[test]
fn pure_component_mutation_cannot_bypass_a_live_kernel_or_owned_charge() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = checked_test_guard();
    let authority = authority();
    let admitted = lease(authority, 100_000, 100_000, worth_execution::CancellationToken::new());
    let components = std::sync::Arc::new(std::sync::Mutex::new(worth_execution::ComponentPartitioner::new()));
    let outcome = map().run(Some(&admitted), |_, context| {
        let mut graph = components.lock().unwrap();
        let bypass = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            graph.upsert_item(item(1));
        }));
        assert!(bypass.is_err());
        assert!(graph.route(item(1)).is_none());
        graph
            .upsert_item_checked(&admitted, context, item(1))
            .unwrap();
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(outcome, worth_execution::MapOutcome::Complete { .. }));
    let mut graph = components.lock().unwrap();
    let bypass = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        graph.upsert_item(item(2));
    }));
    assert!(bypass.is_err());
    assert!(graph.route(item(2)).is_none());
}
