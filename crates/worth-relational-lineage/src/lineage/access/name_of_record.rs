//! The persistent name a record carries, read from the branch-local derived
//! name index.

use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::lineage::data::name_resolution::PersistentNameOfRecord;
use crate::contracts::lineage::data::postures::LineageRecordTarget;
use crate::lineage::index::{LineageHolder, NameState, PersistentNameBranchIndex};
use crate::lineage::tiered::NamingReadScope;

impl PersistentNameBranchIndex {
    /// The name `record` carries at `basis`.
    ///
    /// `Live` when the name designates exactly this record's lineage at
    /// `basis`; `Ended` when the record's name has since moved on or ended;
    /// `Unnamed` when no name was bound to the record's lineage at or before
    /// `basis`. The record's lineage is the one bound to it at `basis`, so a
    /// later history restore onto a fresh lineage never changes the answer.
    ///
    /// Complexity: O(1) expected probes plus O(log t) in that one name's
    /// timeline for a live record, and O(t) in it for an ended one; total
    /// names and unrelated commits never enter the cost.
    pub fn name_of_record(
        &self,
        record: LineageRecordTarget,
        basis: CommitId,
    ) -> PersistentNameOfRecord {
        let scope = NamingReadScope::new();
        let answer = self.name_of_record_in(record, basis, &scope);
        settled(&scope, answer)
    }

    /// [`Self::name_of_record`] inside a read boundary's `scope`.
    pub(crate) fn name_of_record_in(
        &self,
        record: LineageRecordTarget,
        basis: CommitId,
        scope: &NamingReadScope,
    ) -> PersistentNameOfRecord {
        if let Some(retention_floor) = self.retention_floor()
            && basis < retention_floor
        {
            scope.fail_pruned(basis, retention_floor);
            return PersistentNameOfRecord::Unnamed;
        }
        let holder = match record {
            LineageRecordTarget::Entity(entity) => LineageHolder::Entity(entity),
            LineageRecordTarget::Relation(relation) => LineageHolder::Relation(relation),
        };
        let Some(lineage) = self.lineage_of_holder(holder, basis, scope) else {
            return PersistentNameOfRecord::Unnamed;
        };
        let Some(key) = self.name_of_lineage(lineage, scope) else {
            return PersistentNameOfRecord::Unnamed;
        };
        let Some(entry) = self.entry_in(&key, scope) else {
            return PersistentNameOfRecord::Unnamed;
        };
        match entry.state_at(basis) {
            None => PersistentNameOfRecord::Unnamed,
            Some((_, NameState::Live(live))) if live.target.lineage() == Some(lineage) => {
                PersistentNameOfRecord::Live { name: key, lineage }
            }
            Some(_) if entry.binds_lineage_by(lineage, basis) => {
                PersistentNameOfRecord::Ended { name: key, lineage }
            }
            Some(_) => PersistentNameOfRecord::Unnamed,
        }
    }
}

/// A standalone read's answer. Standalone reads serve resident indexes; a
/// cold-backed index is read only through a boundary that settles its own
/// scope, so a failed fault here is a broken invariant, never an answer.
pub(crate) fn settled<T>(scope: &NamingReadScope, answer: T) -> T {
    if let Err(cause) = scope.settle() {
        panic!("a standalone name index read faulted a damaged page: {cause:?}");
    }
    answer
}
