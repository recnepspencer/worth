use std::collections::BTreeSet;

use crate::lineage::access::LineageAccess;
use crate::lineage::data::{
    LineageDivergenceMetrics, LineageDivergenceRequest, LineageDivergenceSummary,
    LineageGraphRequest,
};

impl<'runtime> LineageAccess<'runtime> {
    pub fn divergence_between_branches(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: LineageDivergenceRequest,
    ) -> Result<LineageDivergenceSummary, crate::execution::RelationalExecutionDenialCause> {
        let left_graph = self.graph(execution, LineageGraphRequest {
            branch_id: request.left_branch.clone(),
            traversal_basis:
                crate::lineage::data::LineageGraphTraversalBasis::FullBranchGraphMaterialization,
        })?;
        let right_graph = self.graph(execution, LineageGraphRequest {
            branch_id: request.right_branch.clone(),
            traversal_basis:
                crate::lineage::data::LineageGraphTraversalBasis::FullBranchGraphMaterialization,
        })?;
        self.runtime
            .performance_access()
            .count_lineage_branch_divergence(
                left_graph.metrics.event_count,
                right_graph.metrics.event_count,
                left_graph.metrics.node_count,
                right_graph.metrics.node_count,
            );
        let left_event_ids = left_graph
            .events
            .iter()
            .map(|event| event.event_id)
            .collect::<BTreeSet<_>>();
        let right_event_ids = right_graph
            .events
            .iter()
            .map(|event| event.event_id)
            .collect::<BTreeSet<_>>();
        let left_lineage_ids = left_graph
            .nodes
            .iter()
            .map(|node| node.lineage_id)
            .collect::<BTreeSet<_>>();
        let right_lineage_ids = right_graph
            .nodes
            .iter()
            .map(|node| node.lineage_id)
            .collect::<BTreeSet<_>>();
        let shared_lineage_ids = left_lineage_ids
            .intersection(&right_lineage_ids)
            .copied()
            .collect::<Vec<_>>();
        let shared_lineage_count = shared_lineage_ids.len();
        Ok(LineageDivergenceSummary {
            left_branch: request.left_branch,
            right_branch: request.right_branch,
            traversal_basis: request.traversal_basis,
            left_only_event_ids: left_event_ids
                .difference(&right_event_ids)
                .copied()
                .collect(),
            right_only_event_ids: right_event_ids
                .difference(&left_event_ids)
                .copied()
                .collect(),
            shared_lineage_ids,
            metrics: LineageDivergenceMetrics {
                left_event_count: left_graph.metrics.event_count,
                right_event_count: right_graph.metrics.event_count,
                left_node_count: left_graph.metrics.node_count,
                right_node_count: right_graph.metrics.node_count,
                shared_lineage_count,
            },
        })
    }
}
