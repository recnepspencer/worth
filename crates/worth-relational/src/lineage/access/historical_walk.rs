//! Ordered event traversal after the caller admits and pins the selected event scope.
use crate::history::data::BranchId;
use crate::identity::data::LineageId;
use crate::lineage::data::{
    HistoricalLineageResolution, HistoricalLineageResolutionDigestBasis,
    HistoricalLineageResolutionMetrics, HistoricalResolutionBoundednessBasis,
    HistoricalResolutionDigestMode, HistoricalResolutionRequest, HistoricalResolutionTrace,
    LineageEventKind,
};
use crate::runtime::state::subsystems::lineage::LineageSubsystem;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub(super) struct BranchScopedHistoricalResolutionRequest {
    pub(super) branch_id: BranchId,
    pub(super) lineage_id: LineageId,
    pub(super) boundedness_basis: HistoricalResolutionBoundednessBasis,
    pub(super) lineage_seed_index_probe_count: usize,
    pub(super) lineage_seed_event_visit_count: usize,
}

impl BranchScopedHistoricalResolutionRequest {
    pub(super) fn from_request(request: HistoricalResolutionRequest) -> Self {
        Self {
            branch_id: request.branch_id,
            lineage_id: request.lineage_id,
            boundedness_basis: request.boundedness_basis,
            lineage_seed_index_probe_count: 0,
            lineage_seed_event_visit_count: 0,
        }
    }
}

pub(super) struct HistoricalLineageWalk {
    resolved: BTreeSet<LineageId>,
    traversed_event_ids: Vec<u64>,
    event_visit_count: usize,
    reachable_event_index_probe_count: usize,
}

pub(super) fn walk_lineage(
    lineage: &LineageSubsystem,
    request: &BranchScopedHistoricalResolutionRequest,
    positions_for_sources: &mut dyn FnMut(&BTreeSet<LineageId>) -> (BTreeSet<usize>, usize),
) -> HistoricalLineageWalk {
    let mut current = BTreeSet::from([request.lineage_id]);
    let mut reached = BTreeSet::from([request.lineage_id]);
    let (scheduled_event_positions, mut reachable_event_index_probe_count) =
        positions_for_sources(&current);
    let mut scheduled_event_positions =
        schedule_event_positions(lineage, scheduled_event_positions);
    let mut visited_event_positions = BTreeSet::new();
    let mut traversed_event_ids = Vec::new();
    let mut event_visit_count = request.lineage_seed_event_visit_count;

    while let Some((event_id, position)) = scheduled_event_positions.first().copied() {
        scheduled_event_positions.remove(&(event_id, position));
        if !visited_event_positions.insert(position) {
            continue;
        }
        event_visit_count += 1;
        let event = lineage
                .event(position)
                .expect(/* Proof: Historical event demand pins the selected event pages before traversal; event-index entries are installed together with their published events. */ "lineage index references a published event");
        let source_matches = match event.kind() {
            LineageEventKind::Restore => event
                .sources()
                .iter()
                .any(|source| reached.contains(source)),
            _ => event
                .sources()
                .iter()
                .any(|source| current.contains(source)),
        };
        if !source_matches {
            continue;
        }
        match event.kind() {
            LineageEventKind::Replace
            | LineageEventKind::Split
            | LineageEventKind::Merge
            | LineageEventKind::Restore => {
                traversed_event_ids.push(event.event_id());
                for source in event.sources() {
                    current.remove(source);
                }
                let new_targets = event.targets().iter().copied().collect::<BTreeSet<_>>();
                current.extend(new_targets.iter().copied());
                reached.extend(new_targets.iter().copied());
                let (new_positions, index_probe_count) = positions_for_sources(&new_targets);
                scheduled_event_positions.extend(schedule_event_positions(lineage, new_positions));
                reachable_event_index_probe_count += index_probe_count;
            }
            LineageEventKind::Create | LineageEventKind::Retire => {}
        }
    }
    HistoricalLineageWalk {
        resolved: current,
        traversed_event_ids,
        event_visit_count,
        reachable_event_index_probe_count,
    }
}

impl HistoricalLineageWalk {
    pub(super) fn finish(
        self,
        request: BranchScopedHistoricalResolutionRequest,
        (
            reachable_commit_node_visits,
            reachable_commit_catalog_probes,
            reachable_commit_parent_edge_visits,
        ): (usize, usize, usize),
    ) -> HistoricalLineageResolution {
        let Self {
            resolved: current,
            traversed_event_ids,
            event_visit_count,
            reachable_event_index_probe_count,
        } = self;
        let traversed_event_count = traversed_event_ids.len();
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
}

fn schedule_event_positions(
    lineage: &LineageSubsystem,
    positions: BTreeSet<usize>,
) -> BTreeSet<(u64, usize)> {
    positions
        .into_iter()
        .filter_map(|position| {
            lineage
                .event(position)
                .map(|event| (event.event_id(), position))
        })
        .collect()
}
