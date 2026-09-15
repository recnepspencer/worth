use std::ops::DerefMut;

use crate::data::graph::SignalGraph;
use crate::diagnostics::policy::OrdinaryAccessLane;
use crate::diagnostics::summary::{ExecutionHistorySummary, GraphSummary};

/// Replaces the retained history and graph views with views computed from
/// the graph as it stands, exactly as a snapshot restore or a branch fork
/// does. Call it after a graph is rebuilt outside of one transaction (an
/// envelope import) so `history_now()` describes the rebuilt graph rather
/// than whichever execution happened to run last while building it.
///
/// Complexity: one pass over the live nodes and their retained records.
pub fn refresh_retained_diagnostics_views(mut graph: impl DerefMut<Target = SignalGraph>) {
    let graph = &mut *graph;
    let retention_budget = graph.installed_runtime_policy().retention_budget();
    let profile = graph.diagnostics_profile();
    let history = ExecutionHistorySummary::from_graph(
        graph,
        profile,
        retention_budget.detail_limit,
        retention_budget.retain_history_details,
        OrdinaryAccessLane,
    );
    let graph_summary = GraphSummary::from_graph(
        graph,
        profile,
        retention_budget.detail_limit,
        OrdinaryAccessLane,
    );
    graph
        .diagnostics_state_mut()
        .refresh_retained_views(history, graph_summary);
}
