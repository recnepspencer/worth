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

impl<'runtime> LineageAccess<'runtime> {
    pub fn graph(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: LineageGraphRequest,
    ) -> Result<
        crate::lineage::data::LineageGraphSnapshot,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        crate::execution::request_custody::check_request(execution)?;
        let _events = self.runtime.lineage.demand_event_pages(
            execution,
            None,
            Some(&std::collections::BTreeSet::from([request
                .branch_id
                .clone()])),
        )?;
        let mut nodes = self.branch_nodes_snapshot(execution, &request.branch_id)?;
        nodes.sort_by_key(|node| node.lineage_id);
        let mut events = self.branch_events_snapshot(&request.branch_id);
        events.sort_by_key(|event| event.event_id);
        self.runtime
            .performance_access()
            .count_lineage_graph_snapshot_request(nodes.len(), events.len());
        let node_count = nodes.len();
        let event_count = events.len();
        let digest_basis = crate::lineage::data::LineageGraphDigestBasis::new(
            request.branch_id.clone(),
            request.traversal_basis,
            nodes.iter().map(|node| node.lineage_id).collect(),
            events.iter().map(|event| event.event_id).collect(),
            crate::lineage::data::LineageGraphDigestMode::ExactDigestCanonicalOrder,
        );
        Ok(crate::lineage::data::LineageGraphSnapshot::new(
            request.branch_id.clone(),
            nodes,
            events,
            request.traversal_basis,
            digest_basis,
            crate::lineage::data::LineageGraphMetrics {
                node_count,
                event_count,
            },
        ))
    }
}
