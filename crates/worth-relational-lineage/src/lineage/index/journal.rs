//! Tail-only undo journal that keeps one batch's ingestion atomic over
//! copy-on-write tiered tables.
//!
//! A batch's commit is strictly above the index head, so ingesting it never
//! rewrites a timeline entry, edge row, tenure entry, or epoch earlier commits
//! wrote (other than closing a holder epoch's `ended_at`). Unshared resident
//! entries are journaled strictly by O(1) tail marks, newly inserted lineage
//! ids, and overwritten `ended_at` cells with zero history cloning; absent
//! delta slots roll back by removing the delta slot; shared fork slots
//! (`Arc::strong_count > 1`) roll back in O(1) via `restore_delta_arc`.

use std::collections::HashSet;
use std::sync::Arc;

use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port::identity::data::primitives::LineageId;
use worth_relational_contracts::internal_port::lineage::data::name_resolution::PersistentNameKey;
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameIdentity;

use super::member_groups::MemberTenures;
use super::{HolderEpoch, LineageHolder, NameEntry, PersistentNameBranchIndex};
use crate::lineage::tiered::TieredMap;

type Slot<V> = Option<Option<V>>;

/// The lengths of one name's histories before a batch first touched it.
#[derive(Clone, Copy, Debug)]
pub(super) struct EntryMark {
    pub(super) timeline: usize,
    pub(super) edges: usize,
}

enum NameUndo {
    RemoveFromDelta,
    RestoreArc(Arc<Option<NameEntry>>),
    Truncate(EntryMark),
}

enum MemberDeltaUndo {
    RemoveFromDelta,
    RestoreArc(Arc<Option<MemberTenures>>),
}

enum HolderDeltaUndo {
    RemoveFromDelta,
    RestoreArc(Arc<Option<Vec<HolderEpoch>>>),
    Truncate(usize),
}

/// Tail-only undo state for one batch's ingestion.
///
/// Complexity: O(1) per touched name, member tenure, lineage binding, or
/// holder epoch change to record and restore; independent of prior history.
#[derive(Default)]
pub(super) struct IngestionJournal {
    saved_names: HashSet<PersistentNameKey>,
    truncated_names: HashSet<PersistentNameKey>,
    names: Vec<(PersistentNameKey, NameUndo)>,
    added_lineage_commits: Vec<(PersistentNameKey, LineageId)>,
    saved_member_deltas: HashSet<PersistentNameIdentity>,
    member_deltas: Vec<(PersistentNameIdentity, MemberDeltaUndo)>,
    saved_tenures: HashSet<(PersistentNameIdentity, String)>,
    tenures: Vec<(PersistentNameIdentity, String, Option<usize>)>,
    lineage_names: Vec<(LineageId, Slot<PersistentNameKey>)>,
    lineage_holders: Vec<(LineageId, Slot<LineageHolder>)>,
    saved_holders: HashSet<LineageHolder>,
    truncated_holders: HashSet<LineageHolder>,
    holder_deltas: Vec<(LineageHolder, HolderDeltaUndo)>,
    holder_ended: Vec<(LineageHolder, usize, Option<CommitId>)>,
}

impl IngestionJournal {
    pub(super) fn save_name(
        &mut self,
        key: PersistentNameKey,
        names: &TieredMap<PersistentNameKey, NameEntry>,
    ) {
        if !self.saved_names.insert(key) {
            return;
        }
        let undo = match names.delta_arc(&key) {
            None => NameUndo::RemoveFromDelta,
            Some(arc) if Arc::strong_count(arc) > 1 => NameUndo::RestoreArc(Arc::clone(arc)),
            Some(arc) => match arc.as_ref() {
                Some(entry) => {
                    self.truncated_names.insert(key);
                    NameUndo::Truncate(entry.mark())
                }
                None => NameUndo::RestoreArc(Arc::clone(arc)),
            },
        };
        self.names.push((key, undo));
    }

    pub(super) fn record_added_lineage_commit(
        &mut self,
        key: PersistentNameKey,
        lineage: LineageId,
    ) {
        if self.truncated_names.contains(&key) {
            self.added_lineage_commits.push((key, lineage));
        }
    }

    pub(super) fn save_tenure(
        &mut self,
        member: PersistentNameIdentity,
        scheme: &str,
        members: &TieredMap<PersistentNameIdentity, MemberTenures>,
    ) {
        if self.saved_member_deltas.contains(&member) {
            return;
        }
        match members.delta_arc(&member) {
            None => {
                self.saved_member_deltas.insert(member);
                self.member_deltas
                    .push((member, MemberDeltaUndo::RemoveFromDelta));
            }
            Some(arc) if Arc::strong_count(arc) > 1 || arc.is_none() => {
                self.saved_member_deltas.insert(member);
                self.member_deltas
                    .push((member, MemberDeltaUndo::RestoreArc(Arc::clone(arc))));
            }
            Some(arc) => {
                if self.saved_tenures.insert((member, scheme.to_owned())) {
                    let length = arc
                        .as_ref()
                        .as_ref()
                        .and_then(|tenures| tenures.get(scheme))
                        .map(Vec::len);
                    self.tenures.push((member, scheme.to_owned(), length));
                }
            }
        }
    }

