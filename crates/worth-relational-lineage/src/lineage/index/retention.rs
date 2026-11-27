//! Retention-floor compaction of the derived name index.
//!
//! Raising the floor to `F` refuses every basis below `F` and keeps exactly
//! what bases at or above `F` read: each name's state in force at `F` and
//! everything after it, the forward edges those states can follow, each
//! record's binding in force at `F`, and each member tenure in force at `F`.
//! The pure kernels below compact one row in `O(row)`, independent of any
//! other row; the index applies them over every row once per raise.
//!
//! A name is never removed outright: a held name must still resolve
//! `Retired` at any retained basis. A name retired before `F` compacts to its
//! tombstone: its last live target (stripped of carriers), its retirement
//! and the edges recorded there.
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;

use super::entry::ordmap_prev;
use super::member_groups::{MemberTenure, MemberTenures};
use super::{
    HolderEpochs, NameEntry, NameState, PersistentNameBranchIndex, PersistentNameIndexDenial,
};

/// The first commit in a commit-keyed history still read at bases `>= floor`:
/// the latest entry at or before `floor`, or the earliest entry after it.
///
/// Complexity: single `O(log n)` B-tree descent.
pub(crate) fn first_retained_commit<T: Clone>(
    run: &im::OrdMap<CommitId, T>,
    floor: CommitId,
) -> Option<CommitId> {
    ordmap_prev(run, &floor)
        .map(|(c, _)| *c)
        .or_else(|| run.get_min().map(|(c, _)| *c))
}

/// Compacts one name entry for `floor`; `None` when nothing changes, with zero
/// map rebuilds on unchanged entries.
///
/// Complexity: `O(log t + log e)` on unchanged entries; `O(t + e)` when changed.
pub(crate) fn compact_entry(entry: &NameEntry, floor: CommitId) -> Option<NameEntry> {
    let mut keep = first_retained_commit(&entry.timeline, floor)?;
    // An ended state at the floor still reports the lineage that ended there.
    if matches!(entry.timeline.get(&keep), Some(NameState::Ended)) {
        if let Some((live_commit, _)) = entry
            .timeline
            .range(..keep)
            .rev()
            .find(|(_, state)| matches!(state, NameState::Live(_)))
        {
            keep = *live_commit;
        }
    }
    let dropped_timeline = entry.timeline.get_min().is_some_and(|(min, _)| *min < keep);
    let effective = first_retained_commit(&entry.timeline, floor);
    let needs_strip = effective.is_some_and(|eff| {
        keep < eff
            && entry.timeline.range(keep..eff).any(
                |(_, state)| matches!(state, NameState::Live(live) if !live.carriers.is_empty()),
            )
    });
    let dropped_edges = entry
        .forward_edges
        .range(..floor)
        .any(|(commit, _)| *commit < keep || !entry.timeline.contains_key(commit));
    if !dropped_timeline && !needs_strip && !dropped_edges {
        return None;
    }
    let mut timeline = if dropped_timeline {
        entry
            .timeline
            .range(keep..)
            .map(|(commit, state)| (*commit, state.clone()))
            .collect()
    } else {
        entry.timeline.clone()
    };
    if needs_strip && let Some(eff) = effective {
        let to_strip: Vec<CommitId> = timeline
            .range(..eff)
            .filter_map(|(commit, state)| match state {
                NameState::Live(live) if !live.carriers.is_empty() => Some(*commit),
                _ => None,
            })
            .collect();
        for commit in to_strip {
            if let Some(NameState::Live(live)) = timeline.get_mut(&commit) {
                live.carriers.clear();
            }
        }
    }
    let forward_edges = if dropped_edges {
        entry
            .forward_edges
            .iter()
            .filter(|(commit, _)| **commit >= floor || timeline.contains_key(commit))
            .map(|(commit, edges)| (*commit, edges.clone()))
            .collect()
    } else {
        entry.forward_edges.clone()
    };
    Some(NameEntry {
        origin: entry.origin.clone(),
        born: entry.born,
        timeline,
        forward_edges,
        lineage_commits: entry.lineage_commits.clone(),
    })
}

/// Compacts one record's binding epochs; `None` when nothing changes.
///
/// Complexity: `O(log e)` when unchanged; `O(e)` over retained suffix when changed.
pub(crate) fn compact_epochs(epochs: &HolderEpochs, floor: CommitId) -> Option<HolderEpochs> {
    let keep = ordmap_prev(&epochs.0, &(floor, LineageId(u64::MAX)))
        .map(|(k, _)| *k)
        .or_else(|| epochs.0.get_min().map(|(k, _)| *k))?;
    if !epochs.0.get_min().is_some_and(|(min, _)| *min < keep) {
        return None;
    }
    let retained: im::OrdMap<_, _> = epochs
        .0
        .range(keep..)
        .map(|(key, ended_at)| (*key, *ended_at))
        .collect();
    Some(HolderEpochs(retained))
}

fn tenure_change_kind(tenure: &MemberTenure, floor: CommitId) -> Option<(Option<CommitId>, bool)> {
    let Some(keep) = first_retained_commit(tenure, floor) else {
        return Some((None, true));
    };
    let dropped = tenure.get_min().is_some_and(|(min, _)| *min < keep);
    let mut retained_iter = tenure.range(keep..);
    let remove_scheme = matches!(retained_iter.next(), Some((at, None)) if *at <= floor)
        && retained_iter.next().is_none();
    if !dropped && !remove_scheme {
        None
    } else {
        Some((Some(keep), remove_scheme))
    }
}

