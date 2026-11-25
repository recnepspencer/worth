//! Undo journal that keeps one batch's ingestion atomic.

use std::collections::HashSet;

use worth_relational_contracts::internal_port::identity::data::primitives::LineageId;
use worth_relational_contracts::internal_port::lineage::data::name_resolution::PersistentNameKey;

use super::member_groups::MemberTenures;
use super::{NameEntry, PersistentNameBranchIndex};
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameIdentity;

/// Prior name entries one batch touched, saved before first change, and the
/// lineages it bound for the first time.
///
/// Complexity: O(touched entries) to record and to restore.
#[derive(Default)]
pub(super) struct IngestionJournal {
    saved_names: HashSet<PersistentNameKey>,
    names: Vec<(PersistentNameKey, Option<NameEntry>)>,
    /// Lineages first bound to a name or record in this batch.
    bound_lineage_names: Vec<LineageId>,
    /// Lineages first bound to a record in this batch; each appended one
    /// binding epoch to its record.
    bound_lineage_holders: Vec<LineageId>,
    saved_members: HashSet<PersistentNameIdentity>,
    members: Vec<(PersistentNameIdentity, Option<MemberTenures>)>,
}

impl IngestionJournal {
    pub(super) fn save_name(&mut self, key: PersistentNameKey, prior: Option<&NameEntry>) {
        if self.saved_names.insert(key) {
            self.names.push((key, prior.cloned()));
        }
    }

    pub(super) fn save_member(
        &mut self,
        member: PersistentNameIdentity,
        prior: Option<&MemberTenures>,
    ) {
        if self.saved_members.insert(member) {
            self.members.push((member, prior.cloned()));
        }
    }

    pub(super) fn bound_lineage_name(&mut self, lineage: LineageId) {
        self.bound_lineage_names.push(lineage);
    }

    pub(super) fn bound_lineage_holder(&mut self, lineage: LineageId) {
        self.bound_lineage_holders.push(lineage);
    }

    /// Puts every touched entry back, newest change first, and unbinds every
    /// lineage this batch bound.
    pub(super) fn restore(self, index: &mut PersistentNameBranchIndex) {
        for (key, prior) in self.names.into_iter().rev() {
            match prior {
                Some(entry) => index.names.insert(key, entry),
                None => index.names.remove(&key),
            };
        }
        for (key, prior) in self.members.into_iter().rev() {
            match prior {
                Some(tenure) => index.member_groups.insert(key, tenure),
                None => index.member_groups.remove(&key),
            };
        }
        for lineage in self.bound_lineage_names {
            index.lineage_names.remove(&lineage);
        }
        for lineage in self.bound_lineage_holders.into_iter().rev() {
            let Some(holder) = index.lineage_holders.remove(&lineage) else {
                continue;
            };
            let Some(epochs) = index.holder_lineages.get_mut(&holder) else {
                continue;
            };
            if epochs.last().is_some_and(|(_, last)| *last == lineage) {
                epochs.pop();
            }
            if epochs.is_empty() {
                index.holder_lineages.remove(&holder);
            }
        }
    }
}