    /// A lineage is bound once per batch at most: it is saved unconditionally.
    pub(super) fn save_lineage_name(
        &mut self,
        lineage: LineageId,
        names: &TieredMap<LineageId, PersistentNameKey>,
    ) {
        self.lineage_names.push((lineage, names.slot(&lineage)));
    }

    pub(super) fn save_lineage_holder(
        &mut self,
        lineage: LineageId,
        holders: &TieredMap<LineageId, LineageHolder>,
    ) {
        self.lineage_holders.push((lineage, holders.slot(&lineage)));
    }

    pub(super) fn save_holder_lineages(
        &mut self,
        holder: LineageHolder,
        epochs: &TieredMap<LineageHolder, Vec<HolderEpoch>>,
    ) {
        if !self.saved_holders.insert(holder) {
            return;
        }
        let undo = match epochs.delta_arc(&holder) {
            None => HolderDeltaUndo::RemoveFromDelta,
            Some(arc) if Arc::strong_count(arc) > 1 => HolderDeltaUndo::RestoreArc(Arc::clone(arc)),
            Some(arc) => match arc.as_ref() {
                Some(vec) => {
                    self.truncated_holders.insert(holder);
                    HolderDeltaUndo::Truncate(vec.len())
                }
                None => HolderDeltaUndo::RestoreArc(Arc::clone(arc)),
            },
        };
        self.holder_deltas.push((holder, undo));
    }

    pub(super) fn record_holder_ended(
        &mut self,
        holder: LineageHolder,
        index: usize,
        prior_ended_at: Option<CommitId>,
    ) {
        if self.truncated_holders.contains(&holder) {
            self.holder_ended.push((holder, index, prior_ended_at));
        }
    }

    /// Truncates every touched resident history back to its saved length,
    /// removes newly inserted lineage commits and delta slots, restores
    /// overwritten `ended_at` markers, and restores shared arcs in O(batch).
    pub(super) fn restore(self, index: &mut PersistentNameBranchIndex) {
        for (key, lineage) in self.added_lineage_commits.into_iter().rev() {
            if let Ok(Some(entry)) = index
                .names
                .get_mut_with(&key, NameEntry::duplicate_for_branch_fork)
            {
                entry.remove_lineage_commit(lineage);
            }
        }
        for (key, undo) in self.names.into_iter().rev() {
            match undo {
                NameUndo::RemoveFromDelta => index.names.remove_delta(&key),
                NameUndo::RestoreArc(arc) => index.names.restore_delta_arc(key, arc),
                NameUndo::Truncate(mark) => {
                    if let Ok(Some(entry)) = index
                        .names
                        .get_mut_with(&key, NameEntry::duplicate_for_branch_fork)
                    {
                        entry.truncate_to(mark);
                    }
                }
            }
        }
        for (member, scheme, length) in self.tenures.into_iter().rev() {
            let Ok(Some(tenures)) = index.member_groups.get_mut(&member) else {
                continue;
            };
            match length {
                Some(length) => {
                    if let Some(tenure) = tenures.get_mut(&scheme) {
                        tenure.truncate(length);
                    }
                }
                None => {
                    tenures.remove(&scheme);
                }
            }
        }
        for (member, undo) in self.member_deltas.into_iter().rev() {
            match undo {
                MemberDeltaUndo::RemoveFromDelta => index.member_groups.remove_delta(&member),
                MemberDeltaUndo::RestoreArc(arc) => {
                    index.member_groups.restore_delta_arc(member, arc);
                }
            }
        }
        for (key, slot) in self.lineage_names.into_iter().rev() {
            index.lineage_names.restore_slot(key, slot);
        }
        for (key, slot) in self.lineage_holders.into_iter().rev() {
            index.lineage_holders.restore_slot(key, slot);
        }
        for (holder, pos, prior_ended_at) in self.holder_ended.into_iter().rev() {
            if let Ok(Some(epochs)) = index.holder_lineages.get_mut(&holder)
                && let Some(epoch) = epochs.get_mut(pos)
            {
                epoch.ended_at = prior_ended_at;
            }
        }
        for (holder, undo) in self.holder_deltas.into_iter().rev() {
            match undo {
                HolderDeltaUndo::RemoveFromDelta => index.holder_lineages.remove_delta(&holder),
                HolderDeltaUndo::RestoreArc(arc) => {
                    index.holder_lineages.restore_delta_arc(holder, arc);
                }
                HolderDeltaUndo::Truncate(len) => {
                    if let Ok(Some(epochs)) = index.holder_lineages.get_mut(&holder) {
                        epochs.truncate(len);
                    }
                }
            }
        }
    }
}
