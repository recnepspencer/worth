//! A read-only view of one branch's derived name index at one observation's
//! commit basis.

use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::lineage::data::name_resolution::{
    PersistentNameKey, PersistentNameOfRecord, PersistentNameResolutionDenial,
    PersistentNameResolutionOutcome, PersistentNameSelection,
};
use crate::contracts::lineage::data::naming::{
    PersistentNameBirthParts, PersistentNameIdentity, PersistentNameSubjectKind,
};
use crate::contracts::lineage::data::postures::LineageRecordTarget;
use crate::lineage::index::PersistentNameBranchIndex;
use crate::lineage::tiered::NamingReadScope;

/// One branch's persistent names as an admitted observation sees them.
///
/// Every read is pinned to [`Self::basis`]: later commits never change an
/// answer, and nothing here can mutate the index. A branch that has
/// published no named lineage reads as an empty index: every record is
/// `Unnamed` and every held name refuses `UnknownName`.
///
/// Rows a cold-backed index faults live in the read boundary's scope; the
/// boundary refuses the whole read when any fault failed.
#[derive(Clone, Copy, Debug)]
pub struct PersistentNameObservationView<'index> {
    index: Option<&'index PersistentNameBranchIndex>,
    scope: &'index NamingReadScope,
    basis: CommitId,
}

impl<'index> PersistentNameObservationView<'index> {
    pub(crate) const fn new(
        index: Option<&'index PersistentNameBranchIndex>,
        scope: &'index NamingReadScope,
        basis: CommitId,
    ) -> Self {
        Self {
            index,
            scope,
            basis,
        }
    }

    /// The observation's commit, at which every read answers.
    pub const fn basis(&self) -> CommitId {
        self.basis
    }

