use crate::facade::*;
use crate::tests::support::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Domain {
    Cache,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Impact {
    One,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ev {
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Tier {
    Feature,
}

fn build_runtime(graph: SignalGraph) -> SignalRuntime<Domain, Impact, Ev, (), Tier> {
    let _ = Domain::Cache;
    let _ = Impact::One;
    let _ = Tier::Feature;
    SignalRuntime::builder(graph)
        .with_kernel_defaults()
        .with_domains::<Domain>()
        .with_impacts::<Impact>()
        .with_events::<Ev>()
        .with_tiers::<Tier>()
        .checkpoint_barrier(CheckpointBarrier::PerOperation)
        .build()
}

#[test]
fn rollback_heavy_workload_leaves_runtime_consistent() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let root = graph.node().build();
    let mut runtime = build_runtime(graph);

    let mut ctx = ();
    for _ in 0..100 {
        let mut tx = runtime.begin(request_execution, &mut ctx);
        tx.mark_dirty(root, ASPECT_B).unwrap();
        tx.emit_event(Ev::Tick);
        tx.flush_events(CheckpointBarrier::PerOperation).unwrap();
        assert_eq!(
            tx.rollback().unwrap().outcome,
            TransactionOutcome::RolledBack
        );
    }

    assert_eq!(
        runtime.telemetry().transaction.transaction_rollback_count,
        100
    );
}

#[test]
#[ignore = "long-running stress test for CI/nightly profiles"]
fn stress_100k_nodes_transaction_commit() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let mut nodes = Vec::with_capacity(100_000);
    for _ in 0..100_000 {
        nodes.push(graph.node().build());
    }

    let mut runtime = build_runtime(graph);
    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);

    for node in nodes.iter().step_by(97) {
        tx.mark_dirty(*node, ASPECT_B).unwrap();
    }
    assert_eq!(tx.commit().unwrap().outcome, TransactionOutcome::Committed);
    assert!(runtime.telemetry().transaction.staged_node_patch_count > 0);
}
