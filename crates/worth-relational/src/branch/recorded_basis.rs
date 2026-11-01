//! Runtime-neutral descriptions exported from an actual current Native basis.
use serde::{Deserialize, Serialize};

use super::{RelationalBranchBasisDescriptor, RelationalBranchRootDescriptor};
use crate::history::data::{CommitId, RelationalCommitReceipt};
use crate::publication::patch::data::PatchStreamPosition;

mod export;
mod readmission;

/// Recorded Native meaning, never an admitted basis or recovery capability.
/// Empty has no fabricated commit, journal position, or artifact digest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedRelationalComponentReference {
    descriptor: RelationalBranchBasisDescriptor,
    target: RecordedRelationalComponentTarget,
}

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
        descriptor: RelationalBranchBasisDescriptor,
        entry: &crate::history::RelationalCommitIndexEntry,
    ) -> Self {
        Self {
            descriptor,
            target: RecordedRelationalComponentTarget::Canonical(
                RecordedRelationalCanonicalComponent {
                    receipt: entry.commit.clone(),
                    position: entry.position,
                    artifact_digest: entry.payload_digest,
                    roots: entry.root.clone(),
                },
            ),
        }
    }

    /// Descriptor in Native's canonical runtime frame. It cannot be used as
    /// a live descriptor by rewriting its runtime or allocation identity.
    pub fn descriptor(&self) -> &RelationalBranchBasisDescriptor {
        &self.descriptor
    }

    pub fn target(&self) -> &RecordedRelationalComponentTarget {
        &self.target
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
