use sha2::{Digest, Sha256};

use crate::filesystem_media::{
    ArtifactTreeDirectory, ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeFile,
};

use super::AdmittedRecoveryFilesystemMedia;

/// One bounded checkpoint artifact observation under admitted recovery media.
/// A staging candidate observation is residue, never a selected source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedRecoveryCheckpointArtifact {
    artifact: ArtifactTreeFile,
    byte_count: u64,
    digest: [u8; 32],
}

impl AdmittedRecoveryFilesystemMedia {
    /// Passively observes the currently selected checkpoint for an exact
    /// comparison with the C9-admitted stream before cleanup admission.
    pub fn observe_current_checkpoint_for_residue(
        &self,
        maximum_bytes: u64,
    ) -> Result<ObservedRecoveryCheckpointArtifact, ArtifactTreeFailure> {
        if maximum_bytes == 0 {
            return Err(ArtifactTreeFailure::recovery_damaged());
        }
        let artifact = ArtifactTreeDirectory::families()
            .file("checkpoint.current")
            .map_err(|_| ArtifactTreeFailure::recovery_damaged())?;
        let bytes = self
            .parts
            .artifact_tree()
            .read_bounded(&artifact, maximum_bytes)?;
        Ok(ObservedRecoveryCheckpointArtifact {
            artifact,
            byte_count: bytes.len() as u64,
            digest: Sha256::digest(bytes).into(),
        })
    }

    /// Reads the exact candidate named by `selected_sequence + 1` under the
    /// already admitted recovery media owner. Absence is the only no-op case;
    /// oversized or unreadable media is a typed denial.
    pub fn observe_next_checkpoint_candidate(
        &self,
        selected_sequence: u64,
        maximum_bytes: u64,
    ) -> Result<Option<ObservedRecoveryCheckpointArtifact>, ArtifactTreeFailure> {
        let next = selected_sequence
            .checked_add(1)
            .filter(|next| *next != 0)
            .ok_or_else(ArtifactTreeFailure::recovery_damaged)?;
        if maximum_bytes == 0 {
            return Err(ArtifactTreeFailure::recovery_damaged());
        }
        let artifact = ArtifactTreeDirectory::staging()
            .file(&format!("checkpoint-{next:016x}.candidate"))
            .map_err(|_| ArtifactTreeFailure::recovery_damaged())?;
        match self
            .parts
            .artifact_tree()
            .read_bounded(&artifact, maximum_bytes)
        {
            Ok(bytes) => Ok(Some(ObservedRecoveryCheckpointArtifact {
                artifact,
                byte_count: bytes.len() as u64,
                digest: Sha256::digest(bytes).into(),
            })),
            Err(failure) if failure.kind() == ArtifactTreeFailureKind::Absent => Ok(None),
            Err(failure) => Err(failure),
        }
    }
}

impl ObservedRecoveryCheckpointArtifact {
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
