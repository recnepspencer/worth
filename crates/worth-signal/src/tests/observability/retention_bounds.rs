use super::runtime_world::build_runtime;
use crate::facade::{
    render_execution_history_summary, render_graph_summary, DiagnosticsAvailability,
    DiagnosticsTier, NodeEvaluationResult, SignalGraph, SignalObservationRequest,
    SignalRuntimePolicy,
};
use crate::tests::support::{version_ab, ASPECT_A};

#[test]
fn branch_and_snapshot_churn_respect_retention_budget_under_all_tiers() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    for policy in [
        SignalRuntimePolicy::operational()
            .with_history_limit(2)
            .with_detail_limit(1)
            .with_history_details(false),
        SignalRuntimePolicy::development()
            .with_history_limit(3)
            .with_detail_limit(2)
            .with_history_details(true),
        SignalRuntimePolicy::forensic()
            .with_history_limit(4)
            .with_detail_limit(3)
            .with_history_details(true),
    ] {
        let mut runtime = build_runtime(SignalGraph::new());
        runtime.set_runtime_policy(policy);
        let session = runtime
            .begin_observation_session(SignalObservationRequest::operation())
            .unwrap();
        runtime.cancel_observation_session(&session).unwrap();
        let source = runtime.graph_mut().node().output_identity().build();
        let mut runtime_ctx = ();

        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.read(source, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(1, 0))
                            .with_output_identity(format!("main-seed-{}", policy.tier.label())),
                    ))
                })?;
                Ok(())
            })
            .unwrap();

        let main = runtime.observe().current_branch();
        let feature = runtime
            .create_branch(format!("feature-retention-{}", policy.tier.label()))
            .unwrap();
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
                            .with_output_identity(format!("feature-seed-{}", policy.tier.label())),
                    ))
                })?;
                Ok(())
            })
            .unwrap();
        let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();

        for cycle in 0..18 {
            let (branch, snapshot, output_identity) = if cycle % 2 == 0 {
                (
                    main.clone(),
                    &main_snapshot,
                    format!("main-cycle-{}-{cycle}", policy.tier.label()),
                )
            } else {
                (
                    feature.clone(),
                    &feature_snapshot,
                    format!("feature-cycle-{}-{cycle}", policy.tier.label()),
                )
            };
            runtime.switch_branch(branch.clone()).unwrap();
            runtime
                .restore_branch_snapshot(branch.clone(), snapshot)
                .unwrap();
            runtime
                .transaction(request_execution, &mut runtime_ctx, |tx| {
                    tx.mark_dirty(source, ASPECT_A)?;
                    tx.read(source, &|view| {
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version_ab(cycle + 3, 0))
                                .with_output_identity(output_identity.clone()),
                        ))
                    })?;
                    Ok(())
                })
                .unwrap();

            let diagnostics = runtime.observe().diagnostics();
            assert!(
                diagnostics.recent_history().len() <= policy.retention_budget.history_limit,
                "recent history must stay within retention budget for tier {}",
                policy.tier.label()
            );
            assert!(
                runtime.graph().replay_events().len()
                    <= policy.retention_budget.history_limit.max(1) * 32,
                "replay retention must stay bounded for tier {}",
                policy.tier.label()
            );
            assert!(
                runtime.graph().observe().lineage_records().len()
                    <= policy.retention_budget.history_limit.max(1) * 32,
                "lineage retention must stay bounded for tier {}",
                policy.tier.label()
            );
        }
    }
}

#[test]
fn ordinary_summary_and_history_rendering_respect_retained_detail_limits() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let policy = SignalRuntimePolicy::development()
        .with_history_limit(3)
        .with_detail_limit(1)
        .with_history_details(true);
    let mut runtime = build_runtime(SignalGraph::new());
    runtime.set_runtime_policy(policy);
    let source_a = runtime.graph_mut().node().output_identity().build();
    let source_b = runtime.graph_mut().node().output_identity().build();
    let source_c = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    for (index, node) in [source_a, source_b, source_c].into_iter().enumerate() {
        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.read(node, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(index as u64 + 1, 0))
                            .with_output_identity(format!("render-bounded-{index}")),
                    ))
                })?;
                Ok(())
            })
            .unwrap();
    }

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source_a, ASPECT_A)?;
            tx.mark_dirty(source_b, ASPECT_A)?;
            Ok(())
        })
        .unwrap();

    let metrics_before = runtime.observe().metrics().storage;
    let diagnostics = runtime.observe().diagnostics();
    let summary = diagnostics.summary(DiagnosticsTier::Development);
    let history = diagnostics.history(DiagnosticsTier::Development);
    let rendered_summary = render_graph_summary(&summary);
    let rendered_history = render_execution_history_summary(&history);
    let metrics_after = runtime.observe().metrics().storage;

    assert!(summary.sample_dirty_nodes.len() <= policy.retention_budget.detail_limit);
    assert!(
        summary.sample_nodes_with_execution_record.len() <= policy.retention_budget.detail_limit
    );
    assert!(history.nodes.len() <= policy.retention_budget.detail_limit);
    assert!(rendered_summary.contains("GraphSummary"));
    assert!(rendered_history.contains("ExecutionHistorySummary"));
    assert_eq!(
        metrics_before.explicit_cold_materialization_request_count,
        metrics_after.explicit_cold_materialization_request_count,
        "ordinary rendering must not request cold materialization"
    );
    assert_eq!(
        metrics_before.cold_explanation_reconstruction_count,
        metrics_after.cold_explanation_reconstruction_count,
        "ordinary rendering must not reconstruct explanation artifacts"
    );
    assert_eq!(
        metrics_before.cold_provenance_reconstruction_count,
        metrics_after.cold_provenance_reconstruction_count,
        "ordinary rendering must not reconstruct provenance artifacts"
    );
}

