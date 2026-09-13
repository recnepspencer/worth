use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::diagnostics::flow::{
    ApplySummary, ChangeInputSummary, FlowCauseSample, FlowSummary, InvalidationSummary,
    PlanningSummary, PrecomputeSummary,
};
use crate::diagnostics::policy::OrdinaryAccessLane;
use crate::diagnostics::replay::{ReplayEvent, ReplayEventDetail, ReplayEventKind};
use crate::diagnostics::summary::{
    EvaluationPlanSummary, ExecutionHistorySummary, ExplanationSummary,
};
use crate::logic::explain::CausalLinkKind;
use crate::logic::planner::{ExecutionReport, TaskExecutionOutcome};

pub(crate) fn record_semantic_execution(
    graph: &mut SignalGraph,
    plan_summary: EvaluationPlanSummary,
    first_target: Option<NodeId>,
    report: &ExecutionReport,
) {
    if !graph.captures_observation_surface(
        crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
    ) {
        return;
    }
    let installed_policy = graph.installed_runtime_policy();
    let retention_budget = installed_policy.retention_budget();
    let profile = installed_policy.tier();
    let (change, invalidation) = graph
        .diagnostics_state()
        .pending_change_summary()
        .unwrap_or_else(|| {
            (
                ChangeInputSummary::new(Vec::new(), Vec::new(), 0, None),
                InvalidationSummary::empty_frontier(),
            )
        });
    let explanation = if retention_budget.retain_flow_explanation {
        first_target
            .as_ref()
            .and_then(|target| graph.materialize_explanation_artifact(*target).ok())
            .and_then(|(explanation, _availability)| explanation)
            .map(|explanation| ExplanationSummary::from_explanation(&explanation, profile))
    } else {
        None
    };
    let flow = FlowSummary::new(
        profile,
        change,
        invalidation,
        PlanningSummary::from_summary(plan_summary),
        PrecomputeSummary::from_report(report, profile),
        ApplySummary::from_report(report, profile),
        if retention_budget.retain_stage_details {
            sample_flow_causes(graph, report, retention_budget.detail_limit.get())
        } else {
            Vec::new()
        },
        Vec::new(),
        None,
        None,
        explanation,
    );
    let history = if !retention_budget.retain_history_details {
        ExecutionHistorySummary::from_report(report, profile, retention_budget.detail_limit, false)
    } else if execution_history_unchanged(report) {
        graph
            .diagnostics_state()
            .recent_history()
            .back()
            .cloned()
            .or_else(|| {
                ExecutionHistorySummary::from_complete_execution_facts(
                    graph,
                    report,
                    profile,
                    retention_budget.detail_limit,
                )
            })
            .unwrap_or_else(|| {
                ExecutionHistorySummary::from_graph(
                    graph,
                    profile,
                    retention_budget.detail_limit,
                    retention_budget.retain_history_details,
                    OrdinaryAccessLane,
                )
            })
    } else {
        ExecutionHistorySummary::from_complete_execution_facts(
            graph,
            report,
            profile,
            retention_budget.detail_limit,
        )
        .unwrap_or_else(|| {
            ExecutionHistorySummary::from_graph(
                graph,
                profile,
                retention_budget.detail_limit,
                retention_budget.retain_history_details,
                OrdinaryAccessLane,
            )
        })
    };
    let _ = retention_budget;
    let _ = profile;
    let _ = OrdinaryAccessLane;
    graph
        .diagnostics_state_mut()
        .complete_flow_without_graph_summary(flow, history);
    if graph.captures_observation_surface(
        crate::logic::transaction::SignalObservationSurface::ReplayDetail,
    ) {
        let branch_id = graph.current_branch().id;
        for task in report
            .stages
            .iter()
            .flat_map(|stage| stage.task_records.iter())
        {
            let cursor = graph.diagnostics_state_mut().allocate_replay_cursor();
            let replay_projection = graph
                .node_replay_projection(task.node)
                .ok()
                .unwrap_or_default();
            let lineage_artifact_id = graph.node_lineage_artifact_id(task.node).ok().flatten();
            graph
                .diagnostics_state_mut()
                .record_replay_event(ReplayEvent::new(
                    cursor,
                    ReplayEventKind::TaskApplied,
                    branch_id,
                    None,
                    Some(task.node),
                    Some(task.id.0),
                    Some(task.semantic_segment_id.0),
                    lineage_artifact_id.or(replay_projection.lineage_artifact_id),
                    Some(task.reuse_origin),
                    replay_projection.persistent_correspondence_kind,
                    replay_projection.composition_region_count,
                    Some(ReplayEventDetail::TaskOutcome(task.outcome)),
                ));
        }
    }
}