/// Compacts one member's tenures; `Some(None)` removes the member. Unchanged
/// schemes and unchanged members reuse their persistent maps with zero rebuild.
///
/// Complexity: `O(S * log t)` when unchanged; rebuilds only changed schemes.
pub(crate) fn compact_tenures(
    tenures: &MemberTenures,
    floor: CommitId,
) -> Option<Option<MemberTenures>> {
    let mut updates: Vec<(&str, Option<CommitId>, bool)> = Vec::new();
    for (scheme, tenure) in tenures {
        if let Some((keep, remove_scheme)) = tenure_change_kind(tenure, floor) {
            updates.push((scheme.as_str(), keep, remove_scheme));
        }
    }
    if updates.is_empty() {
        return None;
    }
    let mut compacted = tenures.clone();
    for (scheme, keep, remove_scheme) in updates {
        if remove_scheme {
            compacted.remove(scheme);
        } else if let (Some(keep), Some(tenure)) = (keep, tenures.get(scheme)) {
            let run: MemberTenure = tenure
                .range(keep..)
                .map(|(commit, group)| (*commit, *group))
                .collect();
            compacted.insert(scheme.to_owned(), run);
        }
    }
    Some((!compacted.is_empty()).then_some(compacted))
}

impl PersistentNameBranchIndex {
    /// Raises the retention floor and compacts every row atomically: all cold
    /// pages are read and staged before any table or floor mutates, so a cold
    /// fault midway leaves the index and its prior floor completely untouched.
    ///
    /// Complexity: O(rows) once per raise (a maintenance pass, never a read).
    pub fn advance_retention_floor(
        &mut self,
        floor: CommitId,
    ) -> Result<(), PersistentNameIndexDenial> {
        if self.retention_floor.is_some_and(|current| current >= floor) {
            return Ok(());
        }
        let mut staged_names = Vec::new();
        for (key, entry) in self
            .names
            .collect_merged_with(NameEntry::duplicate_for_branch_fork)?
        {
            if let Some(compacted) = compact_entry(&entry, floor) {
                staged_names.push((key, compacted));
            }
        }
        let mut staged_epochs = Vec::new();
        for (holder, epochs) in self.holder_lineages.collect_merged()? {
            if let Some(compacted) = compact_epochs(&epochs, floor) {
                staged_epochs.push((holder, compacted));
            }
        }
        let mut staged_tenures = Vec::new();
        for (member, tenures) in self.member_groups.collect_merged()? {
            if let Some(change) = compact_tenures(&tenures, floor) {
                staged_tenures.push((member, change));
            }
        }
        // All cold reads succeeded; apply atomically.
        for (key, compacted) in staged_names {
            self.names.insert(key, compacted);
        }
        for (holder, compacted) in staged_epochs {
            self.holder_lineages.insert(holder, compacted);
        }
        for (member, change) in staged_tenures {
            match change {
                None => self.member_groups.remove_blind(&member),
                Some(compacted) => self.member_groups.insert(member, compacted),
            }
        }
        self.retention_floor = Some(floor);
        Ok(())
    }

    /// Snapshots this index truncated to `<= cutoff` for a retained historical
    /// fork when full resident event history is unavailable (cold reopen).
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
        let mut out = Self::new(target, self.retention_floor);
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
                continue;
            }
            let timeline: im::OrdMap<_, _> = entry
                .timeline
                .range(..=cutoff)
                .map(|(commit, state)| (*commit, state.clone()))
                .collect();
            if timeline.is_empty() {
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
                    lineage_commits,
                },
            );
        }
        for (holder, epochs) in self.holder_lineages.collect_merged()? {
            let kept: im::OrdMap<_, _> = epochs
                .0
                .range(..=(cutoff, LineageId(u64::MAX)))
                .map(|(&(bound_at, lineage), &ended_at)| {
                    ((bound_at, lineage), ended_at.filter(|&end| end <= cutoff))
                })
                .collect();
            if kept.is_empty() {
                continue;
            }
            for (&(bound_at, lineage), &ended_at) in &kept {
                note(bound_at);
                if let Some(end) = ended_at {
                    note(end);
                }
                out.lineage_holders.insert(lineage, holder);
                if let Some(name) = self.lineage_names.get(&lineage)? {
                    out.lineage_names.insert(lineage, *name);
                }
            }
            out.holder_lineages.insert(holder, HolderEpochs(kept));
        }
        for (member, tenures) in self.member_groups.collect_merged()? {
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
            if !kept.is_empty() {
                out.member_groups.insert(member, kept);
            }
        }
        out.earliest_commit = min_commit;
        out.head = max_commit;
        Ok(out)
    }

    /// Exports every live/retired non-group name entry and holder binding epoch
    /// for deriving a retained historical fork's naming basis from a truncated index.
    pub(crate) fn export_entries_for_naming_basis(
        &self,
    ) -> Result<
        (
            Vec<(PersistentNameKey, NameEntry)>,
            Vec<(LineageId, super::LineageHolder)>,
            Vec<(super::LineageHolder, HolderEpochs)>,
        ),
        PersistentNameIndexDenial,
    > {
        Ok((
            self.names
                .collect_merged_with(NameEntry::duplicate_for_branch_fork)?,
            self.lineage_holders.collect_merged()?,
            self.holder_lineages.collect_merged()?,
        ))
    }
}
