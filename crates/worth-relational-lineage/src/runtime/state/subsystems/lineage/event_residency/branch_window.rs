//! Per-branch publication-interval index for bounded, zero-scratch-map cold
//! event scanning during merge preparation.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::durability::data::lineage_event_pages::{
    RelationalCheckpointLineageEventPage, RelationalCheckpointLineageEventRow,
};
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;

use super::{EventPageCell, RuntimeLineageEventResidency};

#[derive(Debug)]
struct BranchWindow {
    /// Legacy descriptors lacking publication-commit bounds (`last_publication_commit == 0`).
    legacy: Vec<Arc<EventPageCell>>,
    /// Deduplicated pages routed to this branch, sorted ascending by
    /// `(last_publication_commit, first_publication_commit, member)`.
    pages: Vec<Arc<EventPageCell>>,
    /// `suffix_min_first[i] = min_{j >= i} pages[j].descriptor.first_publication_commit`,
    /// enabling O(1) termination once every remaining page lies past `bound`.
    suffix_min_first: Vec<u64>,
}

#[derive(Debug, Default)]
pub(super) struct BranchEventWindowIndex {
    by_branch: BTreeMap<BranchId, BranchWindow>,
}

impl BranchEventWindowIndex {
    pub(super) fn build(cells: impl Iterator<Item = Arc<EventPageCell>>) -> Self {
        let mut raw: BTreeMap<BranchId, Vec<Arc<EventPageCell>>> = BTreeMap::new();
        for cell in cells {
            let mut seen_branches = BTreeSet::new();
            for route in &cell.descriptor.routes {
                if seen_branches.insert(route.branch_id.clone()) {
                    raw.entry(route.branch_id.clone())
                        .or_default()
                        .push(Arc::clone(&cell));
                }
            }
        }
        let mut by_branch = BTreeMap::new();
        for (branch, all_cells) in raw {
            let mut legacy = Vec::new();
            let mut pages = Vec::new();
            for cell in all_cells {
                if cell.descriptor.last_publication_commit == 0 {
                    legacy.push(cell);
                } else {
                    pages.push(cell);
                }
            }
            legacy.sort_by_key(|c| c.descriptor.member);
            pages.sort_by_key(|c| {
                (
                    c.descriptor.last_publication_commit,
                    c.descriptor.first_publication_commit,
                    c.descriptor.member,
                )
            });
            let mut suffix_min_first = vec![0u64; pages.len()];
            let mut running = u64::MAX;
            for (idx, cell) in pages.iter().enumerate().rev() {
                running = running.min(cell.descriptor.first_publication_commit);
                suffix_min_first[idx] = running;
            }
            by_branch.insert(
                branch,
                BranchWindow {
                    legacy,
                    pages,
                    suffix_min_first,
                },
            );
        }
        Self { by_branch }
    }
}

fn page_qualifies(
    descriptor: &RelationalCheckpointLineageEventPage,
    after: CommitId,
    bound: CommitId,
) -> bool {
    descriptor.last_publication_commit == 0
        || (descriptor.last_publication_commit > after.0
            && descriptor.first_publication_commit <= bound.0)
}

/// Canonical owner deduplication with `O(1)` scratch space: because
/// `branch_bounds` iterates in ascending `BranchId` order, a page routed to
/// multiple qualifying branches is visited strictly on its lexicographically
/// smallest qualifying branch.
fn has_earlier_qualifying_branch(
    descriptor: &RelationalCheckpointLineageEventPage,
    current: &BranchId,
    branch_bounds: &BTreeMap<BranchId, CommitId>,
    after: CommitId,
) -> bool {
    descriptor.routes.iter().any(|route| {
        &route.branch_id < current
            && branch_bounds
                .get(&route.branch_id)
                .is_some_and(|&bound| page_qualifies(descriptor, after, bound))
    })
}

impl RuntimeLineageEventResidency {
    /// Streams event pages overlapping `(after, branch_bounds[b]]` per branch
    /// with `O(log P_b)` pre-base seek, `O(1)` post-bound termination, `O(1)`
    /// deduplication scratch space, and strictly ephemeral page reads (zero
    /// resident retention).
    pub(in super::super) fn visit_after(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_bounds: &BTreeMap<BranchId, CommitId>,
        after: CommitId,
        mut visit: impl FnMut(&RelationalCheckpointLineageEventRow),
    ) -> Result<(), Failure> {
        crate::contracts::execution::request_custody::check_request(execution)
            .map_err(Failure::Request)?;
        for (branch, &bound) in branch_bounds {
            if bound <= after {
                continue;
            }
            let Some(window) = self.branch_windows.by_branch.get(branch) else {
                continue;
            };
            for cell in &window.legacy {
                if has_earlier_qualifying_branch(&cell.descriptor, branch, branch_bounds, after) {
                    continue;
                }
                let source = self.source().map_err(|cause| cell.storage_failure(cause))?;
                cell.visit_ephemeral(
                    execution,
                    source.as_ref(),
                    &self.commit_bindings,
                    &mut visit,
                )?;
            }
            let start = window
                .pages
                .partition_point(|p| p.descriptor.last_publication_commit <= after.0);
            for idx in start..window.pages.len() {
                if window.suffix_min_first[idx] > bound.0 {
                    break;
                }
                let cell = &window.pages[idx];
                if cell.descriptor.first_publication_commit > bound.0
                    || has_earlier_qualifying_branch(&cell.descriptor, branch, branch_bounds, after)
                {
                    continue;
                }
                let source = self.source().map_err(|cause| cell.storage_failure(cause))?;
                cell.visit_ephemeral(
                    execution,
                    source.as_ref(),
                    &self.commit_bindings,
                    &mut visit,
                )?;
            }
        }
        Ok(())
    }
}
