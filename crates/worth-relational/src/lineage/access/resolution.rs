use std::collections::BTreeSet;
use worth_relational_contracts::internal_port::lineage::data::graph::LineageNodeFieldsPort as _;

use super::historical_walk::BranchScopedHistoricalResolutionRequest;
use crate::history::data::BranchId;
use crate::history::data::CommitId;
use crate::identity::data::LineageId;
use crate::lineage::access::LineageAccess;
use crate::lineage::data::{
    HistoricalLineageResolution, HistoricalResolutionRequest, RecordHistoryRequest,
};

pub(super) enum HistoricalTraversalScope {
    AuthoringBranch,
    SelectedRootReachable(SelectedRootReachableCommitScope),
}

pub(super) struct SelectedRootReachableCommitScope {
    pub(super) selected_commit_id: CommitId,
    pub(super) selected_branch_id: BranchId,
    pub(super) ancestry: Option<crate::history::CommitAncestryInspection>,
}

impl SelectedRootReachableCommitScope {
    pub(super) fn ancestry<'runtime>(
        &mut self,
        access: &LineageAccess<'runtime>,
    ) -> &crate::history::CommitAncestryInspection {
        self.ancestry.get_or_insert_with(|| {
            access
                .runtime
                .history()
                .inspect_commit_ancestry(self.selected_commit_id)
        })
    }

    pub(super) fn ancestry_work(&self) -> (usize, usize, usize) {
        self.ancestry.as_ref().map_or((0, 0, 0), |ancestry| {
            (
                ancestry.node_visits(),
                ancestry.catalog_probes(),
                ancestry.parent_edge_visits(),
            )
        })
    }

    fn relevant_branch_ids<'runtime>(
        &mut self,
        access: &LineageAccess<'runtime>,
    ) -> &BTreeSet<BranchId> {
        self.ancestry(access).authoring_branches()
    }

    pub(super) fn indexed_branches_for_lineages<'runtime>(
        &mut self,
        access: &LineageAccess<'runtime>,
        lineage_ids: &BTreeSet<LineageId>,
        sources_only: bool,
    ) -> (BTreeSet<BranchId>, usize) {
        let (selected_branch_only, branch_axis_probe_count) = access
            .runtime
            .lineage
            .indexed_lineages_are_exclusive_to_branch(
                lineage_ids,
                &self.selected_branch_id,
                sources_only,
            );
        let branches = if selected_branch_only {
            BTreeSet::from([self.selected_branch_id.clone()])
        } else {
            self.relevant_branch_ids(access).clone()
        };
        (branches, branch_axis_probe_count)
    }
}

impl<'runtime> LineageAccess<'runtime> {
    pub fn resolve_historical_lineage(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: HistoricalResolutionRequest,
    ) -> Result<HistoricalLineageResolution, crate::execution::RelationalExecutionDenialCause> {
        let branches = BTreeSet::from([request.branch_id.clone()]);
        let _events = super::event_demand::demand_historical_events(
            &self.runtime.lineage,
            execution,
            request.lineage_id,
            &branches,
            None,
        )?;
        Ok(self.resolve_historical_lineage_in_scope(
            BranchScopedHistoricalResolutionRequest::from_request(request),
            HistoricalTraversalScope::AuthoringBranch,
        ))
    }

    pub(super) fn resolve_historical_lineage_in_scope(
        &self,
        request: BranchScopedHistoricalResolutionRequest,
        mut traversal_scope: HistoricalTraversalScope,
    ) -> HistoricalLineageResolution {
        let walk = super::historical_walk::walk_lineage(
            &self.runtime.lineage,
            &request,
            &mut |lineage_ids| {
                self.event_positions_for_sources(
                    &request.branch_id,
                    lineage_ids,
                    &mut traversal_scope,
                )
            },
        );
        let ancestry_work = match &traversal_scope {
            HistoricalTraversalScope::AuthoringBranch => (0, 0, 0),
            HistoricalTraversalScope::SelectedRootReachable(scope) => scope.ancestry_work(),
        };
        let resolved = walk.finish(request, ancestry_work);
        let metrics = resolved.metrics;
        self.runtime
            .performance_access()
            .count_lineage_historical_resolution(
                metrics
                    .lineage_seed_index_probe_count
                    .saturating_add(metrics.reachable_event_index_probe_count),
                metrics.event_visit_count,
                metrics.traversed_event_count,
                metrics.reachable_commit_node_visits,
                metrics.reachable_commit_parent_edge_visits,
                metrics.reachable_commit_catalog_probes,
            );
        resolved
    }

    pub fn resolve_record_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: RecordHistoryRequest,
    ) -> Result<Option<HistoricalLineageResolution>, crate::execution::RelationalExecutionDenialCause>
    {
        let Some(lineage) = self.for_record(execution, request.entity_id)? else {
            return Ok(None);
        };
        Ok(Some(self.resolve_historical_lineage(
            execution,
            HistoricalResolutionRequest {
                branch_id: request.branch_id,
                lineage_id: (*lineage.owner_lineage_id()),
                boundedness_basis: request.boundedness_basis,
            },
        )?))
    }

    fn event_positions_for_sources(
        &self,
        branch_id: &BranchId,
        lineage_ids: &BTreeSet<LineageId>,
        traversal_scope: &mut HistoricalTraversalScope,
    ) -> (BTreeSet<usize>, usize) {
        match traversal_scope {
            HistoricalTraversalScope::AuthoringBranch => (
                self.runtime
                    .lineage
                    .branch_event_positions_for_sources(branch_id, lineage_ids),
                0,
            ),
            HistoricalTraversalScope::SelectedRootReachable(scope) => {
                let (relevant_branches, branch_axis_probe_count) =
                    scope.indexed_branches_for_lineages(self, lineage_ids, true);
                let (candidates, index_probe_count) = self
                    .runtime
                    .lineage
                    .branch_event_positions_for_lineages(&relevant_branches, lineage_ids, true);
                let candidate_count = candidates.len();
                let reachable = candidates
                    .into_iter()
                    .filter(|position| self.event_position_is_reachable(*position, scope))
                    .collect();
                (
                    reachable,
                    branch_axis_probe_count + index_probe_count + candidate_count,
                )
            }
        }
    }

    pub(super) fn event_position_is_reachable(
        &self,
        position: usize,
        scope: &mut SelectedRootReachableCommitScope,
    ) -> bool {
        let Some(_event) = self.runtime.lineage.event(position) else {
            return false;
        };
        let Some(publication_commit_id) = self.runtime.lineage.event_publication_commit(position)
        else {
            return false;
        };
        let classification = self
            .runtime
            .history()
            .classify_commit_in_ancestry(scope.ancestry(self), publication_commit_id);
        classification.posture() == crate::history::CommitAncestryPosture::Reachable
    }
}
