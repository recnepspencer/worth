use crate::lineage::access::LineageAccess;
use crate::lineage::data::{
    LineageGraphDigestBasis, LineageGraphDigestMode, LineageGraphMetrics, LineageGraphRequest,
    LineageGraphSnapshot,
};

impl<'runtime> LineageAccess<'runtime> {
    pub fn graph(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: LineageGraphRequest,
    ) -> Result<LineageGraphSnapshot, crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
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
        let digest_basis = LineageGraphDigestBasis::new(
            request.branch_id.clone(),
            request.traversal_basis,
            nodes.iter().map(|node| node.lineage_id).collect(),
            events.iter().map(|event| event.event_id).collect(),
            LineageGraphDigestMode::ExactDigestCanonicalOrder,
        );
        Ok(LineageGraphSnapshot::new(
            request.branch_id.clone(),
            nodes,
            events,
            request.traversal_basis,
            digest_basis,
            LineageGraphMetrics {
                node_count,
                event_count,
            },
        ))
    }
}
