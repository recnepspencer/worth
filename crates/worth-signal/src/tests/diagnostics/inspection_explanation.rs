use crate::facade::*;
use crate::tests::support::*;

#[test]
fn diagnostics_entrypoint_exposes_one_discoverable_surface() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(1, 0)));
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph.execute_prepared_plan(&plan, &(), &compute).unwrap();

    let diagnostics = graph.observe().diagnostics();
    let summary = diagnostics.summary(DiagnosticsTier::Operational);
    let history = diagnostics.history(DiagnosticsTier::Operational);
    let latest_flow = diagnostics.latest_flow();
    let compare = diagnostics.compare();
    let graph_inspector = diagnostics.inspect_graph();
    let execution_inspector = diagnostics.inspect_execution();

    assert_eq!(summary.active_node_count, 1);
    assert!(history.latest_execution_record_id.is_some());
    assert!(latest_flow.is_some());
    assert!(compare.graphs(&summary, &summary).is_empty());
    assert!(graph_inspector
        .nodes_with_execution_record()
        .contains(&node));
    assert_eq!(execution_inspector.nodes_with_trace_summaries(), vec![node]);
}

#[test]
fn diagnostics_grouped_job_views_are_discoverable() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(1, 0)));
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    let report = graph.execute_prepared_plan(&plan, &(), &compute).unwrap();

    let diagnostics = graph.observe().diagnostics();
    let health = diagnostics.health_view();
    let inspect = diagnostics.inspect();

    assert_eq!(health.current_now().active_node_count, 1);
    assert!(health.latest_flow().is_some());
    assert!(health.recent_history().last().is_some());
    assert!(inspect
        .graph()
        .nodes_with_execution_record()
        .contains(&node));
    assert_eq!(inspect.execution().nodes_with_trace_summaries(), vec![node]);
    assert_eq!(inspect.plan(&plan).stage_count(), 1);
    assert!(inspect.report(&report).task_record_for_node(node).is_some());
}

#[test]
fn diagnostics_plan_summary_reports_contract_pruning() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().reads_aspects(mask_a()).build();
    let mut source_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut graph, source, &mut source_compute).unwrap();
    graph
        .append_dependency(dependent, source, ASPECT_B)
        .unwrap();

    let mut dependent_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut graph, dependent, &mut dependent_compute).unwrap();
    mark_dirty(&mut graph, source, ASPECT_B).unwrap();

    let plan = graph
        .build_evaluation_plan(&[dependent], EvaluationRequestMode::Default)
        .unwrap();
    let summary = plan.diagnostics_summary(DiagnosticsTier::Development);

    assert_eq!(summary.task_count, 0);
    assert_eq!(summary.contract_pruned_count, 1);
}

#[test]
fn explanation_summary_includes_contract_metadata() {
    let mut graph = SignalGraph::new();
    let node = graph
        .node()
        .reads_aspects([ASPECT_A])
        .produces_aspects([ASPECT_B])
        .requires_context(ContextRequirement::DomainContext)
        .with_partition_scope(PartitionSubscription::whole_partition("wing"))
        .build();

    let explanation = graph.observe().explain(node).unwrap();
    let summary = explanation.diagnostics_summary(DiagnosticsTier::Development);

    assert_eq!(
        summary.contract_reads_mask,
        AspectMask::from([ASPECT_A]).bits() as u128
    );
    assert_eq!(
        summary.contract_produces_mask,
        AspectMask::from([ASPECT_B]).bits() as u128
    );
    assert_eq!(summary.contract_partition_scope_count, 1);
    assert_eq!(summary.required_context, ContextRequirement::DomainContext);
}

#[test]
fn inspectors_query_graph_plan_report_and_execution_history() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().on_demand().build();

    let source_compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(1, 0)));
    let dependent_compute = |ctx: &mut EvaluationContext<'_, ()>| {
        let version = ctx.read_aspect_version(source, ASPECT_A)?;
        Ok(ctx.finish(NodeEvaluationResult::from_version(version)))
    };

    let bootstrap = graph
        .build_evaluation_plan(&[source, dependent], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|ctx| {
            if ctx.node() == source {
                source_compute(ctx)
            } else {
                dependent_compute(ctx)
            }
        })
        .unwrap();

    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    let plan = graph
        .build_evaluation_plan(&[dependent], EvaluationRequestMode::Default)
        .unwrap();
    let report = graph
        .execute_prepared_plan(&plan, &(), &|ctx| {
            if ctx.node() == source {
                source_compute(ctx)
            } else {
                dependent_compute(ctx)
            }
        })
        .unwrap();

    let graph_inspector = inspect_graph(&graph);
    assert_eq!(
        graph_inspector.nodes_with_condition(&EvaluationCondition::OnDemand),
        vec![dependent]
    );
    assert_eq!(graph_inspector.nodes_with_execution_record().len(), 2);

    let plan_inspector = inspect_plan(&plan);
    assert_eq!(plan_inspector.stage_count(), 2);
    assert_eq!(plan_inspector.tasks_for_node(dependent).len(), 1);

    let report_inspector = inspect_report(&report);
    assert_eq!(
        report_inspector
            .tasks_with_outcome(TaskExecutionOutcome::ValidatedClean)
            .len(),
        1
    );

    let execution_inspector = inspect_execution(&graph);
    assert_eq!(execution_inspector.nodes_with_trace_summaries().len(), 2);
    assert!(execution_inspector.latest_execution_record_id().is_some());
}