fn sample_flow_causes(
    graph: &SignalGraph,
    report: &ExecutionReport,
    limit: usize,
) -> Vec<FlowCauseSample> {
    let mut nodes = Vec::new();
    for task in report
        .stages
        .iter()
        .flat_map(|stage| stage.task_records.iter())
        .map(|record| record.node)
    {
        if !nodes.contains(&task) {
            nodes.push(task);
        }
        if nodes.len() >= limit {
            break;
        }
    }

    let mut samples = Vec::new();
    for node in nodes {
        let explanation = match graph.observe().explain(node) {
            Ok(explanation) => explanation,
            Err(_) => {
                let Some(fact) = graph.explanation_fact(node) else {
                    continue;
                };
                fact.explanation.clone()
            }
        };
        let rewired = explanation
            .rewiring
            .as_ref()
            .is_some_and(|rewiring| !rewiring.added.is_empty() || !rewiring.removed.is_empty());
        let mut suspect_classes = Vec::new();
        if rewired {
            suspect_classes.push("rewiring".to_string());
        }
        if explanation.causal_links.iter().any(|link| {
            matches!(
                link.scope.kind,
                crate::logic::explain::ScopeProvenanceKind::Direct
                    | crate::logic::explain::ScopeProvenanceKind::Translated
                    | crate::logic::explain::ScopeProvenanceKind::Discarded
                    | crate::logic::explain::ScopeProvenanceKind::InsufficientEvidence
            )
        }) {
            suspect_classes.push("locality".to_string());
        }
        if explanation.causal_links.iter().any(|link| {
            matches!(
                link.disposition,
                crate::logic::explain::CausalDisposition::Conservative
            ) || matches!(
                link.kind,
                CausalLinkKind::SkippedByComparator | CausalLinkKind::ConditionDeferred { .. }
            )
        }) && !suspect_classes.contains(&"validation".to_string())
        {
            suspect_classes.push("validation".to_string());
        }
        samples.push(FlowCauseSample {
            node,
            cause_kinds: explanation
                .causal_links
                .iter()
                .take(limit)
                .map(|link| link.kind.to_string())
                .collect(),
            scope_kinds: explanation
                .causal_links
                .iter()
                .filter(|link| {
                    !matches!(
                        link.scope.kind,
                        crate::logic::explain::ScopeProvenanceKind::None
                    )
                })
                .take(limit)
                .map(|link| format!("{:?}", link.scope.kind))
                .collect(),
            scope_notes: explanation
                .causal_links
                .iter()
                .filter_map(|link| link.note.clone())
                .take(limit)
                .collect(),
            suspect_classes,
            rewired,
            conservative_recompute: explanation.causal_links.iter().any(|link| {
                matches!(
                    link.disposition,
                    crate::logic::explain::CausalDisposition::Conservative
                )
            }),
        });
    }
    samples
}

fn execution_history_unchanged(report: &ExecutionReport) -> bool {
    report
        .stages
        .iter()
        .flat_map(|stage| &stage.task_records)
        .all(|task| {
            matches!(
                task.outcome,
                TaskExecutionOutcome::ValidatedClean
                    | TaskExecutionOutcome::ConditionDeferred
                    | TaskExecutionOutcome::ConditionRevertedClean
                    | TaskExecutionOutcome::Pruned
            )
        })
}
