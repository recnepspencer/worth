mod retained_charge;

use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::{MemoizedResultOrigin, OutputChange};
use crate::data::reuse::{PersistentCorrespondenceKind, ReuseBasis, ReuseOrigin};
use crate::diagnostics::facts::ExplanationFact;
use crate::diagnostics::policy::{DetailLimit, OrdinaryAccessLane};
use crate::diagnostics::profile::DiagnosticsTier;
use crate::logic::planner::ExecutionReport;

pub type ReuseOriginCounts = BTreeMap<ReuseOrigin, u32>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionHistoryNodeSummary {
    pub node: NodeId,
    pub execution_record_id: Option<u64>,
    pub semantic_segment_id: Option<u64>,
    pub output_change: Option<OutputChange>,
    pub memoized_origin: Option<MemoizedResultOrigin>,
    pub reuse_basis: Option<ReuseBasis>,
    pub reuse_origin: Option<ReuseOrigin>,
    pub persistent_correspondence_kind: Option<PersistentCorrespondenceKind>,
    pub composition_region_count: u32,
    pub reuse_certification_proof_count: u32,
    pub changed_partition_count: u32,
    pub causality_kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionHistorySummary {
    pub profile: DiagnosticsTier,
    pub traced_node_count: u32,
    pub execution_record_count: u32,
    pub latest_execution_record_id: Option<u64>,
    pub reuse_origin_counts: ReuseOriginCounts,
    pub nodes: Vec<ExecutionHistoryNodeSummary>,
}

impl ExecutionHistorySummary {
    pub fn with_profile(&self, profile: DiagnosticsTier) -> Self {
        let mut cloned = self.clone();
        cloned.profile = profile;
        cloned
    }

    pub fn from_graph(
        graph: &SignalGraph,
        profile: DiagnosticsTier,
        detail_limit: DetailLimit,
        retain_history_details: bool,
        _lane: OrdinaryAccessLane,
    ) -> Self {
        let mut traced_node_count = 0_u32;
        let mut execution_record_count = 0_u32;
        let mut latest_execution_record_id = None;
        let mut reuse_origin_counts = ReuseOriginCounts::new();
        let mut nodes = Vec::new();

        for index in 0..graph.arena_capacity() {
            let Some(node) = graph.live_node_id_at(index) else {
                continue;
            };
            let Ok(Some(trace)) = graph.observe().runtime_artifact_state(node) else {
                continue;
            };
            let execution_trace = graph.node_execution_trace_stamp(node).ok().flatten();
            traced_node_count += 1;
            *reuse_origin_counts.entry(trace.reuse_origin()).or_insert(0) += 1;
            if let Some(id) = execution_trace.and_then(|stamp| stamp.execution_record_id) {
                execution_record_count += 1;
                latest_execution_record_id =
                    Some(latest_execution_record_id.map_or(id, |current: u64| current.max(id)));
            }
            if retain_history_details {
                nodes.push(ExecutionHistoryNodeSummary {
                    node,
                    execution_record_id: execution_trace
                        .and_then(|stamp| stamp.execution_record_id),
                    semantic_segment_id: execution_trace
                        .and_then(|stamp| stamp.semantic_segment_id),
                    output_change: Some(trace.output_change()),
                    memoized_origin: Some(trace.memoized_origin()),
                    reuse_basis: Some(trace.reuse_basis().clone_inner()),
                    reuse_origin: Some(trace.reuse_origin()),
                    persistent_correspondence_kind: trace
                        .reuse_boundary_authority()
                        .and_then(|authority| authority.persistent_correspondence_kind()),
                    composition_region_count: trace
                        .reuse_boundary_authority()
                        .map(|authority| authority.composition_region_count())
                        .unwrap_or(0),
                    reuse_certification_proof_count: graph
                        .node_cold_artifact_record(node)
                        .unwrap_or(None)
                        .and_then(|retained| retained.reuse_certification.as_ref())
                        .map(|record| record.proofs.len() as u32)
                        .unwrap_or(0),
                    changed_partition_count: trace.changed_partition_count(),
                    causality_kind: graph
                        .causality_of(node)
                        .unwrap_or(None)
                        .map(|c| c.kind.clone()),
                });
            }
        }

        if retain_history_details {
            nodes.sort_by(|left, right| {
                right
                    .execution_record_id
                    .cmp(&left.execution_record_id)
                    .then_with(|| right.semantic_segment_id.cmp(&left.semantic_segment_id))
                    .then_with(|| left.node.index().cmp(&right.node.index()))
                    .then_with(|| left.node.generation().cmp(&right.node.generation()))
            });
            if nodes.len() > detail_limit.get() {
                nodes.truncate(detail_limit.get());
            }
        }

        Self {
            profile,
            traced_node_count,
            execution_record_count,
            latest_execution_record_id,
            reuse_origin_counts,
            nodes,
        }
    }

    pub(crate) fn from_complete_execution_facts(
        graph: &SignalGraph,
        report: &ExecutionReport,
        profile: DiagnosticsTier,
        detail_limit: DetailLimit,
    ) -> Option<Self> {
        let active_node_count = graph.active_node_count();
        let retained_facts = graph.diagnostics_state().explanation_facts();
        let task_record_count = report
            .stages
            .iter()
            .map(|stage| stage.task_records.len())
            .sum::<usize>();
        if report.task_count as usize != active_node_count
            || task_record_count != active_node_count
            || retained_facts.len() != active_node_count
        {
            return None;
        }

        let mut latest_execution_record_id = None;
        let mut reuse_origin_counts = ReuseOriginCounts::new();
        let mut newest_facts = Vec::with_capacity(detail_limit.get().min(active_node_count));
        for task in report
            .stages
            .iter()
            .flat_map(|stage| stage.task_records.iter())
        {
            let fact = retained_facts.get(&task.node)?;
            let explanation = &fact.explanation;
            if explanation.execution_record_id != Some(task.id.0)
                || explanation.semantic_segment_id != Some(task.semantic_segment_id.0)
                || explanation.historical_artifact_record.is_none()
            {
                return None;
            }
            latest_execution_record_id = Some(
                latest_execution_record_id.map_or(task.id.0, |current: u64| current.max(task.id.0)),
            );
            *reuse_origin_counts.entry(task.reuse_origin).or_insert(0) += 1;
            retain_newest_fact(&mut newest_facts, fact, detail_limit.get());
        }
        newest_facts.sort_by(compare_explanation_fact_recency);

        Some(Self {
            profile,
            traced_node_count: active_node_count as u32,
            execution_record_count: task_record_count as u32,
            latest_execution_record_id,
            reuse_origin_counts,
            nodes: newest_facts
                .into_iter()
                .map(ExecutionHistoryNodeSummary::from_explanation_fact)
                .collect(),
        })
    }

    pub fn from_report(
        report: &ExecutionReport,
        profile: DiagnosticsTier,
        detail_limit: DetailLimit,
        retain_history_details: bool,
    ) -> Self {
        if !retain_history_details {
            return Self {
                profile,
                traced_node_count: report.task_count,
                execution_record_count: report.task_count,
                latest_execution_record_id: report.latest_execution_record_id,
                reuse_origin_counts: report.reuse_origin_counts.clone(),
                nodes: Vec::new(),
            };
        }

        let mut traced_node_count = 0_u32;
        let mut execution_record_count = 0_u32;
        let mut latest_execution_record_id = None;
        let mut reuse_origin_counts = ReuseOriginCounts::new();
        let mut nodes = Vec::new();

        for task in report
            .stages
            .iter()
            .flat_map(|stage| stage.task_records.iter())
        {
            traced_node_count += 1;
            execution_record_count += 1;
            latest_execution_record_id = Some(
                latest_execution_record_id.map_or(task.id.0, |current: u64| current.max(task.id.0)),
            );
            *reuse_origin_counts.entry(task.reuse_origin).or_insert(0) += 1;

            if nodes.len() < detail_limit.get() {
                nodes.push(ExecutionHistoryNodeSummary {
                    node: task.node,
                    execution_record_id: Some(task.id.0),
                    semantic_segment_id: Some(task.semantic_segment_id.0),
                    output_change: None,
                    memoized_origin: Some(task.memoized_origin),
                    reuse_basis: Some(task.reuse_basis.clone()),
                    reuse_origin: Some(task.reuse_origin),
                    persistent_correspondence_kind: None,
                    composition_region_count: 0,
                    reuse_certification_proof_count: 0,
                    changed_partition_count: 0,
                    causality_kind: None,
                });
            }
        }

        Self {
            profile,
            traced_node_count,
            execution_record_count,
            latest_execution_record_id,
            reuse_origin_counts,
            nodes,
        }
    }
}

impl ExecutionHistoryNodeSummary {
    fn from_explanation_fact(fact: &ExplanationFact) -> Self {
        let explanation = &fact.explanation;
        let runtime = &explanation
            .historical_artifact_record
            .as_ref()
            .expect("complete execution fact must retain its runtime record")
            .runtime;
        Self {
            node: explanation.node,
            execution_record_id: explanation.execution_record_id,
            semantic_segment_id: explanation.semantic_segment_id,
            output_change: explanation.output_change,
            memoized_origin: explanation.memoized_origin,
            reuse_basis: explanation.reuse_basis.clone(),
            reuse_origin: explanation.reuse_origin,
            persistent_correspondence_kind: runtime
                .reuse_boundary_authority()
                .and_then(|authority| authority.persistent_correspondence_kind()),
            composition_region_count: runtime
                .reuse_boundary_authority()
                .map(|authority| authority.composition_region_count())
                .unwrap_or(0),
            reuse_certification_proof_count: explanation
                .reuse_certification
                .as_ref()
                .map(|record| record.proofs.len() as u32)
                .unwrap_or(0),
            changed_partition_count: runtime.changed_partition_count(),
            causality_kind: explanation
                .causality
                .as_ref()
                .map(|cause| cause.kind.clone()),
        }
    }
}

fn retain_newest_fact<'a>(
    retained: &mut Vec<&'a ExplanationFact>,
    candidate: &'a ExplanationFact,
    limit: usize,
) {
    if limit == 0 {
        return;
    }
    if retained.len() < limit {
        retained.push(candidate);
        return;
    }
    let oldest = retained
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| compare_explanation_fact_recency(left, right))
        .map(|(index, _)| index)
        .expect("positive history detail limit must retain one candidate");
    if compare_explanation_fact_recency(&candidate, &retained[oldest]) == Ordering::Less {
        retained[oldest] = candidate;
    }
}

fn compare_explanation_fact_recency(left: &&ExplanationFact, right: &&ExplanationFact) -> Ordering {
    right
        .explanation
        .execution_record_id
        .cmp(&left.explanation.execution_record_id)
        .then_with(|| {
            right
                .explanation
                .semantic_segment_id
                .cmp(&left.explanation.semantic_segment_id)
        })
        .then_with(|| left.node.index().cmp(&right.node.index()))
        .then_with(|| left.node.generation().cmp(&right.node.generation()))
}
