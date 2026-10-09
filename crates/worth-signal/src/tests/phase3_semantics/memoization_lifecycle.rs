use crate::facade::{
    ComputationFamily, KeyedComputation, NodeEvaluationResult, SignalError, SignalGraph,
    SignalRuntime,
};
use crate::tests::support::{define_keyed_computation, version_ab};
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn memoization_is_scoped_by_family() {
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
    let family_a = define_keyed_computation(&mut runtime, "projection-a", ());
    let family_b = define_keyed_computation(&mut runtime, "projection-b", ());
    let keyed_a = family_a.keyed("bulkhead");
    let keyed_b = family_b.keyed("bulkhead");
    let node_a = keyed_a.node(&mut runtime);
    let node_b = keyed_b.node(&mut runtime);
    let computation_a = keyed_a.memoized("shape-v1");
    let computation_b = keyed_b.memoized("shape-v1");
    let mut runtime_ctx = ();
    let compute_calls = AtomicU32::new(0);

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.evaluate_keyed(node_a, &computation_a, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0)).with_output_identity("a"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.evaluate_keyed(node_b, &computation_b, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0)).with_output_identity("b"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    assert_eq!(compute_calls.load(Ordering::Relaxed), 2);
}

#[test]
fn memoization_write_is_discarded_on_rollback() {
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
    let family = define_keyed_computation(&mut runtime, "projection", ());
    let keyed = family.keyed("bulkhead");
    let node = keyed.node(&mut runtime);
    let computation = keyed.memoized("shape-v1");
    let mut runtime_ctx = ();
    let compute_calls = AtomicU32::new(0);

    let err = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
        tx.evaluate_keyed(node, &computation, &|view| {
            compute_calls.fetch_add(1, Ordering::Relaxed);
            Ok(view.finish(
                NodeEvaluationResult::from_version(version_ab(1, 0)).with_output_identity("cached"),
            ))
        })?;
        Err(SignalError::invalid_input("force rollback"))
    });
    assert!(err.is_err());

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.evaluate_keyed(node, &computation, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("fresh"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    assert_eq!(compute_calls.load(Ordering::Relaxed), 2);
    let metrics = runtime.observe().metrics();
    assert_eq!(metrics.evaluation.memoization_hits, 0);
    assert_eq!(metrics.evaluation.memoization_misses, 2);
}

#[test]
fn aborted_keyed_evaluation_does_not_leak_key_registry_growth() {
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
    let node = runtime.graph_mut().node().build();
    let family = ComputationFamily::from("fresh-family");
    let computation = KeyedComputation::new(family.clone(), "fresh-key").with_memo_key("fresh-v1");
    let before = runtime.config().test_registry_counts();
    let mut runtime_ctx = ();

    let err = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
        tx.evaluate_keyed(node, &computation, &|view| {
            Ok(view.finish(
                NodeEvaluationResult::from_version(version_ab(1, 0)).with_output_identity("cached"),
            ))
        })?;
        Err(SignalError::invalid_input("force rollback"))
    });
    assert!(err.is_err());

    assert_eq!(
        runtime.config().test_registry_counts(),
        before,
        "aborted keyed evaluation must not leak family/key/memo registry entries"
    );
}