#[test]
fn rendered_execution_report_summary_names_advanced_reuse_families() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().build();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    let mut source_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    let mut dependent_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(10, 0));
    evaluate(&mut graph, source, &mut source_compute).unwrap();
    evaluate(&mut graph, dependent, &mut dependent_compute).unwrap();

    let mut report = graph
        .build_evaluation_plan(&[dependent], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    let _ = &mut report;

    let summary = ExecutionReportSummary {
        profile: DiagnosticsTier::Development,
        stage_count: 1,
        task_count: 4,
        tasks_executed: 4,
        tasks_pruned: 0,
        tasks_validated_clean: 0,
        tasks_deferred_by_condition: 0,
        tasks_reverted_clean_by_condition: 0,
        tasks_satisfied_by_memoization: 1,
        tasks_with_suppressed_propagation: 0,
        prepared_evaluations_produced: 4,
        prepared_evaluations_applied: 4,
        dependency_capture_updates: 0,
        semantic_segment_count: 4,
        temporal_summary: TemporalExecutionSummary::default(),
        task_outcome_counts: [
            (
                crate::logic::planner::TaskExecutionOutcome::MemoizedReuse,
                1,
            ),
            (
                crate::logic::planner::TaskExecutionOutcome::SnapshotRestoreReuse,
                1,
            ),
            (
                crate::logic::planner::TaskExecutionOutcome::CrossIdentityPersistentReuse,
                1,
            ),
            (
                crate::logic::planner::TaskExecutionOutcome::PartialArtifactSplice,
                1,
            ),
        ]
        .into_iter()
        .collect(),
        stage_outcome_counts: [(
            crate::logic::planner::StageExecutionOutcome::CompletedSerial,
            1,
        )]
        .into_iter()
        .collect(),
    };

    let rendered = render_execution_report_summary(&summary);
    assert!(rendered.contains("memoized=1"));
    assert!(rendered.contains("advanced_reuse=["));
    assert!(rendered.contains("snapshot_restore=1"));
    assert!(rendered.contains("cross_identity=1"));
    assert!(rendered.contains("partial_splice=1"));
}

#[test]
fn rendered_execution_history_summary_surfaces_correspondence_and_splice_detail() {
    let summary = ExecutionHistorySummary {
        profile: DiagnosticsTier::Development,
        traced_node_count: 2,
        execution_record_count: 2,
        latest_execution_record_id: Some(9),
        reuse_origin_counts: [
            (ReuseOrigin::CrossIdentityPersistentReuse, 1),
            (ReuseOrigin::PartialArtifactSplice, 1),
        ]
        .into_iter()
        .collect(),
        nodes: vec![
            ExecutionHistoryNodeSummary {
                node: NodeId::new(1, 0),
                execution_record_id: Some(8),
                semantic_segment_id: Some(1),
                output_change: Some(OutputChange::Refreshed),
                memoized_origin: None,
                reuse_basis: None,
                reuse_origin: Some(ReuseOrigin::CrossIdentityPersistentReuse),
                persistent_correspondence_kind: Some(
                    crate::data::reuse::PersistentCorrespondenceKind::LineageBackedMapping,
                ),
                composition_region_count: 0,
                reuse_certification_proof_count: 1,
                changed_partition_count: 0,
                causality_kind: None,
            },
            ExecutionHistoryNodeSummary {
                node: NodeId::new(2, 0),
                execution_record_id: Some(9),
                semantic_segment_id: Some(2),
                output_change: Some(OutputChange::Refreshed),
                memoized_origin: None,
                reuse_basis: None,
                reuse_origin: Some(ReuseOrigin::PartialArtifactSplice),
                persistent_correspondence_kind: None,
                composition_region_count: 3,
                reuse_certification_proof_count: 1,
                changed_partition_count: 2,
                causality_kind: None,
            },
        ],
    };

    let rendered = render_execution_history_summary(&summary);
    assert!(rendered.contains("LineageBackedMapping"));
    assert!(rendered.contains("partial_splice_nodes=1"));
    assert!(rendered.contains("partial_splice_regions=3"));
}
