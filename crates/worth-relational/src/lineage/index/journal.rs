//! Undo journal that keeps one batch's ingestion atomic.

use std::collections::HashSet;

use crate::identity::data::LineageId;
use crate::lineage::data::PersistentNameKey;

use super::{NameEntry, PersistentNameBranchIndex};

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
    bound_lineage_holders: Vec<LineageId>,
}

impl IngestionJournal {
    pub(super) fn save_name(&mut self, key: PersistentNameKey, prior: Option<&NameEntry>) {
        if self.saved_names.insert(key) {
            self.names.push((key, prior.cloned()));
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
        for lineage in self.bound_lineage_names {
            index.lineage_names.remove(&lineage);
        }
        for lineage in self.bound_lineage_holders {
            index.lineage_holders.remove(&lineage);
        }
    }
}
