use worth_store::physical_runtime::RecoveryDiscoveryFailure;
use worth_store_physical_format::RecordArtifactFile;

use super::PhysicalRecoveryRootProtocolDenial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoverySuccessorCandidateDenial {
    Discovery {
        artifact: RecordArtifactFile,
        generation: u64,
        failure: RecoveryDiscoveryFailure,
    },
    MissingArtifact {
        artifact: RecordArtifactFile,
        generation: u64,
    },
    InvalidArtifact {
        artifact: RecordArtifactFile,
        generation: u64,
    },
    RootProtocol {
        artifact: RecordArtifactFile,
        generation: u64,
        denial: PhysicalRecoveryRootProtocolDenial,
    },
    ManifestEntryLimit {
        artifact: RecordArtifactFile,
        generation: u64,
        observed: u64,
        admitted: u64,
    },
    Conflict {
        artifact: RecordArtifactFile,
        generation: u64,
        mismatch: PhysicalRecoverySuccessorCandidateMismatch,
    },
}

impl PhysicalRecoverySuccessorCandidateDenial {
    pub(crate) const fn artifact(&self) -> RecordArtifactFile {
        match self {
            Self::Discovery { artifact, .. }
            | Self::MissingArtifact { artifact, .. }
            | Self::InvalidArtifact { artifact, .. }
            | Self::RootProtocol { artifact, .. }
            | Self::ManifestEntryLimit { artifact, .. }
            | Self::Conflict { artifact, .. } => *artifact,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoverySuccessorCandidateMismatch {
    RootGeneration { expected: u64, observed: u64 },
    RootTreeIdentity { expected: u64, observed: u64 },
    RootNodeCapacity { expected: u16, observed: u16 },
    RootRecordCount { expected: u64, observed: u64 },
    SuccessorArtifactInventory,
    SuccessorArtifactBytes,
    RootRoutingFrontier,
    SegmentMembershipFrontier,
    FreeSpaceMembershipFrontier,
    RootLastInlineRecord,
    RootLastInlineSegment,
    RecordPlacements,
    SegmentMembership,
    FreeSpaceGeneration { expected: u64, observed: u64 },
    FreeSpaceTreeIdentity { expected: u64, observed: u64 },
    FreeSpaceNodeCapacity { expected: u16, observed: u16 },
    FreeSpaceSegmentPageCapacity { expected: u32, observed: u32 },
    FreeSpaceEntryCount { expected: u64, observed: u64 },
    FreeSpaceNextSegment { expected: u64, observed: u64 },
    FreeSpaceNextPage { expected: u64, observed: u64 },
    FreeSpaceNextExtent { expected: u64, observed: u64 },
    FreeSpaceMembership,
}
