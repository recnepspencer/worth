use serde::{Deserialize, Serialize};

use super::AdmittedSignalBranchBasis;

/// Portable recipe for the owner-proven, untouched initial root only.
/// Deserializing this description does not admit a Signal basis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedSignalInitialEmptyRoot {
    version: u16,
    root: InitialRootKind,
    schema_commitment: String,
    visibility_commitment: String,
    definition_commitment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum InitialRootKind {
    InitialEmpty,
}

impl RecordedSignalInitialEmptyRoot {
    pub(crate) fn validate_version(&self) -> Result<(), SignalInitialEmptyRootRefusal> {
        if self.version != 1 {
            return Err(SignalInitialEmptyRootRefusal::UnsupportedVersion {
                observed: self.version,
                supported: 1,
            });
        }
        Ok(())
    }

    pub(crate) fn new(schema_commitment: String) -> Self {
        Self {
            version: 1,
            root: InitialRootKind::InitialEmpty,
            schema_commitment,
            visibility_commitment: "signal-initial-empty-visibility-v1".into(),
            definition_commitment: "signal-initial-empty-definition-v1".into(),
        }
    }
}

/// Fresh owner-issued admission retaining the actual initial Signal basis.
#[derive(Debug, Clone)]
pub struct AdmittedSignalInitialEmptyRoot {
    recorded: RecordedSignalInitialEmptyRoot,
    basis: AdmittedSignalBranchBasis,
}

impl AdmittedSignalInitialEmptyRoot {
    pub(crate) fn new(
        recorded: RecordedSignalInitialEmptyRoot,
        basis: AdmittedSignalBranchBasis,
    ) -> Self {
        Self { recorded, basis }
    }

    pub fn recorded(&self) -> &RecordedSignalInitialEmptyRoot {
        &self.recorded
    }

    pub fn basis(&self) -> &AdmittedSignalBranchBasis {
        &self.basis
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalInitialEmptyRootRefusal {
    UnsupportedVersion { observed: u16, supported: u16 },
    OwnerUnavailable,
    InitialRootRequired,
    RecordedMeaningMismatch,
    CurrentBasisRequired,
    OwnerAdmissionRequired,
    RetentionRequired,
}
