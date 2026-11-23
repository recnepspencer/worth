//! The persistent name a record carries, read from the branch-local derived
//! name index.

use crate::history::data::CommitId;
use crate::lineage::data::{LineageRecordTarget, PersistentNameOfRecord};
use crate::lineage::index::{LineageHolder, NameState, PersistentNameBranchIndex};

impl PersistentNameBranchIndex {
    /// The name `record` carries at `basis`.
    ///
    /// `Live` when the name designates exactly this record's lineage at
    /// `basis`; `Ended` when the record's name has since moved on or ended;
    /// `Unnamed` when no name was bound to the record's lineage at or before
    /// `basis`.
    ///
    /// Complexity: O(1) expected probes plus O(log t) in that one name's
    /// timeline for a live record, and O(t) in it for an ended one; total
    /// names and unrelated commits never enter the cost.
    pub fn name_of_record(
        &self,
        record: LineageRecordTarget,
        basis: CommitId,
    ) -> PersistentNameOfRecord {
        let holder = match record {
            LineageRecordTarget::Entity(entity) => LineageHolder::Entity(entity),
            LineageRecordTarget::Relation(relation) => LineageHolder::Relation(relation),
        };
        let Some(lineage) = self.lineage_of_holder(holder) else {
            return PersistentNameOfRecord::Unnamed;
        };
        let Some(key) = self.name_of_lineage(lineage) else {
            return PersistentNameOfRecord::Unnamed;
        };
        let Some(entry) = self.names.get(&key) else {
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
