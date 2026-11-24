//! Branch-local derived index from persistent names to their live records.
//!
//! The index is derived state, never authority: it is built solely from an
//! ordered log of canonical committed batches, and destroying it and calling
//! [`PersistentNameBranchIndex::rebuild_from_log`] reproduces it exactly.
//! Each name keeps its own liveness timeline, so a resolution at any retained
//! basis reads only that name's entry and its forward lineage path.

pub mod batch;
pub mod denial;
pub mod entry;
pub mod ingest_group;
pub mod ingest_lineage;
pub mod journal;
pub mod recorded_name;

use std::collections::HashMap;

pub use batch::PersistentNameCommittedBatch;
pub use denial::PersistentNameIndexDenial;
pub(crate) use entry::{LiveName, NameEntry, NameState};

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;

use journal::IngestionJournal;

/// The record a lineage was created for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LineageHolder {
    Entity(EntityId),
    Relation(RelationId),
}

/// Derived name-to-record index for one branch, with per-name history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistentNameBranchIndex {
    branch_id: BranchId,
    retention_floor: Option<CommitId>,
    head: Option<CommitId>,
    pub(crate) names: HashMap<PersistentNameKey, NameEntry>,
    lineage_names: HashMap<LineageId, PersistentNameKey>,
    lineage_holders: HashMap<LineageId, LineageHolder>,
    /// Every lineage a record was bound to, in binding-commit order: a
    /// history restore rebinds the exact original record to a fresh lineage,
    /// and a pinned read must still see the lineage bound at its basis.
    holder_lineages: HashMap<LineageHolder, Vec<(CommitId, LineageId)>>,
}

impl PersistentNameBranchIndex {
    /// An empty index for `branch_id`. Bases below `min_retained_commit`
    /// refuse as pruned.
    pub fn new(branch_id: BranchId, min_retained_commit: Option<CommitId>) -> Self {
        Self {
            branch_id,
            retention_floor: min_retained_commit,
            head: None,
            names: HashMap::new(),
            lineage_names: HashMap::new(),
            lineage_holders: HashMap::new(),
            holder_lineages: HashMap::new(),
        }
    }

    /// Destroys nothing and trusts nothing: builds a fresh index from the
    /// ordered canonical log alone.
    pub fn rebuild_from_log<'log>(
        branch_id: BranchId,
        min_retained_commit: Option<CommitId>,
        log: impl IntoIterator<Item = &'log PersistentNameCommittedBatch>,
    ) -> Result<Self, PersistentNameIndexDenial> {
        let mut index = Self::new(branch_id, min_retained_commit);
        for batch in log {
            index.apply(batch)?;
        }
        Ok(index)
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub const fn head(&self) -> Option<CommitId> {
        self.head
    }

    pub const fn retention_floor(&self) -> Option<CommitId> {
        self.retention_floor
    }

    /// Raises the retention floor; a lower floor never un-prunes history.
    pub fn advance_retention_floor(&mut self, floor: CommitId) {
        if self.retention_floor.is_none_or(|current| current < floor) {
            self.retention_floor = Some(floor);
        }
    }

    pub fn name_count(&self) -> usize {
        self.names.len()
    }

    /// Every name the index has seen born, in canonical order.
    ///
    /// Complexity: O(N log N); for audits and courts, never resolution.
    pub fn name_keys(&self) -> Vec<PersistentNameKey> {
        let mut keys: Vec<_> = self.names.keys().copied().collect();
        keys.sort_unstable();
        keys
    }

    /// This index re-addressed to `target`, a branch forked from this one.
    ///
    /// Callers fork only an index whose head is at or before the fork
    /// commit, so the copy holds exactly the inherited history; later
    /// batches for `target` then pass the branch check.
    ///
    /// Complexity: O(1); the caller pays the O(names) clone once per fork.
    pub(crate) fn rebranched(mut self, target: BranchId) -> Self {
        self.branch_id = target;
        self
    }

    /// The lineage `holder` was bound to at `basis`: the latest binding at or
    /// before it, so later restores never change a pinned answer.
    ///
    /// Complexity: O(1) expected probe plus O(log e) in that record's own
    /// binding epochs.
    pub(crate) fn lineage_of_holder(
        &self,
        holder: LineageHolder,
        basis: CommitId,
    ) -> Option<LineageId> {
        let epochs = self.holder_lineages.get(&holder)?;
        let after = epochs.partition_point(|(commit, _)| *commit <= basis);
        epochs
            .get(after.checked_sub(1)?)
            .map(|(_, lineage)| *lineage)
    }

    /// The name a lineage was bound to when its named record was created.
    ///
    /// Complexity: O(1) expected.
    pub(crate) fn name_of_lineage(&self, lineage: LineageId) -> Option<PersistentNameKey> {
        self.lineage_names.get(&lineage).copied()
    }

    /// Ingests one committed batch atomically: on refusal the index is left
    /// exactly as it was.
    ///
    /// Complexity: O(batch events + carrier declarations), independent of
    /// how many names or commits the index already holds.
    pub fn apply(
        &mut self,
        batch: &PersistentNameCommittedBatch,
    ) -> Result<(), PersistentNameIndexDenial> {
        if let Some(head) = self.head
            && batch.commit <= head
        {
            return Err(PersistentNameIndexDenial::NonMonotonicCommit {
                head,
                next: batch.commit,
            });
        }
        let mut journal = IngestionJournal::default();
        let ingested = self
            .ingest_lineage_events(batch, &mut journal)
            .and_then(|()| self.ingest_group_events(batch, &mut journal))
            .and_then(|()| self.ingest_carriers(batch, &mut journal));
        match ingested {
            Ok(()) => {
                self.head = Some(batch.commit);
                Ok(())
            }
            Err(denial) => {
                journal.restore(self);
                Err(denial)
            }
        }
    }

    fn ingest_carriers(
        &mut self,
        batch: &PersistentNameCommittedBatch,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        for (key, carriers) in &batch.carriers {
            let Some(live) = self.names.get(key).and_then(NameEntry::current_live) else {
                return Err(PersistentNameIndexDenial::InvalidTransition {
                    name: *key,
                    detail: "carriers declared for a name that is not live",
                });
            };
            if &live.carriers == carriers {
                continue;
            }
            let updated = LiveName {
                target: live.target,
                carriers: carriers.clone(),
            };
            self.entry_mut(*key, journal)
                .set_state(batch.commit, NameState::Live(updated));
        }
        Ok(())
    }

    /// Mutable access to an existing entry, journaled before first change.
    fn entry_mut(
        &mut self,
        key: PersistentNameKey,
        journal: &mut IngestionJournal,
    ) -> &mut NameEntry {
        journal.save_name(key, self.names.get(&key));
        self.names
            .get_mut(&key)
            .expect("callers check that the entry exists before mutating it")
    }

    fn insert_entry(
        &mut self,
        key: PersistentNameKey,
        entry: NameEntry,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        if self.names.contains_key(&key) {
            return Err(PersistentNameIndexDenial::DuplicateBirth { name: key });
        }
        journal.save_name(key, None);
        self.names.insert(key, entry);
        Ok(())
    }
}

#[doc(hidden)]
pub trait PersistentNameBranchIndexFixturePort {
    fn names_for_test(&self) -> &HashMap<PersistentNameKey, NameEntry>;
}

impl PersistentNameBranchIndexFixturePort for PersistentNameBranchIndex {
    fn names_for_test(&self) -> &HashMap<PersistentNameKey, NameEntry> {
        &self.names
    }
}
