use worth_store::physical_runtime::recovery_wal::{WalLsnRange, WalSegmentArtifactIdentity};
use worth_store_recovery_physics::CheckpointCoveredWalArtifact;

/// Consuming authority for one exact post-publication WAL removal.
///
/// Only the cleanup plan owner can construct this value. It is deliberately
/// neither `Clone` nor `Copy`; execution drains it once into one Store command.
pub struct RecoveryCleanupEligibility {
    covered: CheckpointCoveredWalArtifact,
}

impl RecoveryCleanupEligibility {
    pub(super) const fn new(covered: CheckpointCoveredWalArtifact) -> Self {
        Self { covered }
    }

    pub const fn artifact(&self) -> WalSegmentArtifactIdentity {
        self.covered.identity()
    }

    pub const fn range(&self) -> WalLsnRange {
        self.covered.lsn_range()
    }

    pub const fn byte_count(&self) -> u64 {
        self.covered.byte_count()
    }

    pub const fn artifact_digest(&self) -> [u8; 32] {
        self.covered.inspection().artifact_digest()
    }
}
