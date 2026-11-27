//! Tail-only undo journal that keeps one batch's ingestion atomic over
//! copy-on-write tiered tables.
//!
//! A batch's commit is strictly above the index head, so ingesting it never
//! rewrites a timeline entry, edge row, tenure entry, or epoch earlier commits
//! wrote (other than closing a holder epoch's `ended_at`). Unshared resident
//! entries roll back in `O(log n)` per touched key by removing the batch's
//! newly inserted commit/epoch keys and restoring overwritten `ended_at` cells
//! with zero history cloning; absent delta slots roll back by removing the
//! delta slot; shared fork slots (any AVL path ancestor or value `Arc` with
//! `strong_count > 1`) roll back in `O(log N)` via `restore_delta_arc`.

use std::collections::HashSet;
use std::sync::Arc;

use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port::identity::data::primitives::LineageId;
use worth_relational_contracts::internal_port::lineage::data::name_resolution::PersistentNameKey;
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameIdentity;

use super::member_groups::MemberTenures;
use super::{HolderEpochs, LineageHolder, NameEntry, PersistentNameBranchIndex};
use crate::lineage::tiered::TieredMap;

type Slot<V> = Option<Option<V>>;

enum NameUndo {
    RemoveFromDelta,
    RestoreArc(Arc<Option<NameEntry>>),
    RemoveCommit(CommitId),
}

enum MemberDeltaUndo {
    RemoveFromDelta,
    RestoreArc(Arc<Option<MemberTenures>>),
}

enum TenureUndo {
    RemoveScheme,
    RemoveCommit(CommitId),
}

enum HolderDeltaUndo {
    RemoveFromDelta,
    RestoreArc(Arc<Option<HolderEpochs>>),
    InPlace,
}

/// Tail-only undo state for one batch's ingestion.
///
/// Complexity: O(log history) per touched name, member tenure, lineage binding,
/// or holder epoch change to record and restore; independent of prior history.
#[derive(Default)]
pub(super) struct IngestionJournal {
    saved_names: HashSet<PersistentNameKey>,
    in_place_names: HashSet<PersistentNameKey>,
    names: Vec<(PersistentNameKey, NameUndo)>,
    added_lineage_commits: Vec<(PersistentNameKey, LineageId)>,
    saved_member_deltas: HashSet<PersistentNameIdentity>,
    member_deltas: Vec<(PersistentNameIdentity, MemberDeltaUndo)>,
    saved_tenures: HashSet<(PersistentNameIdentity, String)>,
    tenures: Vec<(PersistentNameIdentity, String, TenureUndo)>,
    lineage_names: Vec<(LineageId, Slot<PersistentNameKey>)>,
    lineage_holders: Vec<(LineageId, Slot<LineageHolder>)>,
    saved_holders: HashSet<LineageHolder>,
    in_place_holders: HashSet<LineageHolder>,
    holder_deltas: Vec<(LineageHolder, HolderDeltaUndo)>,
    added_holder_epochs: Vec<(LineageHolder, (CommitId, LineageId))>,
    holder_ended: Vec<(LineageHolder, (CommitId, LineageId), Option<CommitId>)>,
}

