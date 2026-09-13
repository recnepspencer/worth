mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::diagnostics::policy::{DetailLimit, OrdinaryAccessLane};
use crate::diagnostics::profile::DiagnosticsTier;
use crate::presentation::metrics::GraphMetrics;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphSummary {
    pub profile: DiagnosticsTier,
    pub active_node_count: u32,
    pub arena_capacity: u32,
    pub tombstone_count: u32,
    pub clean_node_count: u32,
    pub maybe_stale_node_count: u32,
    pub dirty_node_count: u32,
    pub dependency_edge_count: u32,
    pub subscriber_edge_count: u32,
    pub nodes_with_partition_scopes: u32,
    pub nodes_with_trace_summary: u32,
    pub nodes_with_execution_record: u32,
    pub nodes_with_causality: u32,
    pub partition_interner_size: u32,
    pub sample_dirty_nodes: Vec<NodeId>,
    pub sample_nodes_with_execution_record: Vec<NodeId>,
    pub metrics: GraphMetrics,
}

impl GraphSummary {
    pub fn with_profile(&self, profile: DiagnosticsTier) -> Self {
        let mut cloned = self.clone();
        cloned.profile = profile;
        cloned
    }

    pub fn from_graph(
        graph: &SignalGraph,
        profile: DiagnosticsTier,
        detail_limit: DetailLimit,
        _lane: OrdinaryAccessLane,
    ) -> Self {
        let mut clean_node_count = 0_u32;
        let mut maybe_stale_node_count = 0_u32;
        let mut dirty_node_count = 0_u32;
        let mut dependency_edge_count = 0_u32;
        let mut subscriber_edge_count = 0_u32;
        let mut nodes_with_partition_scopes = 0_u32;
        let mut nodes_with_trace_summary = 0_u32;
        let mut nodes_with_execution_record = 0_u32;
        let mut nodes_with_causality = 0_u32;
        let mut sample_dirty_nodes = Vec::new();
        let mut sample_nodes_with_execution_record = Vec::new();

        for node in graph.diagnostic_nodes() {
            match node.state() {
                NodeState::Clean => clean_node_count += 1,
                NodeState::MaybeStale => maybe_stale_node_count += 1,
                NodeState::Dirty => {
                    dirty_node_count += 1;
                    if sample_dirty_nodes.len() < detail_limit.get() {
                        sample_dirty_nodes.push(node.node());
                    }
                }
            }
            let dependencies = node.dependencies();
            dependency_edge_count += dependencies.len() as u32;
            subscriber_edge_count += node.subscribers().len() as u32;
            if dependencies.iter().any(|edge| edge.scope_ref().is_some()) {
                nodes_with_partition_scopes += 1;
            }
            if node.node_runtime_artifact_state_present() {
                nodes_with_trace_summary += 1;
                if node.execution_record_present() {
                    nodes_with_execution_record += 1;
                    if sample_nodes_with_execution_record.len() < detail_limit.get() {
                        sample_nodes_with_execution_record.push(node.node());
                    }
                }
            }
            if node.causality_present() {
                nodes_with_causality += 1;
            }
        }

        let metrics = graph.observe().metrics();

        Self {
            profile,
            active_node_count: graph.active_node_count() as u32,
            arena_capacity: graph.arena_capacity() as u32,
            tombstone_count: graph.tombstone_count(),
            clean_node_count,
            maybe_stale_node_count,
            dirty_node_count,
            dependency_edge_count,
            subscriber_edge_count,
            nodes_with_partition_scopes,
            nodes_with_trace_summary,
            nodes_with_execution_record,
            nodes_with_causality,
            partition_interner_size: metrics.partition_interner_size as u32,
            sample_dirty_nodes,
            sample_nodes_with_execution_record,
            metrics,
        }
    }
}
