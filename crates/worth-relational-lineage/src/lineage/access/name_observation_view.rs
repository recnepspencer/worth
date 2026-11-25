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

/// One branch's persistent names as an admitted observation sees them.
///
/// Every read is pinned to [`Self::basis`]: later commits never change an
/// answer, and nothing here can mutate the index. A branch that has
/// published no named lineage reads as an empty index: every record is
/// `Unnamed` and every held name refuses `UnknownName`.
#[derive(Clone, Copy, Debug)]
pub struct PersistentNameObservationView<'index> {
    index: Option<&'index PersistentNameBranchIndex>,
    basis: CommitId,
}

impl<'index> PersistentNameObservationView<'index> {
    pub(crate) const fn new(
        index: Option<&'index PersistentNameBranchIndex>,
        basis: CommitId,
    ) -> Self {
        Self { index, basis }
    }

    /// The observation's commit, at which every read answers.
    pub const fn basis(&self) -> CommitId {
        self.basis
    }

    /// The name `record` carries at the basis.
    ///
    /// Complexity: O(1) expected plus O(log t) in that name's timeline for a
    /// live record, O(t) for an ended one.
    pub fn name_of_record(&self, record: LineageRecordTarget) -> PersistentNameOfRecord {
        self.index.map_or(PersistentNameOfRecord::Unnamed, |index| {
            index.name_of_record(record, self.basis)
        })
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
            Some(index) => index.resolve(held, self.basis, selection),
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
        let index = self.index?;
        let key = index.recorded_name(subject, digest)?;
        let born_at = index.names.get(&key)?.born_at();
        (born_at <= self.basis).then_some(key)
    }

    /// The canonical parts `name` was born from, only when its birth is
    /// visible at the basis. A birth recorded after the basis, a group, and
    /// a name the index never recorded all answer `None`. Births are
    /// immutable: a later retire, split, merge or restore never changes one.
    ///
    /// Complexity: O(1) expected.
    pub fn birth_of(&self, name: PersistentNameKey) -> Option<&'index PersistentNameBirthParts> {
        let entry = self.index?.names.get(&name)?;
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
        let group = self.index?.group_of_member(scheme_id, member, self.basis)?;
        Some(PersistentNameKey::Group(group))
    }

    /// Every `(group scheme, group)` holding `member` at the basis.
    ///
    /// Complexity: O(1) expected plus O(S log t) over its S scheme tenures.
    pub fn groups_of_member(
        &self,
        member: &PersistentNameIdentity,
    ) -> Vec<(&'index str, PersistentNameKey)> {
        let Some(index) = self.index else {
            return Vec::new();
        };
        let groups = index.groups_of_member(member, self.basis).into_iter();
        groups
            .map(|(scheme, group)| (scheme, PersistentNameKey::Group(group)))
            .collect()
    }

    /// The member names of `group` at the basis, when it is live there.
    ///
    /// Complexity: O(log t) in the group's own timeline plus O(members).
    pub fn group_members(&self, group: PersistentNameKey) -> Option<Vec<PersistentNameIdentity>> {
        let PersistentNameKey::Group(group) = group else {
            return None;
        };
        let members = self.index?.group_members_at(group, self.basis)?;
        Some(members.iter().copied().collect())
    }

    /// The installed scheme `name` was born under, when its birth is visible
    /// at the basis.
    ///
    /// Complexity: O(1) expected.
    pub fn scheme_of(&self, name: PersistentNameKey) -> Option<&'index str> {
        let entry = self.index?.names.get(&name)?;
        (entry.born_at() <= self.basis)
            .then(|| entry.scheme_id())
            .flatten()
    }
}

#[doc(hidden)]
pub trait PersistentNameObservationViewOwnerPort<'index>: Sized {
    fn new(index: Option<&'index PersistentNameBranchIndex>, basis: CommitId) -> Self;
}

impl<'index> PersistentNameObservationViewOwnerPort<'index>
    for PersistentNameObservationView<'index>
{
    fn new(index: Option<&'index PersistentNameBranchIndex>, basis: CommitId) -> Self {
        Self::new(index, basis)
    }
}
