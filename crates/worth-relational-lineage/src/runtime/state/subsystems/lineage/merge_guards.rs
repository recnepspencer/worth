//! Cold-aware merge-preparation guards for named entity/relation lineage and
//! persistent group lineage after a merge base.

use std::collections::BTreeMap;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::lineage::data::events::{LineageEventKind, LineageEventRecord};
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;

use super::LineageSubsystem;

fn is_named(e: &LineageEventRecord) -> bool {
    use LineageEventKind::{Merge, Split};
    e.name_birth().is_some() || e.kept_name().is_some() || matches!(e.kind(), Split | Merge)
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
                .filter_map(|p| {
                    let (ev, at) = (p.event.as_ref(), p.publication_commit_id);
                    (is_named(ev) && at > c && bounds.get(ev.branch_id()).is_some_and(|b| at <= *b))
                        .then_some(at)
                })
                .min();
            (bounds, earliest)
        };
        if let Some(residency) = self.event_residency.read().clone()
            && residency.may_contain_named_events
        {
            let branch_bounds: BTreeMap<BranchId, CommitId> =
                bounds.into_iter().filter(|&(_, bound)| bound > c).collect();
            if !branch_bounds.is_empty() {
                residency.visit_after(execution, &branch_bounds, c, |row| {
                    let at = row.publication_commit_id;
                    if at > c
                        && is_named(&row.event)
                        && branch_bounds
                            .get(row.event.branch_id())
                            .is_some_and(|b| at <= *b)
                    {
                        earliest = Some(earliest.map_or(at, |prev| prev.min(at)));
                    }
                })?;
            }
        }
        Ok(earliest)
    }
}
