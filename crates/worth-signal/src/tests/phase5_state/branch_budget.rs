use crate::facade::{NodeEvaluationResult, SignalGraph, SignalRuntime, SignalRuntimePolicy};
use crate::tests::support::{version_ab, ASPECT_A};

#[test]
fn branch_churn_respects_history_and_replay_budgets_under_tight_policy() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let policy = SignalRuntimePolicy::operational()
        .with_history_limit(2)
        .with_detail_limit(1)
        .with_history_details(false);
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime.set_runtime_policy(policy);
    let source = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("bounded-main"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-budget").unwrap();
    let main_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("bounded-feature"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();

    for _ in 0..20 {
        runtime.switch_branch(main.clone()).unwrap();
        runtime
            .restore_branch_snapshot(main.clone(), &main_snapshot)
            .unwrap();
        runtime.switch_branch(feature.clone()).unwrap();
        runtime
            .restore_branch_snapshot(feature.clone(), &feature_snapshot)
            .unwrap();
    }

    assert!(
        runtime
            .observe()
            .recent_execution_history_diagnostics()
            .len()
            <= policy.retention_budget.history_limit,
        "execution history should stay within the configured history budget under branch churn"
    );
    assert!(
        runtime.graph().replay_events().len() <= policy.retention_budget.history_limit.max(1) * 32,
        "replay retention should stay within the policy-derived bound under branch churn"
    );
    assert!(
        runtime.graph().observe().lineage_records().len()
            <= policy.retention_budget.history_limit.max(1) * 32,
        "lineage retention should stay within the policy-derived bound under branch churn"
    );
    assert_eq!(
        runtime.observe().known_branches().len(),
        2,
        "branch churn should not fabricate extra branch catalog entries"
    );
    assert_eq!(
        runtime.observe().branch_head_snapshot_id(main.id),
        Some(main_snapshot.meta.snapshot_id),
        "main head should remain pinned to its snapshot under churn"
    );
    assert_eq!(
        runtime.observe().branch_head_snapshot_id(feature.id),
        Some(feature_snapshot.meta.snapshot_id),
        "feature head should remain pinned to its snapshot under churn"
    );
}