#[test]
fn long_session_branch_churn_with_mixed_reads_keeps_bounds_and_cold_work_honest() {
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
    let mut runtime = build_runtime(SignalGraph::new());
    runtime.set_runtime_policy(policy);
    let session = runtime
        .begin_observation_session(SignalObservationRequest::operation())
        .unwrap();
    runtime.cancel_observation_session(&session).unwrap();
    let source = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("long-main"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-long-session").unwrap();
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
                        .with_output_identity("long-feature"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();

    for cycle in 0..24 {
        let (branch, snapshot) = if cycle % 2 == 0 {
            (main.clone(), &main_snapshot)
        } else {
            (feature.clone(), &feature_snapshot)
        };
        runtime.switch_branch(branch.clone()).unwrap();
        runtime
            .restore_branch_snapshot(branch.clone(), snapshot)
            .unwrap();

        let before_ordinary = runtime
            .observe()
            .metrics()
            .storage
            .explicit_cold_materialization_request_count;
        let diagnostics = runtime.observe().diagnostics();
        let summary = diagnostics.summary(DiagnosticsTier::Operational);
        let history = diagnostics.history(DiagnosticsTier::Operational);
        let _recent = diagnostics.recent_history();
        let _replay = runtime.observe().replay_for_branch(branch.id);
        let _lineage = runtime.observe().lineage_chain_for_node(source);
        let rendered_history = render_execution_history_summary(&history);
        let rendered_summary = render_graph_summary(&summary);
        let after_ordinary = runtime
            .observe()
            .metrics()
            .storage
            .explicit_cold_materialization_request_count;
        assert_eq!(
            before_ordinary, after_ordinary,
            "ordinary diagnostics reads must stay zero-cold under long-session churn"
        );
        assert!(rendered_history.contains("ExecutionHistorySummary"));
        assert!(rendered_summary.contains("GraphSummary"));

        if cycle % 4 == 0 {
            let before_burst = runtime.observe().metrics().storage;
            let (explanation, explanation_mode) = runtime
                .observe()
                .materialize()
                .materialize_explanation_artifact(source)
                .unwrap();
            let (provenance, provenance_mode) = runtime
                .observe()
                .materialize()
                .materialize_provenance_artifact(source)
                .unwrap();
            let after_burst = runtime.observe().metrics().storage;
            assert!(explanation.is_some());
            assert!(provenance.is_some());
            assert_eq!(
                explanation_mode,
                DiagnosticsAvailability::ReconstructedAvailable
            );
            assert_eq!(
                provenance_mode,
                DiagnosticsAvailability::ReconstructedAvailable
            );
            assert_eq!(
                after_burst.explicit_cold_materialization_request_count
                    - before_burst.explicit_cold_materialization_request_count,
                2,
                "each explicit cold burst should record exactly two requests"
            );
            assert_eq!(
                after_burst.reconstructed_artifact_read_count
                    - before_burst.reconstructed_artifact_read_count,
                2,
                "each explicit cold burst should record exactly two reconstructed reads"
            );
            assert_eq!(
                after_burst.cold_explanation_reconstruction_count
                    - before_burst.cold_explanation_reconstruction_count,
                1
            );
            assert_eq!(
                after_burst.cold_provenance_reconstruction_count
                    - before_burst.cold_provenance_reconstruction_count,
                1
            );
        }
    }

    let metrics = runtime.observe().metrics();
    assert_eq!(metrics.storage.denied_reconstruction_by_budget_count, 0);
    assert_eq!(metrics.storage.denied_reconstruction_by_tier_count, 0);
    assert!(
        runtime
            .observe()
            .recent_execution_history_diagnostics()
            .len()
            <= policy.retention_budget.history_limit
    );
    assert!(
        runtime.graph().replay_events().len() <= policy.retention_budget.history_limit.max(1) * 32
    );
    assert!(
        runtime.graph().observe().lineage_records().len()
            <= policy.retention_budget.history_limit.max(1) * 32
    );
    assert_eq!(runtime.observe().known_branches().len(), 2);
}
