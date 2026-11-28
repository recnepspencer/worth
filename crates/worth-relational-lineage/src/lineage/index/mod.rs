//! Branch-local derived index from persistent names to their live records.
//!
//! The index is derived state, never authority: it is built solely from an
//! ordered log of canonical committed batches, and destroying it and calling
//! [`PersistentNameBranchIndex::rebuild_from_log`] reproduces it exactly.
//! Each name keeps its own liveness timeline, so a resolution at any retained
//! basis reads only that name's entry and its forward lineage path.
//!
//! Every table is a [`TieredMap`]: a runtime reopened from a checkpoint
//! reads its rows from cold pages on first touch, and readers borrow faulted
//! rows through a [`NamingReadScope`].

pub mod batch;
mod commit_basis;
pub mod codec;
pub mod denial;
pub mod entry;
pub mod ingest_group;
pub mod ingest_lineage;
pub mod journal;
pub mod member_groups;
pub mod preflight;
pub mod recorded_name;
pub mod retention;
pub mod truncate;

#[cfg(test)]
mod name_index_conformance_trace;

pub use batch::PersistentNameCommittedBatch;
pub use denial::PersistentNameIndexDenial;
pub(crate) use entry::{Carriers, HolderEpochs, LiveName, NameEntry, NameOrigin, NameState};

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::contracts::lineage::data::naming::PersistentNameIdentity;
use crate::lineage::naming_pages::tables;
use crate::lineage::tiered::{NamingBaseLink, NamingReadScope, TieredMap};

use journal::IngestionJournal;

/// The record a lineage was created for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum LineageHolder {
    Entity(EntityId),
    Relation(RelationId),
}

crate::lineage::clone_fences::assert_not_clone!(PersistentNameBranchIndex);

/// Derived name-to-record index for one branch, with per-name history.
///
/// Deliberately not `Clone`: a fork shares its tables copy-on-write through
/// [`Self::fork_to`], so an `O(names)` copy cannot be written by accident.
#[derive(Debug, PartialEq, Eq)]
pub struct PersistentNameBranchIndex {
    branch_id: BranchId,
    retention_floor: Option<CommitId>,
    earliest_commit: Option<CommitId>,
    head: Option<CommitId>,
    pub(crate) names: TieredMap<PersistentNameKey, NameEntry>,
    lineage_names: TieredMap<LineageId, PersistentNameKey>,
    lineage_holders: TieredMap<LineageId, LineageHolder>,
    /// Every lineage a record was bound to, in binding-commit order.
    holder_lineages: TieredMap<LineageHolder, HolderEpochs>,
    /// Per member name and group scheme: which group held it from each commit.
    member_groups: TieredMap<PersistentNameIdentity, member_groups::MemberTenures>,
}

impl PersistentNameBranchIndex {
    /// An empty index for `branch_id`. Bases below `min_retained_commit`
    /// refuse as pruned.
    pub fn new(branch_id: BranchId, min_retained_commit: Option<CommitId>) -> Self {
        Self::over(branch_id, min_retained_commit, None, None, None)
    }

