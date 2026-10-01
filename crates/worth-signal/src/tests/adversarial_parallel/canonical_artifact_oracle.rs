use serde_json::json;

use crate::diagnostics::{policy::OrdinaryAccessLane, summary::GraphSummary};
use crate::facade::{DiagnosticsTier, NodeId, SignalGraph};

pub(super) fn canonical_runtime_artifacts(graph: &SignalGraph, node: NodeId) -> serde_json::Value {
    let explanation = graph.observe().explain(node).unwrap();
    let explanation_fact = graph.explanation_fact(node);
    let provenance = graph.provenance_fact(node).cloned();
    // Display summaries may retain the snapshot preceding a pending input.
    // Canonical epoch comparisons inspect the current graph owner instead.
    let diagnostics = GraphSummary::from_graph(
        graph,
        DiagnosticsTier::Development,
        graph
            .installed_runtime_policy()
            .retention_budget()
            .detail_limit,
        OrdinaryAccessLane,
    );
    let replay = graph
        .replay_events()
        .iter()
        .map(|event| {
            json!({
                "cursor": event.cursor.0,
                "kind": format!("{:?}", event.kind),
                "branch_id": event.branch_id.0,
                "snapshot_id": event.snapshot_id.map(|id| id.0),
                "node": event.node.map(|node| node.to_string()),
                "execution_record_id": event.execution_record_id,
                "semantic_segment_id": event.semantic_segment_id,
                "lineage_artifact_id": event.lineage_artifact_id.map(|id| id.0),
                "detail": event.detail,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "explanation": {
            "node": explanation.node.to_string(),
            "state": format!("{:?}", explanation.state),
            "execution_record_id": explanation.execution_record_id,
            "semantic_segment_id": explanation.semantic_segment_id,
            "upstream_count": explanation.upstream.len(),
            "propagation_suppressed": explanation.propagation_suppressed,
            "changed_region_count": explanation.changed_regions.len(),
            "output_change": explanation.output_change.map(|change| format!("{change:?}")),
            "fact_state": explanation_fact.map(|fact| fact.state.clone()),
            "fact_upstream_count": explanation_fact.map(|fact| fact.upstream_count),
        },
        "provenance": provenance,
        "replay": replay,
        "diagnostics": {
            "active_node_count": diagnostics.active_node_count,
            "clean_node_count": diagnostics.clean_node_count,
            "maybe_stale_node_count": diagnostics.maybe_stale_node_count,
            "dirty_node_count": diagnostics.dirty_node_count,
            "dependency_edge_count": diagnostics.dependency_edge_count,
            "subscriber_edge_count": diagnostics.subscriber_edge_count,
            "nodes_with_trace_summary": diagnostics.nodes_with_trace_summary,
            "nodes_with_execution_record": diagnostics.nodes_with_execution_record,
            "nodes_with_causality": diagnostics.nodes_with_causality,
            "partition_interner_size": diagnostics.partition_interner_size,
            "sample_dirty_nodes": diagnostics
                .sample_dirty_nodes
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            "sample_nodes_with_execution_record": diagnostics
                .sample_nodes_with_execution_record
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        },
    })
}
