use std::cmp::Ordering;

use crate::data::dependency::{CanonicalDependencies, DependencyEdge};
use crate::data::error::SignalError;
use crate::data::graph::signal_graph::DependencyTopologyDelta;
use crate::data::handle::NodeId;
use crate::data::proof::OrderedStreamItem;
use crate::data::telemetry::InvalidationPerformedCounter;

use super::DependencyReconciliationReport;
use crate::data::graph::signal_graph::SignalGraph;

impl SignalGraph {
    pub(crate) fn reconcile_dependencies(
        &mut self,
        node: NodeId,
        desired: &[DependencyEdge],
    ) -> Result<DependencyReconciliationReport, SignalError> {
        self.validate_handle(node)?;
        if self.raw_dependencies_of(node)? == desired {
            return Ok(DependencyReconciliationReport {
                unchanged: desired.len() as u32,
                ..DependencyReconciliationReport::default()
            });
        }
        let mut preflight =
            match super::preflight::canonicalize_and_preflight(self, &[(node, desired)]) {
                Ok(preflight) => preflight,
                Err(error) => {
                    self.invalidation_performed_counter_state()
                        .add(InvalidationPerformedCounter::RejectedTopologyMutations, 1);
                    return Err(error);
                }
            };
        let (_, desired) = preflight
            .pop()
            .expect("single dependency reconciliation must produce one plan");
        let desired = self.intern_dependency_edges(desired);
        let desired = desired.as_slice();
        let analysis = analyze_dependency_reconciliation(self.raw_dependencies_of(node)?, desired);
        if analysis.changed() {
            self.set_dependency_edges_sorted_with_delta(
                node,
                desired,
                analysis.delta,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary,
            )?;
            self.reconcile_dependency_subscribers(
                node,
                &analysis.current_sources,
                &analysis.desired_sources,
            )?;
            let performed = self.invalidation_performed_counter_state();
            performed.add(
                InvalidationPerformedCounter::TopologyRevisionRevalidations,
                1,
            );
        }
        self.debug_assert_bidirectional_consistency();
        Ok(analysis.report)
    }

    pub(crate) fn reconcile_dependencies_batch(
        &mut self,
        reconciliations: &[(NodeId, CanonicalDependencies)],
    ) -> Result<Vec<DependencyReconciliationReport>, SignalError> {
        self.reconcile_dependencies_batch_borrowed(
            &reconciliations
                .iter()
                .map(|(node, desired)| (*node, desired.as_slice()))
                .collect::<Vec<_>>(),
        )
    }

    pub(crate) fn reconcile_dependencies_batch_borrowed(
        &mut self,
        reconciliations: &[(NodeId, &[DependencyEdge])],
    ) -> Result<Vec<DependencyReconciliationReport>, SignalError> {
        let reconciliations =
            match super::preflight::canonicalize_and_preflight(self, reconciliations) {
                Ok(reconciliations) => reconciliations,
                Err(error) => {
                    self.invalidation_performed_counter_state()
                        .add(InvalidationPerformedCounter::RejectedTopologyMutations, 1);
                    return Err(error);
                }
            }
            .into_iter()
            .map(|(node, desired)| (node, self.intern_dependency_edges(desired)))
            .collect::<Vec<_>>();
        let mut reports = vec![DependencyReconciliationReport::default(); reconciliations.len()];
        let mut subscriber_ops = Vec::<SubscriberBatchOp>::new();

        for (index, reconciliation) in reconciliations.iter().enumerate() {
            let (node, desired) = reconciliation;
            let desired = desired.as_slice();
            let analysis =
                analyze_dependency_reconciliation(self.raw_dependencies_of(*node)?, desired);
            reports[index] = analysis.report;
            if !analysis.changed() {
                continue;
            }
            collect_subscriber_batch_ops(
                &mut subscriber_ops,
                *node,
                &analysis.current_sources,
                &analysis.desired_sources,
            );
            self.set_dependency_edges_sorted_with_delta(
                *node,
                desired,
                analysis.delta,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary,
            )?;
            let performed = self.invalidation_performed_counter_state();
            performed.add(
                InvalidationPerformedCounter::TopologyRevisionRevalidations,
                1,
            );
        }

        self.apply_subscriber_batch_ops(&subscriber_ops)?;
        self.debug_assert_bidirectional_consistency();
        Ok(reports)
    }

