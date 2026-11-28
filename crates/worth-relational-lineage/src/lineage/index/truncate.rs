//! Historical fork truncation of [`PersistentNameBranchIndex`], preserving
//! shared cold base provenance and untouched `CowMap` subtrees.

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;

use super::member_groups::{MemberTenure, MemberTenures};
use super::{
    HolderEpochs, NameEntry, PersistentNameBranchIndex, PersistentNameIndexDenial,
};

impl PersistentNameBranchIndex {
    /// Snapshots this index truncated to `<= cutoff` for a retained historical
    /// fork when full resident event history is unavailable (cold reopen),
    /// preserving shared cold base provenance and untouched `CowMap` subtrees.
    ///
    /// Refuses `BasisPruned` when `cutoff < retention_floor`.
    pub(crate) fn truncate_at(
        &self,
        target: BranchId,
        cutoff: CommitId,
    ) -> Result<Self, PersistentNameIndexDenial> {
        if let Some(retention_floor) = self.retention_floor
            && cutoff < retention_floor
        {
            return Err(PersistentNameIndexDenial::BasisPruned {
                basis: cutoff,
                retention_floor,
            });
        }
        let link = self.base().map(|base| base.forked(target.clone()));
        let mut out = self.fork_to(target, link);
        let mut min_commit: Option<CommitId> = None;
        let mut max_commit: Option<CommitId> = None;
        let mut note = |c: CommitId| {
            min_commit = Some(min_commit.map_or(c, |m| m.min(c)));
            max_commit = Some(max_commit.map_or(c, |m| m.max(c)));
        };
        for (key, entry) in self
            .names
            .collect_merged_with(NameEntry::duplicate_for_branch_fork)?
        {
            if entry.born > cutoff {
                out.names.remove_blind(&key);
                for &lineage in entry.lineage_commits.keys() {
                    out.lineage_names.remove_blind(&lineage);
                }
                continue;
            }
            let has_later = entry.timeline.get_max().is_some_and(|(c, _)| *c > cutoff)
                || entry
                    .forward_edges
                    .get_max()
                    .is_some_and(|(c, _)| *c > cutoff)
                || entry
                    .group_events
                    .get_max()
                    .is_some_and(|(c, _)| *c > cutoff)
                || entry.lineage_commits.values().any(|&c| c > cutoff);
            if !has_later {
                note(entry.born);
                if let Some((c, _)) = entry.timeline.get_min() {
                    note(*c);
                }
                if let Some((c, _)) = entry.timeline.get_max() {
                    note(*c);
                }
                continue;
            }
            for (&lineage, &commit) in &entry.lineage_commits {
                if commit > cutoff {
                    out.lineage_names.remove_blind(&lineage);
                }
            }
            let timeline: im::OrdMap<_, _> = entry
                .timeline
                .range(..=cutoff)
                .map(|(commit, state)| (*commit, state.clone()))
                .collect();
            if timeline.is_empty() {
                out.names.remove_blind(&key);
                continue;
            }
            note(entry.born);
            for c in timeline.keys() {
                note(*c);
            }
            let forward_edges: im::OrdMap<_, _> = entry
                .forward_edges
                .range(..=cutoff)
                .map(|(commit, edges)| (*commit, edges.clone()))
                .collect();
            let lineage_commits = entry
                .lineage_commits
                .into_iter()
                .filter(|(_, commit)| *commit <= cutoff)
                .collect();
            out.names.insert(
                key,
                NameEntry {
                    origin: entry.origin,
                    born: entry.born,
                    timeline,
                    forward_edges,
                    group_events: entry
                        .group_events
                        .range(..=cutoff)
                        .map(|(&commit, events)| (commit, events.clone()))
                        .collect(),
                    lineage_commits,
                },
            );
        }
        for (holder, epochs) in self.holder_lineages.collect_merged()? {
            let mut has_later = epochs
                .0
                .get_max()
                .is_some_and(|((bound_at, _), _)| *bound_at > cutoff);
            let mut max_end: Option<CommitId> = None;
            for ended_at in epochs.0.values().flatten() {
                if *ended_at > cutoff {
                    has_later = true;
                } else {
                    max_end = Some(max_end.map_or(*ended_at, |m| m.max(*ended_at)));
                }
            }
            if !has_later {
                if let Some(((c, _), _)) = epochs.0.get_min() {
                    note(*c);
                }
                if let Some(((c, _), _)) = epochs.0.get_max() {
                    note(*c);
                }
                if let Some(end) = max_end {
                    note(end);
                }
                continue;
            }
            for (&(bound_at, lineage), _) in &epochs.0 {
                if bound_at > cutoff {
                    out.lineage_holders.remove_blind(&lineage);
                    out.lineage_names.remove_blind(&lineage);
                }
            }
            let kept: im::OrdMap<_, _> = epochs
                .0
                .range(..=(cutoff, LineageId(u64::MAX)))
                .map(|(&(bound_at, lineage), &ended_at)| {
                    ((bound_at, lineage), ended_at.filter(|&end| end <= cutoff))
                })
                .collect();
            if kept.is_empty() {
                out.holder_lineages.remove_blind(&holder);
                continue;
            }
            for (&(bound_at, _), &ended_at) in &kept {
                note(bound_at);
                if let Some(end) = ended_at {
                    note(end);
                }
            }
            out.holder_lineages.insert(holder, HolderEpochs(kept));
        }
        for (member, tenures) in self.member_groups.collect_merged()? {
            let has_later = tenures
                .values()
                .any(|t| t.get_max().is_some_and(|(c, _)| *c > cutoff));
            if !has_later {
                for tenure in tenures.values() {
                    if let Some((c, _)) = tenure.get_min() {
                        note(*c);
                    }
                    if let Some((c, _)) = tenure.get_max() {
                        note(*c);
                    }
                }
                continue;
            }
            let mut kept = MemberTenures::new();
            for (scheme, tenure) in tenures {
                let run: MemberTenure = tenure
                    .range(..=cutoff)
                    .map(|(commit, group)| (*commit, *group))
                    .collect();
                if !run.is_empty() {
                    for commit in run.keys() {
                        note(*commit);
                    }
                    kept.insert(scheme, run);
                }
            }
            if kept.is_empty() {
                out.member_groups.remove_blind(&member);
            } else {
                out.member_groups.insert(member, kept);
            }
        }
        out.earliest_commit = min_commit;
        out.head = max_commit;
        Ok(out)
    }
}
