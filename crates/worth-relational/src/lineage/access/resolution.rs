use std::collections::BTreeSet;

use crate::history::data::BranchId;
use crate::history::data::CommitId;
use crate::identity::data::LineageId;
use crate::lineage::access::LineageAccess;
use crate::lineage::data::{
    HistoricalLineageResolution, HistoricalLineageResolutionDigestBasis,
    HistoricalLineageResolutionMetrics, HistoricalResolutionBoundednessBasis,
    HistoricalResolutionDigestMode, HistoricalResolutionRequest, HistoricalResolutionTrace,
    LineageEventKind, RecordHistoryRequest,
};

#[derive(Debug, Clone)]
pub(super) struct BranchScopedHistoricalResolutionRequest {
    pub(super) branch_id: BranchId,
    pub(super) lineage_id: LineageId,
    pub(super) boundedness_basis: HistoricalResolutionBoundednessBasis,
    pub(super) lineage_seed_index_probe_count: usize,
    pub(super) lineage_seed_event_visit_count: usize,
}

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
        self.ancestry(access);
        self.ancestry
            .as_ref()
            .expect("reachable ancestry was initialized")
            .authoring_branches()
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

impl BranchScopedHistoricalResolutionRequest {
    fn from_request(request: HistoricalResolutionRequest) -> Self {
        Self {
            branch_id: request.branch_id,
            lineage_id: request.lineage_id,
            boundedness_basis: request.boundedness_basis,
            lineage_seed_index_probe_count: 0,
            lineage_seed_event_visit_count: 0,
        }
    }
}

impl<'runtime> LineageAccess<'runtime> {
    pub fn resolve_historical_lineage(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: HistoricalResolutionRequest,
    ) -> Result<HistoricalLineageResolution, crate::execution::RelationalExecutionDenialCause> {
        let branches = BTreeSet::from([request.branch_id.clone()]);
        let _events =
            self.demand_historical_events(execution, request.lineage_id, &branches, None)?;
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
        let mut current = BTreeSet::from([request.lineage_id]);
        let (scheduled_event_positions, mut reachable_event_index_probe_count) =
            self.event_positions_for_sources(&request.branch_id, &current, &mut traversal_scope);
        let mut scheduled_event_positions =
            self.schedule_event_positions(scheduled_event_positions);
        let mut visited_event_positions = BTreeSet::new();
        let mut traversed_event_ids = Vec::new();
        let mut event_visit_count = request.lineage_seed_event_visit_count;

        while let Some((event_id, position)) = scheduled_event_positions.first().copied() {
            scheduled_event_positions.remove(&(event_id, position));
            if !visited_event_positions.insert(position) {
                continue;
            }
            event_visit_count += 1;
            let event = self
                .runtime
                .lineage
                .event(position)
                .expect("lineage index references a published event");
            if !event
                .sources()
                .iter()
                .any(|source| current.contains(source))
            {
                continue;
            }
            match event.kind() {
                LineageEventKind::Replace | LineageEventKind::Split | LineageEventKind::Merge => {
                    traversed_event_ids.push(event.event_id());
                    for source in event.sources() {
                        current.remove(source);
                    }
                    let new_targets = event.targets().iter().copied().collect::<BTreeSet<_>>();
                    current.extend(new_targets.iter().copied());
                    let (new_positions, index_probe_count) = self.event_positions_for_sources(
                        &request.branch_id,
                        &new_targets,
                        &mut traversal_scope,
                    );
                    scheduled_event_positions.extend(self.schedule_event_positions(new_positions));
                    reachable_event_index_probe_count += index_probe_count;
                }
                LineageEventKind::Create | LineageEventKind::Retire => {}
            }
        }
        let traversed_event_count = traversed_event_ids.len();
        let (
            reachable_commit_node_visits,
            reachable_commit_catalog_probes,
            reachable_commit_parent_edge_visits,
        ) = match &traversal_scope {
            HistoricalTraversalScope::AuthoringBranch => (0, 0, 0),
            HistoricalTraversalScope::SelectedRootReachable(scope) => scope.ancestry_work(),
        };
        self.runtime
            .performance_access()
            .count_lineage_historical_resolution(
                request
                    .lineage_seed_index_probe_count
                    .saturating_add(reachable_event_index_probe_count),
                event_visit_count,
                traversed_event_count,
                reachable_commit_node_visits,
                reachable_commit_parent_edge_visits,
                reachable_commit_catalog_probes,
            );

        let metrics = HistoricalLineageResolutionMetrics {
            traversed_event_count,
            event_visit_count,
            resolved_lineage_count: current.len(),
            lineage_seed_index_probe_count: request.lineage_seed_index_probe_count,
            reachable_event_index_probe_count,
            reachable_commit_node_visits,
            reachable_commit_parent_edge_visits,
            reachable_commit_catalog_probes,
        };
        let digest_basis = HistoricalLineageResolutionDigestBasis::new(
            request.branch_id.clone(),
            request.lineage_id,
            current.iter().copied().collect(),
            traversed_event_ids.clone(),
            request.boundedness_basis,
            HistoricalResolutionDigestMode::ExactDigestCanonicalOrder,
        );

        HistoricalLineageResolution::new(
            request.branch_id,
            request.lineage_id,
            current.iter().copied().collect(),
            request.boundedness_basis,
            traversed_event_ids.clone(),
            digest_basis.clone(),
            HistoricalResolutionTrace::new(
                traversed_event_ids,
                request.boundedness_basis,
                digest_basis,
                metrics,
            ),
            metrics,
        )
    }

    fn schedule_event_positions(&self, positions: BTreeSet<usize>) -> BTreeSet<(u64, usize)> {
        positions
            .into_iter()
            .filter_map(|position| {
                self.runtime
                    .lineage
                    .event(position)
                    .map(|event| (event.event_id(), position))
            })
            .collect()
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
                lineage_id: lineage.lineage_id,
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
