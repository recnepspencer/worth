//! The root generation a recovery publication command writes, against the
//! root recovery selected through the current selector.
//!
//! Replacing the current root selector is the only publication point of a
//! root generation, and recovery selects its root through that selector. A
//! candidate-root artifact named at a later generation therefore belongs to no
//! published root: it is a killed runtime's incomplete candidate, for example
//! a graceful-close checkpoint killed before its selector replacement. That
//! runtime's candidate root legitimately differs from the one recovery
//! derives from the durable log, so recovery discards it and writes its own;
//! reopen proceeds from the published root and the durable log. Any other
//! existing artifact may be published, and different bytes there remain
//! damage. Staged page data is not decided here: it is a canonical function
//! of the redo, so it always verifies.

use worth_store_physical_backend::RecoveryStagingExistingArtifact;
use worth_store_physical_format::RecordArtifactFile;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryCandidateGeneration(CandidateGeneration);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateGeneration {
    /// The selected root's successor, which only recovery's own selector
    /// replacement publishes.
    Successor { selected: u64 },
    /// The selected root, republished exactly as recovery admitted it.
    Republication { selected: u64 },
}

impl RecoveryCandidateGeneration {
    /// The successor of the `selected` root generation.
    pub const fn successor(selected: u64) -> Option<Self> {
        match selected.checked_add(1) {
            Some(_) => Some(Self(CandidateGeneration::Successor { selected })),
            None => None,
        }
    }

    /// The `selected` root generation itself, republished.
    pub const fn republication(selected: u64) -> Self {
        Self(CandidateGeneration::Republication { selected })
    }

    /// The generation the command's candidates carry.
    pub const fn generation(self) -> u64 {
        match self.0 {
            CandidateGeneration::Successor { selected } => selected + 1,
            CandidateGeneration::Republication { selected } => selected,
        }
    }

    const fn selected(self) -> u64 {
        match self.0 {
            CandidateGeneration::Successor { selected }
            | CandidateGeneration::Republication { selected } => selected,
        }
    }

    /// What a write does with existing bytes under `artifact` that differ
    /// from its own: discard them when no published root reaches the name.
    pub(super) fn existing_artifact(
        self,
        artifact: RecordArtifactFile,
    ) -> RecoveryStagingExistingArtifact {
        match named_generation(artifact) {
            Some(generation) if generation > self.selected() => {
                RecoveryStagingExistingArtifact::DiscardUnpublished
            }
            _ => RecoveryStagingExistingArtifact::Verify,
        }
    }
}

/// The root generation that created `artifact`, when its name carries one.
/// A root reaches only artifacts created at or below its own generation.
const fn named_generation(artifact: RecordArtifactFile) -> Option<u64> {
    match artifact {
        RecordArtifactFile::RootManifest { generation }
        | RecordArtifactFile::RootRoutingBlock { generation, .. }
        | RecordArtifactFile::ReleaseCustodyHeadBlock { generation, .. }
        | RecordArtifactFile::Segment { generation, .. }
        | RecordArtifactFile::SegmentManifest { generation, .. }
        | RecordArtifactFile::SegmentMembershipBlock { generation, .. }
        | RecordArtifactFile::FreeSpaceManifest { generation }
        | RecordArtifactFile::FreeSpaceMembershipBlock { generation, .. } => Some(generation),
        RecordArtifactFile::BootstrapCatalog
        | RecordArtifactFile::CurrentRootSelector
        | RecordArtifactFile::PreviousRootSelector
        | RecordArtifactFile::RootSelectorCandidate { .. }
        | RecordArtifactFile::CatalogCandidate { .. }
        | RecordArtifactFile::ExtentArena { .. } => None,
    }
}