    /// Returns the index only when `self.basis` is at or above its retention
    /// floor; otherwise latches `BasisPruned` on `self.scope` so the read
    /// boundary fails closed instead of answering from compacted rows.
    fn retained_index(&self) -> Option<&'index PersistentNameBranchIndex> {
        let index = self.index?;
        if let Some(retention_floor) = index.retention_floor()
            && self.basis < retention_floor
        {
            self.scope.fail_pruned(self.basis, retention_floor);
            return None;
        }
        Some(index)
    }

    /// The name `record` carries at the basis.
    ///
    /// Complexity: O(1) expected plus O(log t) in that name's timeline.
    pub fn name_of_record(&self, record: LineageRecordTarget) -> PersistentNameOfRecord {
        self.retained_index()
            .map_or(PersistentNameOfRecord::Unnamed, |index| {
                index.name_of_record_in(record, self.basis, self.scope)
            })
    }

    /// One automatic group's incident events and retirement at this exact basis.
    pub fn aggregate_group_history(
        &self,
        scheme_id: &str,
        identity: worth_foundational::facade::AggregateIdentity,
    ) -> Option<crate::contracts::aggregate_identity::group_lineage::AggregateGroupHistory> {
        use crate::contracts::aggregate_identity::group_lineage::AggregateGroupHistory;
        use crate::lineage::index::NameState;

        let entry = self
            .retained_index()?
            .entry_in(&PersistentNameKey::Group(identity), self.scope)?;
        if entry.scheme_id() != Some(scheme_id) {
            return None;
        }
        let (at, state) = entry.state_at(self.basis)?;
        let retired = matches!(state, NameState::Ended).then_some(at);
        let events = entry
            .group_events
            .range(..=self.basis)
            .flat_map(|(&commit, events)| events.iter().copied().map(move |event| (commit, event)))
            .collect();
        Some(AggregateGroupHistory { retired, events })
    }

    /// Resolves `held` at the basis under `selection`.
    ///
    /// Complexity: see [`PersistentNameBranchIndex::resolve`]: O(lineage
    /// path), never O(names) or O(commits).
    pub fn resolve(
        &self,
        held: PersistentNameKey,
        selection: &PersistentNameSelection,
    ) -> PersistentNameResolutionOutcome {
        match self.index {
            Some(index) => index.resolve_in(held, self.basis, selection, self.scope),
            None => PersistentNameResolutionOutcome {
                result: Err(PersistentNameResolutionDenial::UnknownName { name: held }),
                metrics: Default::default(),
                footprint: Default::default(),
            },
        }
    }

    /// The key the index recorded for `digest` under `subject`, only when its
    /// birth is visible at the basis. Bare bytes with no recorded birth open
    /// nothing.
    ///
    /// Complexity: O(1) expected.
    pub fn recorded_name(
        &self,
        subject: PersistentNameSubjectKind,
        digest: &[u8; PersistentNameIdentity::BYTE_LEN],
    ) -> Option<PersistentNameKey> {
        let index = self.retained_index()?;
        let key = index.recorded_name(subject, digest, self.scope)?;
        let born_at = index.entry_in(&key, self.scope)?.born_at();
        (born_at <= self.basis).then_some(key)
    }

    /// The canonical parts `name` was born from, only when its birth is
    /// visible at the basis. A birth recorded after the basis, a group, and
    /// a name the index never recorded all answer `None`. Births are
    /// immutable: a later retire, split, merge or restore never changes one.
    ///
    /// Complexity: O(1) expected.
    pub fn birth_of(&self, name: PersistentNameKey) -> Option<&'index PersistentNameBirthParts> {
        let entry = self.retained_index()?.entry_in(&name, self.scope)?;
        (entry.born_at() <= self.basis)
            .then(|| entry.birth())
            .flatten()
    }

    /// The live group of installed group scheme `scheme_id` holding `member`
    /// at the basis.
    ///
    /// Complexity: O(1) expected plus O(log t) in the member's own tenure.
    pub fn group_of_member(
        &self,
        scheme_id: &str,
        member: &PersistentNameIdentity,
    ) -> Option<PersistentNameKey> {
        let group = self
            .retained_index()?
            .group_of_member(scheme_id, member, self.basis, self.scope)?;
        Some(PersistentNameKey::Group(group))
    }

    /// Every `(group scheme, group)` holding `member` at the basis.
    ///
    /// Complexity: O(1) expected plus O(S log t) over its S scheme tenures.
    pub fn groups_of_member(
        &self,
        member: &PersistentNameIdentity,
    ) -> Vec<(&'index str, PersistentNameKey)> {
        let Some(index) = self.retained_index() else {
            return Vec::new();
        };
        let groups = index
            .groups_of_member(member, self.basis, self.scope)
            .into_iter();
        groups
            .map(|(scheme, group)| (scheme, PersistentNameKey::Group(group)))
            .collect()
    }

    /// The member names of `group` at the basis, when it is live there.
    ///
    /// Complexity: O(log t) in the group's own timeline plus O(members).
    pub fn group_members(&self, group: PersistentNameKey) -> Option<Vec<PersistentNameIdentity>> {
        let index = self.retained_index()?;
        let PersistentNameKey::Group(group) = group else {
            return None;
        };
        let members = index.group_members_at(group, self.basis, self.scope)?;
        Some(members.iter().copied().collect())
    }

    /// The installed scheme `name` was born under, when its birth is visible
    /// at the basis.
    ///
    /// Complexity: O(1) expected.
    pub fn scheme_of(&self, name: PersistentNameKey) -> Option<&'index str> {
        let entry = self.retained_index()?.entry_in(&name, self.scope)?;
        (entry.born_at() <= self.basis)
            .then(|| entry.scheme_id())
            .flatten()
    }
}

#[doc(hidden)]
pub trait PersistentNameObservationViewOwnerPort<'index>: Sized {
    fn new(
        index: Option<&'index PersistentNameBranchIndex>,
        scope: &'index NamingReadScope,
        basis: CommitId,
    ) -> Self;
}

impl<'index> PersistentNameObservationViewOwnerPort<'index>
    for PersistentNameObservationView<'index>
{
    fn new(
        index: Option<&'index PersistentNameBranchIndex>,
        scope: &'index NamingReadScope,
        basis: CommitId,
    ) -> Self {
        Self::new(index, scope, basis)
    }
}
