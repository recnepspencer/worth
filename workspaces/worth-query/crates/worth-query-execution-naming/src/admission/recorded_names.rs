//! The recorded names one admission reads: a tracked reader's derived index
//! at its observation, or an installed invariant's names over its commit.
//!
//! Admission never depends on where a recorded name came from; it verifies
//! each one against the installed scheme that governs it.

use worth_relational::facade::lineage::{
    PersistentNameBirthParts, PersistentNameKey, PersistentNameObservationView,
    PersistentNameSubjectKind,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::runtime::{CustomInvariantNameReads, CustomInvariantNameView};

/// Recorded names and the installed schemes that govern them.
pub(crate) trait RecordedNames {
    /// The key recorded for `digest` under `subject`; bare bytes open nothing.
    fn recorded_name(
        &self,
        subject: PersistentNameSubjectKind,
        digest: &[u8; 32],
    ) -> Option<PersistentNameKey>;
    fn scheme_of(&self, name: PersistentNameKey) -> Option<&str>;
    fn birth_of(&self, name: PersistentNameKey) -> Option<&PersistentNameBirthParts>;
    /// The installed scheme's subject class and declared subject name.
    fn scheme_subject(&self, scheme_id: &str) -> Option<(PersistentNameSubjectKind, String)>;
}

/// A tracked reader's derived name index at its observation.
pub(crate) struct AtObservation<'a, 'index> {
    pub(crate) runtime: &'a RelationalRuntime,
    pub(crate) view: &'a PersistentNameObservationView<'index>,
}

impl RecordedNames for AtObservation<'_, '_> {
    fn recorded_name(
        &self,
        subject: PersistentNameSubjectKind,
        digest: &[u8; 32],
    ) -> Option<PersistentNameKey> {
        self.view.recorded_name(subject, digest)
    }

    fn scheme_of(&self, name: PersistentNameKey) -> Option<&str> {
        self.view.scheme_of(name)
    }

    fn birth_of(&self, name: PersistentNameKey) -> Option<&PersistentNameBirthParts> {
        self.view.birth_of(name)
    }

    fn scheme_subject(&self, scheme_id: &str) -> Option<(PersistentNameSubjectKind, String)> {
        self.runtime.persistent_name_scheme_subject(scheme_id)
    }
}

/// An installed invariant's names in one posture of its commit.
pub(crate) struct InInvariant<'a> {
    pub(crate) names: &'a CustomInvariantNameReads<'a>,
    pub(crate) view: &'a dyn CustomInvariantNameView,
}

impl RecordedNames for InInvariant<'_> {
    fn recorded_name(
        &self,
        subject: PersistentNameSubjectKind,
        digest: &[u8; 32],
    ) -> Option<PersistentNameKey> {
        self.view.recorded_name(subject, digest)
    }

    fn scheme_of(&self, name: PersistentNameKey) -> Option<&str> {
        self.view.scheme_of(name)
    }

    fn birth_of(&self, name: PersistentNameKey) -> Option<&PersistentNameBirthParts> {
        self.view.birth_of(name)
    }

    fn scheme_subject(&self, scheme_id: &str) -> Option<(PersistentNameSubjectKind, String)> {
        self.names.scheme_subject(scheme_id)
    }
}
