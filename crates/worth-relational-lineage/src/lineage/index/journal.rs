//! Tail-only undo journal that keeps one batch's ingestion atomic.
//!
//! A batch's commit is strictly above the index head, so ingesting it never
//! rewrites a timeline entry, edge row or tenure entry an earlier commit
//! wrote: it only appends, or replaces a row this same batch appended. The
//! journal therefore records, before a touched entry's first change, only how
//! long its histories were, and rollback truncates back to those lengths. No
//! prior entry, timeline or carrier set is ever copied.

use std::collections::HashSet;

use worth_relational_contracts::internal_port::identity::data::primitives::LineageId;
use worth_relational_contracts::internal_port::lineage::data::name_resolution::PersistentNameKey;
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameIdentity;

use super::member_groups::MemberTenures;
use super::{NameEntry, PersistentNameBranchIndex};

/// The lengths of one name's histories before a batch first touched it.
#[derive(Clone, Copy, Debug)]
pub(super) struct EntryMark {
    pub(super) timeline: usize,
    pub(super) edges: usize,
}

/// Where each touched history ended before the batch, and the lineages it
/// bound for the first time.
///
/// Complexity: O(1) per touched name, member tenure or binding to record,
/// and the same to restore; independent of every history's length.
#[derive(Default)]
pub(super) struct IngestionJournal {
    saved_names: HashSet<PersistentNameKey>,
    /// `None` marks a name this batch bore.
    names: Vec<(PersistentNameKey, Option<EntryMark>)>,
    /// Lineages first bound to a name or record in this batch.
    bound_lineage_names: Vec<LineageId>,
    /// Lineages first bound to a record in this batch; each appended one
    /// binding epoch to its record.
    bound_lineage_holders: Vec<LineageId>,
    saved_tenures: HashSet<(PersistentNameIdentity, String)>,
    /// A member's tenure length under one scheme; `None` marks a tenure
    /// this batch started.
    tenures: Vec<(PersistentNameIdentity, String, Option<usize>)>,
}

impl IngestionJournal {
    pub(super) fn save_name(&mut self, key: PersistentNameKey, prior: Option<&NameEntry>) {
        if self.saved_names.insert(key) {
            self.names.push((key, prior.map(NameEntry::mark)));
        }
    }

    pub(super) fn save_tenure(
        &mut self,
        member: PersistentNameIdentity,
        scheme: &str,
        prior: Option<&MemberTenures>,
    ) {
        if self.saved_tenures.insert((member, scheme.to_owned())) {
            let length = prior.and_then(|tenures| tenures.get(scheme)).map(Vec::len);
            self.tenures.push((member, scheme.to_owned(), length));
        }
    }

    pub(super) fn bound_lineage_name(&mut self, lineage: LineageId) {
        self.bound_lineage_names.push(lineage);
    }

    pub(super) fn bound_lineage_holder(&mut self, lineage: LineageId) {
        self.bound_lineage_holders.push(lineage);
    }

    /// Truncates every touched history back to its saved length, newest
    /// change first, removes what the batch bore, and unbinds every lineage
    /// this batch bound.
    pub(super) fn restore(self, index: &mut PersistentNameBranchIndex) {
        for (key, mark) in self.names.into_iter().rev() {
            match mark {
                Some(mark) => {
                    if let Some(entry) = index.names.get_mut(&key) {
                        entry.truncate_to(mark);
                    }
                }
                None => {
                    index.names.remove(&key);
                }
            }
        }
        for (member, scheme, length) in self.tenures.into_iter().rev() {
            let Some(tenures) = index.member_groups.get_mut(&member) else {
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
            if tenures.is_empty() {
                index.member_groups.remove(&member);
            }
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
