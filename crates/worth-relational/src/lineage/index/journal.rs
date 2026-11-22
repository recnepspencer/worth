//! Undo journal that keeps one batch's ingestion atomic.

use std::collections::HashSet;

use crate::identity::data::LineageId;
use crate::lineage::data::PersistentNameKey;

use super::{LineageHolder, NameEntry, PersistentNameBranchIndex};

/// Prior values of everything one batch touched, saved before first change.
///
/// Complexity: O(touched entries) to record and to restore.
#[derive(Default)]
pub(super) struct IngestionJournal {
    saved_names: HashSet<PersistentNameKey>,
    names: Vec<(PersistentNameKey, Option<NameEntry>)>,
    lineage_names: Vec<(LineageId, Option<PersistentNameKey>)>,
    lineage_holders: Vec<(LineageId, Option<LineageHolder>)>,
}

impl IngestionJournal {
    pub(super) fn save_name(&mut self, key: PersistentNameKey, prior: Option<&NameEntry>) {
        if self.saved_names.insert(key) {
            self.names.push((key, prior.cloned()));
        }
    }

    pub(super) fn save_lineage_name(
        &mut self,
        lineage: LineageId,
        prior: Option<PersistentNameKey>,
    ) {
        self.lineage_names.push((lineage, prior));
    }

    pub(super) fn save_lineage_holder(&mut self, lineage: LineageId, prior: Option<LineageHolder>) {
        self.lineage_holders.push((lineage, prior));
    }

    /// Puts every touched value back, newest change first.
    pub(super) fn restore(self, index: &mut PersistentNameBranchIndex) {
        for (key, prior) in self.names.into_iter().rev() {
            match prior {
                Some(entry) => index.names.insert(key, entry),
                None => index.names.remove(&key),
            };
        }
        for (lineage, prior) in self.lineage_names.into_iter().rev() {
            match prior {
                Some(key) => index.lineage_names.insert(lineage, key),
                None => index.lineage_names.remove(&lineage),
            };
        }
        for (lineage, prior) in self.lineage_holders.into_iter().rev() {
            match prior {
                Some(holder) => index.lineage_holders.insert(lineage, holder),
                None => index.lineage_holders.remove(&lineage),
            };
        }
    }
}
