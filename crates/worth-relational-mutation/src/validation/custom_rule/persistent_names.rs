//! Read-only persistent-name reads an installed invariant makes over the
//! commit under evaluation.
//!
//! The committed posture reads the branch's names at the proposal's basis
//! only. The proposed posture overlays the names the commit itself gives its
//! records (births, split pieces, merge results, kept and restored names) and
//! the names it ends, as the commit's own naming authority validated and
//! minted them. The owner supplies both through [`CustomInvariantNameSource`];
//! nothing here mints, validates or copies a name.

use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::contracts::lineage::data::{
    LineageNameDenial, LineageRecordTarget, PersistentNameBirthParts, PersistentNameSubjectKind,
};

/// The name a record carries in one posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CustomInvariantRecordName {
    Named(PersistentNameKey),
    /// The record's name no longer designates it.
    Ended,
    Unnamed,
}

/// One posture's names, borrowed for one read.
pub trait CustomInvariantNameView {
    fn record_name(&self, record: LineageRecordTarget) -> CustomInvariantRecordName;
    /// The key recorded for `digest` under `subject`; bare bytes open nothing.
    fn recorded_name(
        &self,
        subject: PersistentNameSubjectKind,
        digest: &[u8; 32],
    ) -> Option<PersistentNameKey>;
    fn scheme_of(&self, name: PersistentNameKey) -> Option<&str>;
    fn birth_of(&self, name: PersistentNameKey) -> Option<&PersistentNameBirthParts>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CustomInvariantNamePosture {
    Committed,
    Proposed,
}

/// Why an invariant's name read refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CustomInvariantNameReadDenial {
    /// This validation supplies no persistent names.
    Unavailable,
    /// The branch's derived name index is unavailable.
    IndexUnavailable(String),
    /// The commit's naming plan is refused; the commit refuses with this.
    Naming(LineageNameDenial),
    /// The branch's names moved past the proposal's basis; the commit
    /// refuses with a retryable stale-basis conflict.
    StaleBasis,
    /// The invariant's work budget is spent.
    WorkExhausted,
}

/// The owner's names for one validation.
pub trait CustomInvariantNameSource: Sync {
    fn read(
        &self,
        posture: CustomInvariantNamePosture,
        read: &mut dyn FnMut(&dyn CustomInvariantNameView),
    ) -> Result<(), CustomInvariantNameReadDenial>;
    /// The installed scheme's subject class and declared subject name.
    fn scheme_subject(&self, scheme_id: &str) -> Option<(PersistentNameSubjectKind, String)>;
}

/// The names one invariant context reads, charged to its work meter.
#[derive(Clone)]
pub(super) struct InvariantNames<'runtime> {
    source: Option<&'runtime dyn CustomInvariantNameSource>,
    work: super::CustomInvariantWorkMeter,
}

impl<'runtime> InvariantNames<'runtime> {
    pub(super) fn new(
        source: Option<&'runtime dyn CustomInvariantNameSource>,
        work: super::CustomInvariantWorkMeter,
    ) -> Self {
        Self { source, work }
    }

    fn posture(&self, posture: CustomInvariantNamePosture) -> CustomInvariantNameReads<'_> {
        CustomInvariantNameReads {
            source: self.source,
            work: &self.work,
            posture,
        }
    }
}

/// Persistent-name reads in one posture of the commit under evaluation.
#[derive(Clone, Copy)]
pub struct CustomInvariantNameReads<'a> {
    source: Option<&'a dyn CustomInvariantNameSource>,
    work: &'a super::CustomInvariantWorkMeter,
    posture: CustomInvariantNamePosture,
}

impl CustomInvariantNameReads<'_> {
    pub const fn posture(&self) -> CustomInvariantNamePosture {
        self.posture
    }

    /// Runs `read` over this posture's names, charging one work unit.
    pub fn read<T>(
        &self,
        read: impl FnOnce(&dyn CustomInvariantNameView) -> T,
    ) -> Result<T, CustomInvariantNameReadDenial> {
        let source = self
            .source
            .ok_or(CustomInvariantNameReadDenial::Unavailable)?;
        if !self.work.try_charge(1) {
            return Err(CustomInvariantNameReadDenial::WorkExhausted);
        }
        let mut read = Some(read);
        let mut answer = None;
        source.read(self.posture, &mut |view| {
            answer = read.take().map(|read| read(view));
        })?;
        answer.ok_or(CustomInvariantNameReadDenial::Unavailable)
    }

    pub fn scheme_subject(&self, scheme_id: &str) -> Option<(PersistentNameSubjectKind, String)> {
        self.source?.scheme_subject(scheme_id)
    }
}

impl<'runtime> super::CustomInvariantScopePlanner<'runtime> {
    /// The branch's committed names at the proposal's basis.
    pub fn committed_names(&self) -> CustomInvariantNameReads<'_> {
        self.names.posture(CustomInvariantNamePosture::Committed)
    }

    /// The names the proposal leaves: committed names overlaid with the
    /// commit's own births and endings.
    pub fn proposed_names(&self) -> CustomInvariantNameReads<'_> {
        self.names.posture(CustomInvariantNamePosture::Proposed)
    }
}

impl<'runtime> super::CustomInvariantExecutionContext<'runtime> {
    /// The branch's committed names at the proposal's basis.
    pub fn committed_names(&self) -> CustomInvariantNameReads<'_> {
        self.names.posture(CustomInvariantNamePosture::Committed)
    }

    /// The names the proposal leaves: committed names overlaid with the
    /// commit's own births and endings.
    pub fn proposed_names(&self) -> CustomInvariantNameReads<'_> {
        self.names.posture(CustomInvariantNamePosture::Proposed)
    }
}

#[cfg(test)]
#[path = "persistent_names_tests.rs"]
mod tests;
