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
use std::collections::BTreeSet;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;

use super::member_groups::{MemberTenure, MemberTenures};
use super::{
    HolderEpoch, NameEntry, NameState, PersistentNameBranchIndex, PersistentNameIndexDenial,
};

/// The first slot of a commit-ordered run still read at bases `>= floor`:
/// the latest entry at or before `floor`, or the first entry after it.
///
/// Complexity: O(log n).
pub(crate) fn first_retained<T>(run: &[(CommitId, T)], floor: CommitId) -> usize {
    run.partition_point(|(commit, _)| *commit <= floor)
        .saturating_sub(1)
}

/// Compacts one name entry for `floor`; `None` when nothing changes.
///
/// Complexity: O(t + e) in the entry's own timeline and edge runs.
pub(crate) fn compact_entry(entry: &NameEntry, floor: CommitId) -> Option<NameEntry> {
    let mut keep = first_retained(&entry.timeline, floor);
    // An ended state at the floor still reports the lineage that ended there.
    if matches!(entry.timeline.get(keep), Some((_, NameState::Ended))) {
        keep = entry.timeline[..keep]
            .iter()
            .rposition(|(_, state)| matches!(state, NameState::Live(_)))
            .unwrap_or(keep);
    }
    let mut timeline = entry.timeline[keep..].to_vec();
    let effective = first_retained(&timeline, floor);
    let mut stripped = false;
    for (_, state) in &mut timeline[..effective] {
        if let NameState::Live(live) = state
            && !live.carriers.is_empty()
        {
            live.carriers.clear();
            stripped = true;
        }
    }
    let read: BTreeSet<CommitId> = timeline.iter().map(|(commit, _)| *commit).collect();
    let forward_edges = entry
        .forward_edges
        .iter()
        .filter(|(commit, _)| *commit >= floor || read.contains(commit))
        .cloned()
        .collect::<Vec<_>>();
    if keep == 0 && !stripped && forward_edges.len() == entry.forward_edges.len() {
        return None;
    }
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
/// Complexity: O(e).
pub(crate) fn compact_epochs(epochs: &[HolderEpoch], floor: CommitId) -> Option<Vec<HolderEpoch>> {
    let keep = epochs
        .partition_point(|epoch| epoch.bound_at <= floor)
        .saturating_sub(1);
    (keep > 0).then(|| epochs[keep..].to_vec())
}

/// Compacts one member's tenures; `Some(None)` removes the member.
///
/// Complexity: O(S * t) over its S scheme tenures.
pub(crate) fn compact_tenures(
    tenures: &MemberTenures,
    floor: CommitId,
) -> Option<Option<MemberTenures>> {
    let mut changed = false;
    let mut compacted = MemberTenures::new();
    for (scheme, tenure) in tenures {
        let keep = first_retained(tenure, floor);
        let run: MemberTenure = tenure[keep..].to_vec();
        changed |= keep > 0;
        if let [(at, None)] = run.as_slice()
            && *at <= floor
        {
            changed = true;
            continue;
        }
        compacted.insert(scheme.clone(), run);
    }
    match (changed, compacted.is_empty()) {
        (false, _) => None,
        (true, true) => Some(None),
        (true, false) => Some(Some(compacted)),
    }
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
            let timeline: Vec<_> = entry
                .timeline
                .into_iter()
                .filter(|(commit, _)| *commit <= cutoff)
                .collect();
            if timeline.is_empty() {
                continue;
            }
            note(entry.born);
            for &(c, _) in &timeline {
                note(c);
            }
            let forward_edges: Vec<_> = entry
                .forward_edges
                .into_iter()
                .filter(|(commit, _)| *commit <= cutoff)
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
            let kept: Vec<_> = epochs
                .into_iter()
                .filter(|epoch| epoch.bound_at <= cutoff)
                .map(|epoch| HolderEpoch {
                    bound_at: epoch.bound_at,
                    lineage: epoch.lineage,
                    ended_at: epoch.ended_at.filter(|&end| end <= cutoff),
                })
                .collect();
            if kept.is_empty() {
                continue;
            }
            for epoch in &kept {
                note(epoch.bound_at);
                if let Some(end) = epoch.ended_at {
                    note(end);
                }
                out.lineage_holders.insert(epoch.lineage, holder);
                if let Some(name) = self.lineage_names.get(&epoch.lineage)? {
                    out.lineage_names.insert(epoch.lineage, *name);
                }
            }
            out.holder_lineages.insert(holder, kept);
        }
        for (member, tenures) in self.member_groups.collect_merged()? {
            let mut kept = MemberTenures::new();
            for (scheme, tenure) in tenures {
                let run: MemberTenure = tenure
                    .into_iter()
                    .filter(|(commit, _)| *commit <= cutoff)
                    .collect();
                if !run.is_empty() {
                    for &(commit, _) in &run {
                        note(commit);
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
            Vec<(super::LineageHolder, Vec<HolderEpoch>)>,
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
