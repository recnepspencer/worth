use worth_store_physical_backend::{
    ArtifactTreeFailure, BackendRecoveryCleanupArtifactRevalidationDenial,
    BackendRecoveryCleanupArtifactRevalidationProgress,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecoveryCleanupArtifactRevalidationProgress {
    reads_attempted: u64,
    reads_completed: u64,
    bytes_read: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryCleanupArtifactRevalidationDenial {
    CheckpointRead(ArtifactTreeFailure),
    /// Recovery selected no checkpoint, but one is now published.
    CheckpointPresent {
        observed_bytes: u64,
    },
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

impl RecoveryCleanupArtifactRevalidationProgress {
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

impl From<BackendRecoveryCleanupArtifactRevalidationProgress>
    for RecoveryCleanupArtifactRevalidationProgress
{
    fn from(progress: BackendRecoveryCleanupArtifactRevalidationProgress) -> Self {
        Self {
            reads_attempted: progress.reads_attempted(),
            reads_completed: progress.reads_completed(),
            bytes_read: progress.bytes_read(),
        }
    }
}

impl From<BackendRecoveryCleanupArtifactRevalidationDenial>
    for RecoveryCleanupArtifactRevalidationDenial
{
    fn from(denial: BackendRecoveryCleanupArtifactRevalidationDenial) -> Self {
        match denial {
            BackendRecoveryCleanupArtifactRevalidationDenial::CheckpointRead(failure) => {
                Self::CheckpointRead(failure)
            }
            BackendRecoveryCleanupArtifactRevalidationDenial::CheckpointPresent {
                observed_bytes,
            } => Self::CheckpointPresent { observed_bytes },
            BackendRecoveryCleanupArtifactRevalidationDenial::CheckpointLengthMismatch {
                expected_bytes,
                observed_bytes,
            } => Self::CheckpointLengthMismatch {
                expected_bytes,
                observed_bytes,
            },
            BackendRecoveryCleanupArtifactRevalidationDenial::CheckpointDigestMismatch {
                expected_digest,
                observed_digest,
            } => Self::CheckpointDigestMismatch {
                expected_digest,
                observed_digest,
            },
            BackendRecoveryCleanupArtifactRevalidationDenial::Read(failure) => Self::Read(failure),
            BackendRecoveryCleanupArtifactRevalidationDenial::LengthMismatch {
                expected_bytes,
                observed_bytes,
            } => Self::LengthMismatch {
                expected_bytes,
                observed_bytes,
            },
            BackendRecoveryCleanupArtifactRevalidationDenial::DigestMismatch {
                expected_digest,
                observed_digest,
            } => Self::DigestMismatch {
                expected_digest,
                observed_digest,
            },
        }
    }
}