impl IngestionJournal {
    pub(super) fn save_name(
        &mut self,
        key: PersistentNameKey,
        commit: CommitId,
        names: &TieredMap<PersistentNameKey, NameEntry>,
    ) {
        if !self.saved_names.insert(key) {
            return;
        }
        let undo = match names.delta_arc_and_shared(&key) {
            None => NameUndo::RemoveFromDelta,
            Some((arc, true)) => NameUndo::RestoreArc(Arc::clone(arc)),
            Some((arc, false)) => match arc.as_ref() {
                Some(_) => {
                    self.in_place_names.insert(key);
                    NameUndo::RemoveCommit(commit)
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
        if self.in_place_names.contains(&key) {
            self.added_lineage_commits.push((key, lineage));
        }
    }

    pub(super) fn save_tenure(
        &mut self,
        member: PersistentNameIdentity,
        scheme: &str,
        commit: CommitId,
        members: &TieredMap<PersistentNameIdentity, MemberTenures>,
    ) {
        if self.saved_member_deltas.contains(&member) {
            return;
        }
        match members.delta_arc_and_shared(&member) {
            None => {
                self.saved_member_deltas.insert(member);
                self.member_deltas
                    .push((member, MemberDeltaUndo::RemoveFromDelta));
            }
            Some((arc, true)) => {
                self.saved_member_deltas.insert(member);
                self.member_deltas
                    .push((member, MemberDeltaUndo::RestoreArc(Arc::clone(arc))));
            }
            Some((arc, false)) => match arc.as_ref() {
                None => {
                    self.saved_member_deltas.insert(member);
                    self.member_deltas
                        .push((member, MemberDeltaUndo::RestoreArc(Arc::clone(arc))));
                }
                Some(tenures) => {
                    if self.saved_tenures.insert((member, scheme.to_owned())) {
                        let undo = if tenures.contains_key(scheme) {
                            TenureUndo::RemoveCommit(commit)
                        } else {
                            TenureUndo::RemoveScheme
                        };
                        self.tenures.push((member, scheme.to_owned(), undo));
                    }
                }
            },
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
        epochs: &TieredMap<LineageHolder, HolderEpochs>,
    ) {
        if !self.saved_holders.insert(holder) {
            return;
        }
        let undo = match epochs.delta_arc_and_shared(&holder) {
            None => HolderDeltaUndo::RemoveFromDelta,
            Some((arc, true)) => HolderDeltaUndo::RestoreArc(Arc::clone(arc)),
            Some((arc, false)) => match arc.as_ref() {
                Some(_) => {
                    self.in_place_holders.insert(holder);
                    HolderDeltaUndo::InPlace
                }
                None => HolderDeltaUndo::RestoreArc(Arc::clone(arc)),
            },
        };
        self.holder_deltas.push((holder, undo));
    }

    pub(super) fn record_added_holder_epoch(
        &mut self,
        holder: LineageHolder,
        key: (CommitId, LineageId),
    ) {
        if self.in_place_holders.contains(&holder) {
            self.added_holder_epochs.push((holder, key));
        }
    }

    pub(super) fn record_holder_ended(
        &mut self,
        holder: LineageHolder,
        key: (CommitId, LineageId),
        prior_ended_at: Option<CommitId>,
    ) {
        if self.in_place_holders.contains(&holder) {
            self.holder_ended.push((holder, key, prior_ended_at));
        }
    }

    /// Reverses every touched resident entry, newly inserted lineage commit,
    /// tenure update, and holder epoch update in `O(batch * log history)`.
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
                NameUndo::RemoveCommit(commit) => {
                    if let Ok(Some(entry)) = index
                        .names
                        .get_mut_with(&key, NameEntry::duplicate_for_branch_fork)
                    {
                        entry.rollback_commit(commit);
                    }
                }
            }
        }
        for (member, scheme, undo) in self.tenures.into_iter().rev() {
            let Ok(Some(tenures)) = index.member_groups.get_mut(&member) else {
                continue;
            };
            match undo {
                TenureUndo::RemoveCommit(commit) => {
                    if let Some(tenure) = tenures.get_mut(&scheme) {
                        tenure.remove(&commit);
                    }
                }
                TenureUndo::RemoveScheme => {
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
        for (holder, key, prior_ended_at) in self.holder_ended.into_iter().rev() {
            if let Ok(Some(epochs)) = index.holder_lineages.get_mut(&holder) {
                let _ = epochs.set_ended_at(&key, prior_ended_at);
            }
        }
        for (holder, key) in self.added_holder_epochs.into_iter().rev() {
            if let Ok(Some(epochs)) = index.holder_lineages.get_mut(&holder) {
                epochs.remove_epoch(&key);
            }
        }
        for (holder, undo) in self.holder_deltas.into_iter().rev() {
            match undo {
                HolderDeltaUndo::RemoveFromDelta => index.holder_lineages.remove_delta(&holder),
                HolderDeltaUndo::RestoreArc(arc) => {
                    index.holder_lineages.restore_delta_arc(holder, arc);
                }
                HolderDeltaUndo::InPlace => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::history::data::coordinates::BranchId;
    use crate::contracts::lineage::data::name_resolution::PersistentNameTarget;
    use crate::lineage::index::{Carriers, LiveName, NameEntry, NameOrigin, NameState};
    use worth_foundational::facade::AggregateIdentity;

    fn make_key(byte: u8) -> PersistentNameKey {
        PersistentNameKey::Group(AggregateIdentity::from_bytes([byte; 32]))
    }

    fn make_entry(byte: u8, commit: u64) -> NameEntry {
        NameEntry::born(
            CommitId(commit),
            NameOrigin::Group {
                scheme_id: Some("s".into()),
            },
            LiveName {
                target: PersistentNameTarget::Group {
                    group: AggregateIdentity::from_bytes([byte; 32]),
                },
                carriers: Carriers::new(),
            },
        )
    }

    #[test]
    fn fork_shared_avl_descendant_is_detected_and_restores_original_arc() {
        let mut base = PersistentNameBranchIndex::new(BranchId("main".into()), None);
        for b in 1u8..=7 {
            base.names.insert(make_key(b), make_entry(b, u64::from(b)));
        }
        let mut forked = base.fork_to(BranchId("fork".into()), None);
        let target = make_key(1);
        let (orig_arc, shared) = forked
            .names
            .delta_arc_and_shared(&target)
            .expect("key present in forked delta");
        assert!(shared, "AVL ancestor sharing must mark descendant shared");
        assert_eq!(Arc::strong_count(orig_arc), 1);
        let orig_ptr = Arc::as_ptr(orig_arc);

        let mut journal = IngestionJournal::default();
        journal.save_name(target, CommitId(99), &forked.names);
        assert!(
            !journal.in_place_names.contains(&target),
            "fork-shared row must use RestoreArc, not RemoveCommit"
        );
        forked
            .names
            .get_mut_with(&target, NameEntry::duplicate_for_branch_fork)
            .unwrap()
            .unwrap()
            .set_state(CommitId(99), NameState::Ended);
        assert!(matches!(
            base.names.get(&target).unwrap().unwrap().current(),
            NameState::Live(_)
        ));

        journal.restore(&mut forked);
        let (restored_arc, _) = forked.names.delta_arc_and_shared(&target).unwrap();
        assert_eq!(Arc::as_ptr(restored_arc), orig_ptr);
        assert!(matches!(
            forked.names.get(&target).unwrap().unwrap().current(),
            NameState::Live(_)
        ));
    }

    #[test]
    fn fork_write_shares_historical_wide_split_successor_arc() {
        let mut base = PersistentNameBranchIndex::new(BranchId("main".into()), None);
        let target = make_key(1);
        let mut entry = make_entry(1, 1);
        for s in 2u8..=40 {
            entry.add_edge(CommitId(5), make_key(s));
        }
        base.names.insert(target, entry);
        let mut forked = base.fork_to(BranchId("fork".into()), None);
        let forked_entry = forked
            .names
            .get_mut_with(&target, NameEntry::duplicate_for_branch_fork)
            .unwrap()
            .unwrap();
        forked_entry.set_state(CommitId(10), NameState::Ended);
        forked_entry.add_edge(CommitId(10), make_key(99));

        let base_entry = base.names.get(&target).unwrap().unwrap();
        let fork_entry = forked.names.get(&target).unwrap().unwrap();
        assert!(
            Arc::ptr_eq(
                base_entry.forward_edges.get(&CommitId(5)).unwrap(),
                fork_entry.forward_edges.get(&CommitId(5)).unwrap(),
            ),
            "fork write must share prior split successor list behind Arc without copying"
        );
        assert_eq!(fork_entry.edges_at(CommitId(10)), &[make_key(99)]);
        assert!(base_entry.edges_at(CommitId(10)).is_empty());
    }
}
