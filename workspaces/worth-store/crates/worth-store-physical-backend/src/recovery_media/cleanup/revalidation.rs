use sha2::{Digest, Sha256};

use crate::filesystem_media::{ArtifactTreeFailure, ArtifactTreeFile};

use super::super::AdmittedRecoveryFilesystemMedia;
use super::BackendRecoveryCleanupRemovalRequest;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BackendRecoveryCleanupArtifactRevalidationProgress {
    reads_attempted: u64,
    reads_completed: u64,
    bytes_read: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendRecoveryCleanupArtifactRevalidationDenial {
    CheckpointRead(ArtifactTreeFailure),
    CheckpointLengthMismatch {
        expected_bytes: u64,
        observed_bytes: u64,
    },
    CheckpointDigestMismatch {
        expected_digest: [u8; 32],
        observed_digest: [u8; 32],
    },
    Read(ArtifactTreeFailure),
    LengthMismatch {
        expected_bytes: u64,
        observed_bytes: u64,
    },
    DigestMismatch {
        expected_digest: [u8; 32],
        observed_digest: [u8; 32],
    },
}

pub(super) struct RecoveryCleanupArtifactRevalidationFailure {
    denial: BackendRecoveryCleanupArtifactRevalidationDenial,
    progress: BackendRecoveryCleanupArtifactRevalidationProgress,
}

pub(super) struct RevalidatedRecoveryCleanupArtifact {
    artifact: ArtifactTreeFile,
    progress: BackendRecoveryCleanupArtifactRevalidationProgress,
}

pub(super) fn verify(
    media: &AdmittedRecoveryFilesystemMedia,
    request: &BackendRecoveryCleanupRemovalRequest,
) -> Result<RevalidatedRecoveryCleanupArtifact, RecoveryCleanupArtifactRevalidationFailure> {
    let checkpoint_progress = verify_checkpoint(media, request)?;
    let progress = verify_artifact(
        media,
        request.artifact(),
        request.artifact_bytes(),
        request.artifact_digest(),
        checkpoint_progress,
    )?;
    Ok(RevalidatedRecoveryCleanupArtifact {
        artifact: request.artifact().clone(),
        progress,
    })
}

fn verify_checkpoint(
    media: &AdmittedRecoveryFilesystemMedia,
    request: &BackendRecoveryCleanupRemovalRequest,
) -> Result<
    BackendRecoveryCleanupArtifactRevalidationProgress,
    RecoveryCleanupArtifactRevalidationFailure,
> {
    let attempted = BackendRecoveryCleanupArtifactRevalidationProgress::default().attempted();
    let observed = media
        .parts
        .artifact_tree()
        .read_bounded(
            request.checkpoint(),
            request.checkpoint_bytes().saturating_add(1),
        )
        .map_err(|failure| {
            RecoveryCleanupArtifactRevalidationFailure::checkpoint_read(attempted, failure)
        })?;
    let progress = attempted.completed(observed.len() as u64);
    if observed.len() as u64 != request.checkpoint_bytes() {
        return Err(RecoveryCleanupArtifactRevalidationFailure {
            denial: BackendRecoveryCleanupArtifactRevalidationDenial::CheckpointLengthMismatch {
                expected_bytes: request.checkpoint_bytes(),
                observed_bytes: observed.len() as u64,
            },
            progress,
        });
    }
    let observed_digest = <[u8; 32]>::from(Sha256::digest(&observed));
    if observed_digest != request.checkpoint_digest() {
        return Err(RecoveryCleanupArtifactRevalidationFailure {
            denial: BackendRecoveryCleanupArtifactRevalidationDenial::CheckpointDigestMismatch {
                expected_digest: request.checkpoint_digest(),
                observed_digest,
            },
            progress,
        });
    }
    Ok(progress)
}

fn verify_artifact(
    media: &AdmittedRecoveryFilesystemMedia,
    artifact: &ArtifactTreeFile,
    expected_bytes: u64,
    expected_digest: [u8; 32],
    progress: BackendRecoveryCleanupArtifactRevalidationProgress,
) -> Result<
    BackendRecoveryCleanupArtifactRevalidationProgress,
    RecoveryCleanupArtifactRevalidationFailure,
> {
    let attempted = progress.attempted();
    let observed = media
        .parts
        .artifact_tree()
        .read_bounded(artifact, expected_bytes.saturating_add(1))
        .map_err(|failure| RecoveryCleanupArtifactRevalidationFailure::read(attempted, failure))?;
    let progress = attempted.completed(observed.len() as u64);
    if observed.len() as u64 != expected_bytes {
        return Err(RecoveryCleanupArtifactRevalidationFailure {
            denial: BackendRecoveryCleanupArtifactRevalidationDenial::LengthMismatch {
                expected_bytes,
                observed_bytes: observed.len() as u64,
            },
            progress,
        });
    }
    let observed_digest = <[u8; 32]>::from(Sha256::digest(&observed));
    if observed_digest != expected_digest {
        return Err(RecoveryCleanupArtifactRevalidationFailure {
            denial: BackendRecoveryCleanupArtifactRevalidationDenial::DigestMismatch {
                expected_digest,
                observed_digest,
            },
            progress,
        });
    }
    Ok(progress)
}

impl RevalidatedRecoveryCleanupArtifact {
    pub(super) const fn artifact(&self) -> &ArtifactTreeFile {
        &self.artifact
    }

    pub(super) const fn progress(&self) -> BackendRecoveryCleanupArtifactRevalidationProgress {
        self.progress
    }
}

impl BackendRecoveryCleanupArtifactRevalidationProgress {
    const fn attempted(self) -> Self {
        Self {
            reads_attempted: self.reads_attempted.saturating_add(1),
            ..self
        }
    }

    const fn completed(self, bytes_read: u64) -> Self {
        Self {
            reads_completed: self.reads_completed.saturating_add(1),
            bytes_read: self.bytes_read.saturating_add(bytes_read),
            ..self
        }
    }

    pub const fn reads_attempted(self) -> u64 {
        self.reads_attempted
    }

    pub const fn reads_completed(self) -> u64 {
        self.reads_completed
    }

    pub const fn bytes_read(self) -> u64 {
        self.bytes_read
    }
}

impl RecoveryCleanupArtifactRevalidationFailure {
    fn checkpoint_read(
        progress: BackendRecoveryCleanupArtifactRevalidationProgress,
        failure: ArtifactTreeFailure,
    ) -> Self {
        Self {
            denial: BackendRecoveryCleanupArtifactRevalidationDenial::CheckpointRead(failure),
            progress,
        }
    }

    fn read(
        progress: BackendRecoveryCleanupArtifactRevalidationProgress,
        failure: ArtifactTreeFailure,
    ) -> Self {
        Self {
            denial: BackendRecoveryCleanupArtifactRevalidationDenial::Read(failure),
            progress,
        }
    }

    pub(super) const fn denial(&self) -> BackendRecoveryCleanupArtifactRevalidationDenial {
        self.denial
    }

    pub(super) const fn progress(&self) -> BackendRecoveryCleanupArtifactRevalidationProgress {
        self.progress
    }
}
