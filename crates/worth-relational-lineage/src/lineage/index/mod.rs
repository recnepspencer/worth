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
pub mod member_groups;
pub mod recorded_name;

use std::collections::HashMap;

pub use batch::PersistentNameCommittedBatch;
pub use denial::PersistentNameIndexDenial;
pub(crate) use entry::{Carriers, LiveName, NameEntry, NameOrigin, NameState};

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::contracts::lineage::data::naming::PersistentNameIdentity;
use worth_foundational::facade::AggregateIdentity;

use journal::IngestionJournal;

/// The record a lineage was created for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LineageHolder {
    Entity(EntityId),
    Relation(RelationId),
}

crate::lineage::clone_fences::assert_not_clone!(PersistentNameBranchIndex);

/// Derived name-to-record index for one branch, with per-name history.
///
/// Not `Clone`: a copy is O(names × history). The one copy the platform
/// makes is a branch fork's, through [`Self::duplicate_for_branch_fork`].
#[derive(Debug, PartialEq, Eq)]
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
    /// Per member name and group scheme: which group held it from each commit.
    member_groups: HashMap<PersistentNameIdentity, member_groups::MemberTenures>,
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
            member_groups: HashMap::new(),
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

    /// This index, moved and re-addressed to `target`: a rebuild hands an
    /// ancestor's index on to its descendant without copying it.
    ///
    /// Complexity: O(1).
    pub(crate) fn rebranched(mut self, target: BranchId) -> Self {
        self.branch_id = target;
        self
    }

    /// A copy of this index re-addressed to `target`, a branch forked from
    /// this one: the fork's explicit, named O(names × history) copy (Semantic
    /// Git `P0-B` replaces it with a copy-on-write overlay).
    ///
    /// Callers fork only an index whose head is at or before the fork
    /// commit, so the copy holds exactly the inherited history; later
    /// batches for `target` then pass the branch check.
    pub(crate) fn duplicate_for_branch_fork(&self, target: BranchId) -> Self {
        let names = self.names.iter();
        Self {
            branch_id: target,
            retention_floor: self.retention_floor,
            head: self.head,
            names: names
                .map(|(key, entry)| (*key, entry.duplicate_for_branch_fork()))
                .collect(),
            lineage_names: self.lineage_names.clone(),
            lineage_holders: self.lineage_holders.clone(),
            holder_lineages: self.holder_lineages.clone(),
            member_groups: self.member_groups.clone(),
        }
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

    /// Applies each declared carrier set as a delta onto the name's current
    /// carriers: the new timeline state shares every unchanged member with
    /// its predecessor, and only members that left or joined move tenures.
    ///
    /// Complexity: O(declared) comparison plus O(delta log M) allocation per
    /// changed name; never a copy of a whole carrier set or history.
    fn ingest_carriers(
        &mut self,
        batch: &PersistentNameCommittedBatch,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        let mut changed = Vec::new();
        for (key, carriers) in &batch.carriers {
            self.declare_carriers(*key, carriers.iter(), batch.commit, journal, &mut changed)?;
        }
        for assignment in batch.group_assignments() {
            let key = PersistentNameKey::Group(assignment.identity);
            let members = assignment.members.iter().map(|(name, _)| name);
            self.declare_carriers(key, members, batch.commit, journal, &mut changed)?;
        }
        // Every departure ends before any arrival starts, so a member moving
        // between two groups of one scheme ends in its new group.
        for (group, left, _) in &changed {
            self.end_member_tenures(*group, left, batch.commit, journal);
        }
        for (group, _, joined) in &changed {
            self.start_member_tenures(*group, joined, batch.commit, journal);
        }
        Ok(())
    }

    /// Applies one declared carrier set (ascending) as a delta onto `key`'s
    /// current carriers and records a group's left and joined members.
    fn declare_carriers<'m>(
        &mut self,
        key: PersistentNameKey,
        declared: impl Iterator<Item = &'m PersistentNameIdentity>,
        commit: CommitId,
        journal: &mut IngestionJournal,
        changed: &mut Vec<(AggregateIdentity, MemberDelta, MemberDelta)>,
    ) -> Result<(), PersistentNameIndexDenial> {
        let Some(live) = self.names.get(&key).and_then(NameEntry::current_live) else {
            return Err(PersistentNameIndexDenial::InvalidTransition {
                name: key,
                detail: "carriers declared for a name that is not live",
            });
        };
        let (left, joined) = carrier_delta(&live.carriers, declared);
        if left.is_empty() && joined.is_empty() {
            return Ok(());
        }
        let mut updated = live.clone();
        for member in &left {
            updated.carriers.remove(member);
        }
        for member in &joined {
            updated.carriers.insert(*member);
        }
        self.entry_mut(key, journal)
            .set_state(commit, NameState::Live(updated));
        if let PersistentNameKey::Group(group) = key {
            changed.push((group, left, joined));
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

/// Member names that left or joined a carrier set in one commit.
type MemberDelta = Vec<PersistentNameIdentity>;

/// The members of `prior` that the ascending `declared` drops, and those it
/// adds, by one merge walk over both.
///
/// Complexity: O(|prior| + |declared|) comparisons, O(delta) allocation.
fn carrier_delta<'m>(
    prior: &Carriers,
    declared: impl Iterator<Item = &'m PersistentNameIdentity>,
) -> (MemberDelta, MemberDelta) {
    use std::cmp::Ordering;
    let (mut left, mut joined) = (Vec::new(), Vec::new());
    let (mut old, mut new) = (prior.iter().peekable(), declared.peekable());
    loop {
        match (old.peek(), new.peek()) {
            (None, None) => break,
            (Some(member), None) => {
                left.push(**member);
                old.next();
            }
            (None, Some(member)) => {
                joined.push(**member);
                new.next();
            }
            (Some(was), Some(now)) => match was.cmp(now) {
                Ordering::Less => {
                    left.push(**was);
                    old.next();
                }
                Ordering::Greater => {
                    joined.push(**now);
                    new.next();
                }
                Ordering::Equal => {
                    old.next();
                    new.next();
                }
            },
        }
    }
    (left, joined)
}

#[doc(hidden)]
pub trait PersistentNameBranchIndexFixturePort {
    fn names_for_test(&self) -> &HashMap<PersistentNameKey, NameEntry>;
    /// A court's copy of the whole index, to compare against after a refusal.
    fn duplicate_for_test(&self) -> Self;
}

impl PersistentNameBranchIndexFixturePort for PersistentNameBranchIndex {
    fn names_for_test(&self) -> &HashMap<PersistentNameKey, NameEntry> {
        &self.names
    }

    fn duplicate_for_test(&self) -> Self {
        self.duplicate_for_branch_fork(self.branch_id.clone())
    }
}
