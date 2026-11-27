//! Cold-aware merge-preparation guards for named entity/relation lineage and
//! persistent group lineage after a merge base.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::lineage::data::events::{LineageEventKind, LineageEventRecord};
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;

use super::LineageSubsystem;

fn is_named_lineage_event(event: &LineageEventRecord) -> bool {
    event.name_birth().is_some()
        || event.kept_name().is_some()
        || matches!(
            event.kind(),
            LineageEventKind::Split | LineageEventKind::Merge
        )
}

impl LineageSubsystem {
    /// The first group lineage `b` published after `c` (merge refusal),
    /// scanning both cold paged `GROUP_LOG` and the resident tail.
    pub fn group_lineage_after(
        &self,
        b: &BranchId,
        c: CommitId,
    ) -> Result<Option<(CommitId, String)>, Cause> {
        self.state.read().group_lineage_after(b, c)
    }

    /// The first commit `b` published named lineage at after `c` (merge
    /// refusal), checking both routed cold event pages and the resident tail.
    pub fn named_lineage_after(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        b: &BranchId,
        c: CommitId,
    ) -> Result<Option<CommitId>, Failure> {
        let (bounds, mut earliest) = {
            let state = self.state.read();
            let bounds: BTreeMap<BranchId, CommitId> =
                state.ancestry(b, CommitId(u64::MAX)).into_iter().collect();
            let earliest = state
                .published_events
                .iter()
                .filter_map(|published| {
                    let (event, at) = (published.event.as_ref(), published.publication_commit_id);
                    let bound = bounds.get(event.branch_id());
                    (is_named_lineage_event(event)
                        && at > c
                        && bound.is_some_and(|bound| at <= *bound))
                    .then_some(at)
                })
                .min();
            (bounds, earliest)
        };
        let residency = self.event_residency.read().clone();
        if let Some(residency) = residency {
            let branches: BTreeSet<BranchId> = bounds.keys().cloned().collect();
            let pages = residency.demand(execution, None, Some(&branches))?;
            for page in &pages {
                for row in page.rows() {
                    let at = row.publication_commit_id;
                    if at > c
                        && is_named_lineage_event(&row.event)
                        && bounds
                            .get(row.event.branch_id())
                            .is_some_and(|bound| at <= *bound)
                    {
                        earliest = Some(earliest.map_or(at, |prev| prev.min(at)));
                    }
                }
            }
        }
        Ok(earliest)
    }
}