    pub(crate) fn set_dependency_edges_sorted(
        &mut self,
        node: NodeId,
        edges: &[DependencyEdge],
    ) -> Result<(), SignalError> {
        self.set_dependency_edges_sorted_with_work(
            node,
            edges,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
    }

    pub(in crate::data::graph) fn set_dependency_edges_sorted_with_work(
        &mut self,
        node: NodeId,
        edges: &[DependencyEdge],
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        let current = self.raw_dependencies_of(node)?;
        let delta = diff_dependency_topology(current, edges);
        if delta.added_edges.is_empty() && delta.removed_edges.is_empty() {
            return Ok(());
        }
        self.set_dependency_edges_sorted_with_delta(node, edges, delta, work)
    }
}

pub(super) fn compare_dependency_edges(left: &DependencyEdge, right: &DependencyEdge) -> Ordering {
    left.sort_key().cmp(&right.sort_key())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SubscriberBatchOp {
    pub(crate) source: NodeId,
    pub(crate) subscriber: NodeId,
    pub(crate) should_subscribe: bool,
}

impl OrderedStreamItem for SubscriberBatchOp {
    type OrderKey = ((u32, u32), (u32, u32), bool);

    fn order_key(&self) -> Self::OrderKey {
        (
            compare_dependency_identity_key(self.source),
            compare_dependency_identity_key(self.subscriber),
            !self.should_subscribe,
        )
    }
}

fn collect_subscriber_batch_ops(
    ops: &mut Vec<SubscriberBatchOp>,
    node: NodeId,
    current_sources: &[NodeId],
    desired_sources: &[NodeId],
) {
    let mut current_index = 0usize;
    let mut desired_index = 0usize;

    while current_index < current_sources.len() && desired_index < desired_sources.len() {
        let current = current_sources[current_index];
        let desired = desired_sources[desired_index];
        match compare_dependency_identity(current, desired) {
            Ordering::Less => {
                ops.push(SubscriberBatchOp {
                    source: current,
                    subscriber: node,
                    should_subscribe: false,
                });
                current_index += 1;
            }
            Ordering::Greater => {
                ops.push(SubscriberBatchOp {
                    source: desired,
                    subscriber: node,
                    should_subscribe: true,
                });
                desired_index += 1;
            }
            Ordering::Equal => {
                current_index += 1;
                desired_index += 1;
            }
        }
    }

    while current_index < current_sources.len() {
        ops.push(SubscriberBatchOp {
            source: current_sources[current_index],
            subscriber: node,
            should_subscribe: false,
        });
        current_index += 1;
    }

    while desired_index < desired_sources.len() {
        ops.push(SubscriberBatchOp {
            source: desired_sources[desired_index],
            subscriber: node,
            should_subscribe: true,
        });
        desired_index += 1;
    }
}

fn compare_dependency_identity_key(node: NodeId) -> (u32, u32) {
    (node.index(), node.generation())
}

#[derive(Debug)]
struct DependencyReconciliationAnalysis {
    report: DependencyReconciliationReport,
    current_sources: Vec<NodeId>,
    desired_sources: Vec<NodeId>,
    delta: DependencyTopologyDelta,
}

impl DependencyReconciliationAnalysis {
    fn changed(&self) -> bool {
        self.report.added != 0 || self.report.removed != 0
    }
}

fn analyze_dependency_reconciliation(
    current: &[DependencyEdge],
    desired: &[DependencyEdge],
) -> DependencyReconciliationAnalysis {
    let mut report = DependencyReconciliationReport::default();
    let mut current_sources = Vec::new();
    let mut desired_sources = Vec::new();
    let mut added_edges = Vec::new();
    let mut removed_edges = Vec::new();

    let mut current_index = 0usize;
    let mut desired_index = 0usize;
    let mut last_current_source = None;
    let mut last_desired_source = None;

    while current_index < current.len() && desired_index < desired.len() {
        let current_edge = &current[current_index];
        let desired_edge = &desired[desired_index];
        let current_source = current_edge.source();
        if last_current_source != Some(current_source) {
            current_sources.push(current_source);
            last_current_source = Some(current_source);
        }
        let desired_source = desired_edge.source();
        if last_desired_source != Some(desired_source) {
            desired_sources.push(desired_source);
            last_desired_source = Some(desired_source);
        }

        match compare_dependency_edges(current_edge, desired_edge) {
            Ordering::Less => {
                report.removed += 1;
                removed_edges.push(current_edge.clone());
                current_index += 1;
            }
            Ordering::Greater => {
                report.added += 1;
                added_edges.push(desired_edge.clone());
                desired_index += 1;
            }
            Ordering::Equal => {
                report.unchanged += 1;
                current_index += 1;
                desired_index += 1;
            }
        }
    }

    while current_index < current.len() {
        let current_edge = &current[current_index];
        let current_source = current_edge.source();
        if last_current_source != Some(current_source) {
            current_sources.push(current_source);
            last_current_source = Some(current_source);
        }
        report.removed += 1;
        removed_edges.push(current_edge.clone());
        current_index += 1;
    }

    while desired_index < desired.len() {
        let desired_edge = &desired[desired_index];
        let desired_source = desired_edge.source();
        if last_desired_source != Some(desired_source) {
            desired_sources.push(desired_source);
            last_desired_source = Some(desired_source);
        }
        report.added += 1;
        added_edges.push(desired_edge.clone());
        desired_index += 1;
    }

    DependencyReconciliationAnalysis {
        report,
        current_sources,
        desired_sources,
        delta: DependencyTopologyDelta {
            added_edges,
            removed_edges,
        },
    }
}

fn diff_dependency_topology(
    current: &[DependencyEdge],
    desired: &[DependencyEdge],
) -> DependencyTopologyDelta {
    analyze_dependency_reconciliation(current, desired).delta
}

fn compare_dependency_identity(left: NodeId, right: NodeId) -> Ordering {
    (left.index(), left.generation()).cmp(&(right.index(), right.generation()))
}
