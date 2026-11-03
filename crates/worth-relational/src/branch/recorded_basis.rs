//! Runtime-neutral descriptions exported from an actual current Native basis.
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{RelationalBranchBasisDescriptor, RelationalBranchRootDescriptor};
use crate::history::data::{CommitId, RelationalCommitReceipt};
use crate::publication::patch::data::PatchStreamPosition;

mod copying;
mod memory_component;
pub use memory_component::ObservedRelationalMemoryComponent;
mod export;
mod json;
mod readmission;

/// Recorded Native meaning, never an admitted basis or recovery capability.
/// Empty has no fabricated commit, journal position, or artifact digest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RecordedRelationalComponentReference {
    meaning: Arc<RecordedComponentMeaning>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedComponentMeaning {
    descriptor: RelationalBranchBasisDescriptor,
    target: RecordedRelationalComponentTarget,
    #[serde(skip)]
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl PartialEq for RecordedComponentMeaning {
    fn eq(&self, other: &Self) -> bool {
        self.descriptor == other.descriptor && self.target == other.target
    }
}
impl Eq for RecordedComponentMeaning {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordedRelationalComponentTarget {
    Empty,
    Canonical(RecordedRelationalCanonicalComponent),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedRelationalCanonicalComponent {
    receipt: RelationalCommitReceipt,
    position: PatchStreamPosition,
    artifact_digest: [u8; 32],
    roots: RelationalBranchRootDescriptor,
}

impl RecordedRelationalComponentReference {
    pub(crate) fn from_readmitted_source(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        descriptor: &RelationalBranchBasisDescriptor,
        entry: &crate::history::RelationalCommitIndexEntry,
    ) -> Result<Self, RelationalRecordedComponentRefusal> {
        Self::copy_original(
            execution,
            descriptor,
            Some(copying::CanonicalDescription {
                receipt: &entry.commit,
                position: entry.position,
                artifact_digest: entry.payload_digest,
                roots: &entry.root,
            }),
        )
    }

    /// Decode descriptive owner data with the original request's retained
    /// payload and temporary parser backing. This admits no Native basis.
    pub fn from_json_in(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bytes: &[u8],
    ) -> Result<Self, RelationalRecordedComponentRefusal> {
        json::decode(execution, bytes)
    }

    /// Descriptor in Native's canonical runtime frame. It cannot be used as
    /// a live descriptor by rewriting its runtime or allocation identity.
    pub fn descriptor(&self) -> &RelationalBranchBasisDescriptor {
        &self.meaning.descriptor
    }

    pub fn target(&self) -> &RecordedRelationalComponentTarget {
        &self.meaning.target
    }
}

impl RecordedRelationalCanonicalComponent {
    pub fn receipt(&self) -> &RelationalCommitReceipt {
        &self.receipt
    }

    pub fn position(&self) -> PatchStreamPosition {
        self.position
    }

    pub fn artifact_digest(&self) -> &[u8; 32] {
        &self.artifact_digest
    }

    pub fn roots(&self) -> &RelationalBranchRootDescriptor {
        &self.roots
    }
}

#[derive(Debug)]
pub enum RelationalRecordedComponentRefusal {
    Request(crate::execution::RelationalExecutionDenialCause),
    Basis(super::RelationalBranchBasisDenial),
    Frame(super::RelationalBranchObservationConstructionDenial),
    Source(crate::durability::data::RelationalRecoveryStop),
    MissingPerformedCommit { commit_id: CommitId },
    CanonicalMismatch { commit_id: CommitId },
    PayloadEncoding { detail: String },
}