    /// The index a checkpoint recorded: its scalars resident, its tables
    /// cold behind `base`.
    pub(crate) fn over(
        branch_id: BranchId,
        retention_floor: Option<CommitId>,
        earliest_commit: Option<CommitId>,
        head: Option<CommitId>,
        base: Option<NamingBaseLink>,
    ) -> Self {
        Self {
            branch_id,
            retention_floor,
            earliest_commit,
            head,
            names: TieredMap::over(tables::INDEX_NAME, base.clone()),
            lineage_names: TieredMap::over(tables::INDEX_LINEAGE_NAME, base.clone()),
            lineage_holders: TieredMap::over(tables::INDEX_LINEAGE_HOLDER, base.clone()),
            holder_lineages: TieredMap::over(tables::INDEX_HOLDER_LINEAGES, base.clone()),
            member_groups: TieredMap::over(tables::INDEX_MEMBER_GROUPS, base),
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

    pub const fn earliest_commit(&self) -> Option<CommitId> {
        self.earliest_commit
    }

    pub const fn head(&self) -> Option<CommitId> {
        self.head
    }

    pub const fn retention_floor(&self) -> Option<CommitId> {
        self.retention_floor
    }

    /// Every name the index holds, by a full scan.
    ///
    /// Complexity: O(N); for audits and courts, never resolution.
    pub fn name_count(&self) -> usize {
        self.name_keys().len()
    }

    /// Every name the index has seen born, in canonical order.
    ///
    /// Complexity: O(N log N); for audits and courts, never resolution.
    pub fn name_keys(&self) -> Vec<PersistentNameKey> {
        let mut keys = Vec::new();
        self.names
            .scan_merged(&[self.names.table()], |key, _| {
                keys.push(key);
                Ok(true)
            })
            .unwrap_or_else(|cause| panic!("audit read of a damaged naming page: {cause:?}"));
        keys.sort_unstable();
        keys
    }

    /// Complexity: O(1): tables are shared copy-on-write.
    pub(crate) fn fork_to(&self, target: BranchId, base: Option<NamingBaseLink>) -> Self {
        Self {
            branch_id: target,
            retention_floor: self.retention_floor,
            earliest_commit: self.earliest_commit,
            head: self.head,
            names: self.names.fork(base.clone()),
            lineage_names: self.lineage_names.fork(base.clone()),
            lineage_holders: self.lineage_holders.fork(base.clone()),
            holder_lineages: self.holder_lineages.fork(base.clone()),
            member_groups: self.member_groups.fork(base),
        }
    }

    /// Rebinds every table cleanly onto an acknowledged cold base, dropping
    /// resident deltas.
    pub(crate) fn rebind_clean(&mut self, base: NamingBaseLink) {
        self.names.rebind_clean(base.clone());
        self.lineage_names.rebind_clean(base.clone());
        self.lineage_holders.rebind_clean(base.clone());
        self.holder_lineages.rebind_clean(base.clone());
        self.member_groups.rebind_clean(base);
    }

    /// Rebinds existing cold bases onto `source` (and optional `checkpointed_root`).
    pub(crate) fn rebind_source(
        &mut self,
        source: &std::sync::Arc<dyn crate::lineage::tiered::ColdNamingSource>,
        root: Option<&BranchId>,
    ) {
        self.names.rebind_source(source, root);
        self.lineage_names.rebind_source(source, root);
        self.lineage_holders.rebind_source(source, root);
        self.holder_lineages.rebind_source(source, root);
        self.member_groups.rebind_source(source, root);
    }

    pub(crate) fn base(&self) -> Option<&NamingBaseLink> {
        self.names.base()
    }

    pub(crate) fn encoded_changes(&self) -> Vec<(Vec<u8>, Option<Vec<u8>>)> {
        let mut changes = self.names.encoded_changes();
        changes.extend(self.lineage_names.encoded_changes());
        changes.extend(self.lineage_holders.encoded_changes());
        changes.extend(self.holder_lineages.encoded_changes());
        changes.extend(self.member_groups.encoded_changes());
        changes
    }

    pub(crate) fn collect_encoded_in(
        &self,
        store: Option<&crate::lineage::naming_pages::store::ColdNamingStore>,
    ) -> Result<
        Vec<(Vec<u8>, Option<Vec<u8>>)>,
        crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    > {
        let mut rows = self.names.collect_encoded_in(store)?;
        rows.extend(self.lineage_names.collect_encoded_in(store)?);
        rows.extend(self.lineage_holders.collect_encoded_in(store)?);
        rows.extend(self.holder_lineages.collect_encoded_in(store)?);
        rows.extend(self.member_groups.collect_encoded_in(store)?);
        Ok(rows)
    }

    pub(crate) fn rebranched(mut self, target: BranchId) -> Self {
        self.branch_id = target;
        self
    }

    pub(crate) fn latest_holder_lineage_in(
        &self,
        holder: LineageHolder,
        scope: &NamingReadScope,
    ) -> Option<LineageId> {
        self.holder_lineages
            .get_in(&holder, scope)?
            .latest()
            .map(|(_, lineage, _)| lineage)
    }

    /// The lineage `holder` was bound to at `basis`: the latest binding at or
    /// before it, so later restores never change a pinned answer.
    ///
    /// Complexity: O(log H) probe plus single `O(log e)` predecessor descent.
    pub(crate) fn lineage_of_holder(
        &self,
        holder: LineageHolder,
        basis: CommitId,
        scope: &NamingReadScope,
    ) -> Option<LineageId> {
        self.holder_lineages
            .get_in(&holder, scope)?
            .lineage_at(basis)
    }

    /// The name a lineage was bound to when its named record was created.
    ///
    /// Complexity: O(log N) probe.
    pub(crate) fn name_of_lineage(
        &self,
        lineage: LineageId,
        scope: &NamingReadScope,
    ) -> Option<PersistentNameKey> {
        self.lineage_names.get_in(&lineage, scope).copied()
    }

    /// The entry of `key`, borrowed for `scope`.
    ///
    /// Complexity: O(log N) probe and at most one page fault.
    pub(crate) fn entry_in<'s>(
        &'s self,
        key: &PersistentNameKey,
        scope: &'s NamingReadScope,
    ) -> Option<&'s NameEntry> {
        self.names.get_in(key, scope)
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
                self.earliest_commit = Some(
                    self.earliest_commit
                        .map_or(batch.commit, |e| e.min(batch.commit)),
                );
                self.head = Some(batch.commit);
                Ok(())
            }
            Err(denial) => {
                journal.restore(self);
                Err(denial)
            }
        }
    }

    /// Mutable access to an existing entry, journaled before first change.
    fn entry_mut(
        &mut self,
        key: PersistentNameKey,
        commit: CommitId,
        journal: &mut IngestionJournal,
    ) -> Result<&mut NameEntry, PersistentNameIndexDenial> {
        journal.save_name(key, commit, &self.names);
        self.names
            .get_mut_with(&key, NameEntry::duplicate_for_branch_fork)?
            .ok_or(PersistentNameIndexDenial::InvalidTransition {
                name: key,
                detail: "a mutated name has no entry",
            })
    }

    fn insert_entry(
        &mut self,
        key: PersistentNameKey,
        commit: CommitId,
        entry: NameEntry,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        if self.names.contains_key(&key)? {
            return Err(PersistentNameIndexDenial::DuplicateBirth { name: key });
        }
        journal.save_name(key, commit, &self.names);
        self.names.insert(key, entry);
        Ok(())
    }
}

#[doc(hidden)]
pub trait PersistentNameBranchIndexFixturePort {
    /// Every entry, by a full scan of a resident index.
    fn names_for_test(&self) -> std::collections::BTreeMap<PersistentNameKey, NameEntry>;
    /// Copy-on-write fork of this index for rollback/refusal assertions in tests.
    fn fork_for_test(&self) -> PersistentNameBranchIndex;
    /// Alias of [`Self::fork_for_test`] for Phase 7B-2b test callers.
    fn duplicate_for_test(&self) -> Self;
}

impl PersistentNameBranchIndexFixturePort for PersistentNameBranchIndex {
    fn names_for_test(&self) -> std::collections::BTreeMap<PersistentNameKey, NameEntry> {
        self.names
            .collect_merged_with(NameEntry::duplicate_for_branch_fork)
            .unwrap_or_else(|cause| panic!("audit read of a damaged naming page: {cause:?}"))
            .into_iter()
            .collect()
    }

    fn fork_for_test(&self) -> PersistentNameBranchIndex {
        self.fork_to(self.branch_id.clone(), self.names.base().cloned())
    }

    fn duplicate_for_test(&self) -> Self {
        self.fork_for_test()
    }
}
