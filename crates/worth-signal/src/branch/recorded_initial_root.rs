use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::AdmittedSignalBranchBasis;

/// Portable recipe for the owner-proven, untouched initial root only.
/// Deserializing this description does not admit a Signal basis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RecordedSignalInitialEmptyRoot {
    meaning: Arc<RecordedInitialEmptyMeaning>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedInitialEmptyMeaning {
    version: u16,
    root: InitialRootKind,
    schema_commitment: String,
    visibility_commitment: String,
    definition_commitment: String,
    #[serde(skip)]
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl PartialEq for RecordedInitialEmptyMeaning {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.root == other.root
            && self.schema_commitment == other.schema_commitment
            && self.visibility_commitment == other.visibility_commitment
            && self.definition_commitment == other.definition_commitment
    }
}
impl Eq for RecordedInitialEmptyMeaning {}
mod json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum InitialRootKind {
    InitialEmpty,
}

impl RecordedSignalInitialEmptyRoot {
    pub(crate) fn validate_version(&self) -> Result<(), SignalInitialEmptyRootRefusal> {
        if self.meaning.version != 1 {
            return Err(SignalInitialEmptyRootRefusal::UnsupportedVersion {
                observed: self.meaning.version,
                supported: 1,
            });
        }
        Ok(())
    }

    pub(crate) fn new(schema_commitment: String) -> Self {
        Self {
            meaning: Arc::new(RecordedInitialEmptyMeaning {
                version: 1,
                root: InitialRootKind::InitialEmpty,
                schema_commitment,
                visibility_commitment: "signal-initial-empty-visibility-v1".into(),
                definition_commitment: "signal-initial-empty-definition-v1".into(),
                _funding: None,
            }),
        }
    }
    /// Original-request ownership for decoded description data, never a basis.
    pub fn from_json_in(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bytes: &[u8],
    ) -> Result<Self, SignalInitialEmptyRootRefusal> {
        json::decode(execution, bytes)
    }
    pub(crate) fn retained_copy_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, SignalInitialEmptyRootRefusal> {
        let value = &self.meaning;
        let bytes = json::fixed_bytes()?
            .checked_add(value.schema_commitment.len())
            .and_then(|bytes| bytes.checked_add(value.visibility_commitment.len()))
            .and_then(|bytes| bytes.checked_add(value.definition_commitment.len()))
            .ok_or(SignalInitialEmptyRootRefusal::Description)?;
        let funding = json::reserve(execution, bytes)?;
        Ok(Self {
            meaning: Arc::new(RecordedInitialEmptyMeaning {
                version: value.version,
                root: value.root.clone(),
                schema_commitment: value.schema_commitment.clone(),
                visibility_commitment: value.visibility_commitment.clone(),
                definition_commitment: value.definition_commitment.clone(),
                _funding: funding,
            }),
        })
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
    Request(worth_execution::WorkCeilingDenial),
    Description,
    UnsupportedVersion { observed: u16, supported: u16 },
    OwnerUnavailable,
    InitialRootRequired,
    RecordedMeaningMismatch,
    CurrentBasisRequired,
    OwnerAdmissionRequired,
    RetentionRequired,
}
