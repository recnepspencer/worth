use super::runtime_world::build_runtime;
use crate::facade::{
    lineage_records_equivalent, mark_dirty, replay_slices_equivalent, DiagnosticsAvailability,
    DiagnosticsTier, ExecutionHistorySummary, ExplanationSummary, FlowSummary, GraphSummary,
    LineageRecord, NodeEvaluationResult, NodeId, ReplaySlice, SignalGraph,
    SignalObservationRequest, SignalRuntimePolicy, SnapshotRestoreIntent,
    SnapshotRestoreLineageMode,
};
use crate::tests::support::{evaluate, version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn tier_matrix_public_observer_surfaces_preserve_truth_while_availability_changes() {
    #[derive(Clone)]
    struct TierRun {
        summary: GraphSummary,
        history: ExecutionHistorySummary,
        flow: Option<FlowSummary>,
        replay: ReplaySlice,
        lineage: Vec<LineageRecord>,
        explanation: ExplanationSummary,
        explanation_availability: DiagnosticsAvailability,
        provenance_availability: DiagnosticsAvailability,
        ordinary_cold_requests: u64,
    }

    fn run(policy: SignalRuntimePolicy) -> TierRun {
        let mut graph = SignalGraph::new();
        graph.set_runtime_policy(policy);
        let session = graph
            .begin_observation_session(SignalObservationRequest::operation())
            .unwrap();
        graph.cancel_observation_session(&session).unwrap();
        let source = graph.node().output_identity().build();
        let dependent = graph.node().build();
        graph
            .append_dependency(dependent, source, ASPECT_A)
            .unwrap();

        let mut source_v1 = |_id: NodeId, _graph: &SignalGraph| {
            Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
                .with_output_identity("artifact-v1"))
        };
        let mut source_v2 = |_id: NodeId, _graph: &SignalGraph| {
            Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
                .with_output_identity("artifact-v2"))
        };
        let mut dependent_compute = |_id: NodeId, graph: &SignalGraph| {
            Ok(NodeEvaluationResult::from_version(
                graph.get_entry(source).unwrap().get_aspect_version(),
            ))
        };

        evaluate(&mut graph, source, &mut source_v1).unwrap();
        evaluate(&mut graph, dependent, &mut dependent_compute).unwrap();
        let snapshot = graph.capture_snapshot();

        mark_dirty(&mut graph, source, ASPECT_A).unwrap();
        evaluate(&mut graph, source, &mut source_v2).unwrap();
        evaluate(&mut graph, dependent, &mut dependent_compute).unwrap();
        graph
            .restore_snapshot_with_intent(
                &snapshot,
                SnapshotRestoreIntent::restore_runtime_truth_with_active_policy(),
            )
            .unwrap();

        let before_ordinary = graph
            .observe()
            .metrics()
            .storage
            .explicit_cold_materialization_request_count;
        let summary = graph.observe().diagnostics_summary(policy.tier);
        let history = graph.observe().execution_history_summary(policy.tier);
        let flow = graph
            .observe()
            .latest_flow_diagnostics()
            .map(|flow| flow.to_owned_summary());
        let replay = graph
            .observe()
            .replay_around_snapshot(snapshot.snapshot_id())
            .to_owned_slice();
        let lineage = graph.observe().lineage_for_node(source).to_owned_records();
        let explanation = graph
            .observe()
            .explain(dependent)
            .unwrap()
            .diagnostics_summary(policy.tier);
        let after_ordinary = graph
            .observe()
            .metrics()
            .storage
            .explicit_cold_materialization_request_count;
        let ordinary_cold_requests = after_ordinary.saturating_sub(before_ordinary);

        let explanation_availability = graph.materialize_explanation_artifact(dependent).unwrap().1;
        let provenance_availability = graph.materialize_provenance_artifact(dependent).unwrap().1;

        TierRun {
            summary,
            history,
            flow,
            replay,
            lineage,
            explanation,
            explanation_availability,
            provenance_availability,
            ordinary_cold_requests,
        }
    }

    let operational = run(SignalRuntimePolicy::operational());
    let development = run(SignalRuntimePolicy::development());
    let forensic = run(SignalRuntimePolicy::forensic()
        .with_snapshot_restore_lineage_mode(SnapshotRestoreLineageMode::CompactGlobal));

    for (left, right) in [
        (&operational, &development),
        (&development, &forensic),
        (&operational, &forensic),
    ] {
        assert!(
            left.summary.active_node_count == right.summary.active_node_count
                && left.summary.clean_node_count == right.summary.clean_node_count
                && left.summary.maybe_stale_node_count == right.summary.maybe_stale_node_count
                && left.summary.dirty_node_count == right.summary.dirty_node_count
                && left.summary.dependency_edge_count == right.summary.dependency_edge_count
                && left.summary.subscriber_edge_count == right.summary.subscriber_edge_count
                && left.summary.nodes_with_causality == right.summary.nodes_with_causality,
            "graph summaries should preserve the same canonical graph truth across tier changes"
        );
        assert!(
            left.history.traced_node_count == right.history.traced_node_count
                && left.history.execution_record_count == right.history.execution_record_count
                && left.history.latest_execution_record_id
                    == right.history.latest_execution_record_id
                && left.history.reuse_origin_counts == right.history.reuse_origin_counts,
            "execution history should preserve the same conclusion set across tier changes"
        );
        if let (Some(left_flow), Some(right_flow)) = (&left.flow, &right.flow) {
            assert!(
                left_flow.change == right_flow.change
                    && left_flow.invalidation == right_flow.invalidation
                    && left_flow.planning.plan.task_count == right_flow.planning.plan.task_count
                    && left_flow.planning.plan.stage_count == right_flow.planning.plan.stage_count
                    && left_flow.precompute.prepared_evaluations_produced
                        == right_flow.precompute.prepared_evaluations_produced
                    && left_flow.apply.prepared_evaluations_applied
                        == right_flow.apply.prepared_evaluations_applied
                    && left_flow.rollback == right_flow.rollback,
                "latest flow should preserve the same semantic truth across retained tiers"
            );
            assert!(
                replay_slices_equivalent(&left.replay, &right.replay),
                "replay should remain semantically equivalent across retained tiers"
            );
            assert!(
                lineage_records_equivalent(&left.lineage, &right.lineage),
                "lineage should remain semantically equivalent across retained tiers"
            );
        }
        assert!(
            left.explanation.node == right.explanation.node
                && left.explanation.state == right.explanation.state
                && left.explanation.upstream_count == right.explanation.upstream_count
                && left.explanation.changed_upstream_count
                    == right.explanation.changed_upstream_count
                && left.explanation.skipped_upstream_count
                    == right.explanation.skipped_upstream_count
                && left.explanation.condition_deferred_count
                    == right.explanation.condition_deferred_count
                && left.explanation.clean_upstream_count == right.explanation.clean_upstream_count
                && left.explanation.missing_snapshot_count
                    == right.explanation.missing_snapshot_count
                && left.explanation.dependency_removed_count
                    == right.explanation.dependency_removed_count
                && left.explanation.propagation_suppressed
                    == right.explanation.propagation_suppressed
                && left.explanation.output_change == right.explanation.output_change
                && left.explanation.memoized_origin == right.explanation.memoized_origin
                && left.explanation.reuse_basis == right.explanation.reuse_basis
                && left.explanation.reuse_origin == right.explanation.reuse_origin
                && left.explanation.contract_reads_mask == right.explanation.contract_reads_mask
                && left.explanation.contract_produces_mask
                    == right.explanation.contract_produces_mask
                && left.explanation.required_context == right.explanation.required_context,
            "explanations should preserve the same semantic truth across tier changes"
        );
        assert_eq!(
            left.ordinary_cold_requests, 0,
            "ordinary observer access must not trigger cold materialization"
        );
        assert_eq!(
            right.ordinary_cold_requests, 0,
            "ordinary observer access must not trigger cold materialization"
        );
    }

    assert_eq!(
        operational.explanation_availability,
        DiagnosticsAvailability::ReconstructedAvailable
    );
    assert_eq!(
        operational.provenance_availability,
        DiagnosticsAvailability::ReconstructedAvailable
    );
    assert!(development.explanation_availability.is_available());
    assert!(development.provenance_availability.is_available());
    assert!(forensic.explanation_availability.is_available());
    assert!(forensic.provenance_availability.is_available());
}

