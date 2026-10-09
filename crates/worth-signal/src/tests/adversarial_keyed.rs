use std::sync::atomic::{AtomicU32, Ordering};

use crate::facade::*;
use crate::tests::support::{define_keyed_computation, version_ab, ASPECT_A};

#[test]
fn many_families_sharing_same_key_remain_distinct() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let families: Vec<_> = (0..64)
        .map(|index| define_keyed_computation(&mut runtime, format!("family-{index}"), ()))
        .collect();

    let mut seen = std::collections::BTreeSet::new();
    for family in &families {
        let node = family.keyed("shared-key").node(&mut runtime);
        assert!(seen.insert(node));
    }
}

#[test]
fn many_keys_in_one_family_reuse_stably_across_large_lookup_sequences() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let family = define_keyed_computation(&mut runtime, "airframe", ());
    let mut first_pass = Vec::new();

    for index in 0..256 {
        first_pass.push(
            family
                .keyed(format!("component-{index}"))
                .node(&mut runtime),
        );
    }

    for (index, expected) in first_pass.iter().enumerate() {
        let reused = family
            .keyed(format!("component-{index}"))
            .node(&mut runtime);
        assert_eq!(&reused, expected);
    }
}

#[test]
fn repeated_failed_transactions_do_not_promote_memoized_results() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let family = define_keyed_computation(&mut runtime, "fintech-pricing", ());
    let keyed = family.keyed("book");
    let node = keyed.node(&mut runtime);
    let computation = keyed.memoized("session-a");
    let mut runtime_ctx = ();
    let compute_calls = AtomicU32::new(0);

    for _ in 0..20 {
        let err = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.evaluate_keyed(node, &computation, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Err(SignalError::invalid_input("rollback memo write"))
        });
        assert!(err.is_err());
    }

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.evaluate_keyed(node, &computation, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("committed-artifact"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            tx.evaluate_keyed(node, &computation, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(99, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let metrics = runtime.observe().metrics();
    assert_eq!(metrics.evaluation.memoization_misses, 21);
    assert_eq!(metrics.evaluation.memoization_hits, 1);
    assert_eq!(compute_calls.load(Ordering::Relaxed), 21);
}

#[test]
fn keyed_telemetry_stays_coherent_under_mixed_hit_and_miss_workload() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let family = define_keyed_computation(&mut runtime, "kernel", ());
    let mut runtime_ctx = ();

    for index in 0..24 {
        let keyed = family.keyed(format!("part-{index}"));
        let node = keyed.node(&mut runtime);
        let computation = keyed.memoized("shape");

        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.evaluate_keyed(node, &computation, &|view| {
                    Ok(view.finish(NodeEvaluationResult::from_version(version_ab(index + 1, 0))))
                })?;
                Ok(())
            })
            .unwrap();

        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.mark_dirty(node, ASPECT_A)?;
                tx.evaluate_keyed(node, &computation, &|view| {
                    Ok(view.finish(NodeEvaluationResult::from_version(version_ab(999, 0))))
                })?;
                Ok(())
            })
            .unwrap();
    }

    let metrics = runtime.observe().metrics();
    assert_eq!(metrics.invalidation.keyed_evaluation_count, 48);
    assert_eq!(metrics.evaluation.memoization_misses, 24);
    assert_eq!(metrics.evaluation.memoization_hits, 24);
}

#[test]
#[ignore = "stress coverage for keyed-cardinality and memoization churn"]
fn stress_thousands_of_keyed_lookups_and_memo_keys() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let families: Vec<_> = (0..64)
        .map(|index| define_keyed_computation(&mut runtime, format!("family-{index}"), ()))
        .collect();
    let mut runtime_ctx = ();

    for round in 0..8 {
        for family in &families {
            for key_index in 0..128 {
                let key = format!("key-{key_index}");
                let keyed = family.keyed(key);
                let node = keyed.node(&mut runtime);
                let computation = keyed.memoized(format!("memo-{round}-{key_index}"));
                runtime
                    .transaction(request_execution, &mut runtime_ctx, |tx| {
                        tx.evaluate_keyed(node, &computation, &|view| {
                            Ok(view.finish(NodeEvaluationResult::from_version(version_ab(
                                round + key_index as u64 + 1,
                                0,
                            ))))
                        })?;
                        Ok(())
                    })
                    .unwrap();
            }
        }
    }

    assert!(
        runtime
            .observe()
            .metrics()
            .invalidation
            .keyed_evaluation_count
            > 1_000
    );
}
