//! What cleanup expects to find on media immediately before an unlink.

use crate::filesystem_media::ArtifactTreeFile;

/// Exact physical bytes expected at one recovery artifact coordinate.
///
/// This is descriptive input, not removal authority. The cleanup execution
/// capability separately binds the complete request to one admitted media
/// owner before C.4 will perform any reread or unlink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendRecoveryArtifactExpectation {
    artifact: ArtifactTreeFile,
    byte_count: u64,
    digest: [u8; 32],
}

/// What `checkpoint.current` must still be when a removal is revalidated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendRecoveryCheckpointExpectation {
    /// The checkpoint recovery selected, byte-exact.
    Selected(BackendRecoveryArtifactExpectation),
    /// Recovery observed no checkpoint; the coordinate must still be absent.
    Absent(ArtifactTreeFile),
}

impl BackendRecoveryArtifactExpectation {
    pub fn new(artifact: ArtifactTreeFile, byte_count: u64, digest: [u8; 32]) -> Option<Self> {
        (byte_count != 0).then_some(Self {
            artifact,
            byte_count,
            digest,
        })
    }

    /// Preserves the exact bounded checkpoint-residue observation, including
    /// an interrupted zero-byte candidate. This does not grant unlink authority.
    #[cfg(feature = "recovery-runtime-owner")]
    pub fn from_checkpoint_observation(
        observation: &super::super::ObservedRecoveryCheckpointArtifact,
    ) -> Self {
        Self {
            artifact: observation.artifact().clone(),
            byte_count: observation.byte_count(),
            digest: observation.digest(),
        }
    }

    pub const fn artifact(&self) -> &ArtifactTreeFile {
        &self.artifact
    }

    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

impl BackendRecoveryCheckpointExpectation {
    pub const fn artifact(&self) -> &ArtifactTreeFile {
        match self {
            Self::Selected(expectation) => expectation.artifact(),
            Self::Absent(artifact) => artifact,
        }
    }
}