#[test]
fn ordinary_observer_access_never_increments_cold_or_denial_counters_across_tiers() {
    for tier in [
        DiagnosticsTier::Operational,
        DiagnosticsTier::Development,
        DiagnosticsTier::Forensic,
    ] {
        let mut runtime = build_runtime(SignalGraph::new());
        runtime.set_runtime_policy(SignalRuntimePolicy::for_tier(tier));
        let source = runtime.graph_mut().node().output_identity().build();
        let mut runtime_ctx = ();

        runtime
            .transaction(&mut runtime_ctx, |tx| {
                tx.read(source, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(1, 0))
                            .with_output_identity(format!("ordinary-tier-{}", tier.label())),
                    ))
                })?;
                Ok(())
            })
            .unwrap();

        let branch = runtime.observe().current_branch();
        let metrics_before = runtime.observe().metrics().storage;

        let diagnostics = runtime.observe().diagnostics();
        let _summary = diagnostics.summary(tier);
        let _history = diagnostics.history(tier);
        let _recent = diagnostics.recent_history();
        let _latest_flow = runtime.observe().latest_flow_diagnostics();
        let _replay = runtime.observe().replay_for_branch(branch.id);
        let _lineage = runtime.observe().lineage_chain_for_node(source);
        let _explanation = runtime.observe().explain(source).unwrap();

        let metrics_after = runtime.observe().metrics().storage;
        assert_eq!(
            metrics_before.explicit_cold_materialization_request_count,
            metrics_after.explicit_cold_materialization_request_count,
            "ordinary observer access must not request cold materialization for tier {}",
            tier.label()
        );
        assert_eq!(
            metrics_before.cold_explanation_reconstruction_count,
            metrics_after.cold_explanation_reconstruction_count,
            "ordinary observer access must not reconstruct explanation artifacts for tier {}",
            tier.label()
        );
        assert_eq!(
            metrics_before.cold_provenance_reconstruction_count,
            metrics_after.cold_provenance_reconstruction_count,
            "ordinary observer access must not reconstruct provenance artifacts for tier {}",
            tier.label()
        );
        assert_eq!(
            metrics_before.reconstructed_artifact_read_count,
            metrics_after.reconstructed_artifact_read_count,
            "ordinary observer access must not record reconstructed artifact reads for tier {}",
            tier.label()
        );
        assert_eq!(
            metrics_before.denied_reconstruction_by_budget_count,
            metrics_after.denied_reconstruction_by_budget_count,
            "ordinary observer access must not produce budget denial counts for tier {}",
            tier.label()
        );
        assert_eq!(
            metrics_before.denied_reconstruction_by_tier_count,
            metrics_after.denied_reconstruction_by_tier_count,
            "ordinary observer access must not produce tier denial counts for tier {}",
            tier.label()
        );
        assert_eq!(
            metrics_before.denied_reconstruction_explanation_api_count,
            metrics_after.denied_reconstruction_explanation_api_count,
            "ordinary observer access must not increment explanation denial attribution for tier {}",
            tier.label()
        );
        assert_eq!(
            metrics_before.denied_reconstruction_provenance_api_count,
            metrics_after.denied_reconstruction_provenance_api_count,
            "ordinary observer access must not increment provenance denial attribution for tier {}",
            tier.label()
        );
    }
}
