//! A read-only view of one branch's derived name index at one observation's
//! commit basis.

use crate::history::data::CommitId;
use crate::lineage::data::{
    LineageRecordTarget, PersistentNameIdentity, PersistentNameKey, PersistentNameOfRecord,
    PersistentNameResolutionDenial, PersistentNameResolutionOutcome, PersistentNameSelection,
    PersistentNameSubjectKind,
};
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
    /// Complexity: O(1) expected plus O(log t) in that name's timeline.
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

    /// The installed scheme `name` was born under, when its birth is visible
    /// at the basis; groups carry none.
    ///
    /// Complexity: O(1) expected.
    pub fn scheme_of(&self, name: PersistentNameKey) -> Option<&'index str> {
        let entry = self.index?.names.get(&name)?;
        (entry.born_at() <= self.basis)
            .then(|| entry.scheme_id())
            .flatten()
    }
}
