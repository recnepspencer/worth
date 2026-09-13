use worth_store_wal::{WalLsnRange, WalSegmentArtifactIdentity, WalSegmentInspection};

use super::PhysicalWalSegmentCandidate;

/// One fully verified WAL artifact whose complete admitted range is covered by
/// the selected checkpoint. This is descriptive recovery truth, not cleanup
/// authority; the post-publication cleanup owner must independently prove the
/// artifact is still removable immediately before the effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointCoveredWalArtifact {
    inspection: WalSegmentInspection,
    byte_count: u64,
    cleanup_safe: bool,
}

impl CheckpointCoveredWalArtifact {
    pub(super) fn from_candidate(candidate: PhysicalWalSegmentCandidate) -> Self {
        let interruption = candidate.interrupted_tail();
        let inspection = candidate.inspection();
        Self {
            inspection,
            byte_count: interruption.map_or(inspection.byte_count(), |tail| tail.observed_bytes()),
            cleanup_safe: interruption.is_none(),
        }
    }

    pub const fn identity(&self) -> WalSegmentArtifactIdentity {
        self.inspection.identity()
    }

    pub const fn lsn_range(&self) -> WalLsnRange {
        self.inspection.lsn_range()
    }

    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    /// Complete WAL facts retained from the exact verified artifact.
    ///
    /// This remains descriptive input. The Store cleanup owner must bind it to
    /// the independently reopened root and verified checkpoint before removal.
    pub const fn inspection(&self) -> WalSegmentInspection {
        self.inspection
    }

    pub const fn cleanup_safe(&self) -> bool {
        self.cleanup_safe
    }
}
