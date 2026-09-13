use worth_store_physical_format::{
    DurableExtentRecordPlacement, ExtentChunkCoordinate, FreeSpaceHeaderScopeIdentity,
    FreeSpaceMembershipBlockScopeIdentity, PageGenerationCell, PhysicalCheckpointIdentity,
    PhysicalRecordFormatDeclaration, PhysicalWorkObligationIdentity, RootRoutingBlockScopeIdentity,
    SegmentMembershipBlockScopeIdentity, WalSegmentIdentity,
};

use super::CheckpointStreamHeaderScopeIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PhysicalArtifactScopeIdentity {
    CurrentRootSelector(PhysicalRecordFormatDeclaration),
    PreviousRootSelector(PhysicalRecordFormatDeclaration),
    RootManifest {
        record_format: PhysicalRecordFormatDeclaration,
        generation: u64,
    },
    PhysicalWorkObligation(PhysicalWorkObligationIdentity),
    BootstrapCatalog(PhysicalRecordFormatDeclaration),
    RootRoutingBlock {
        record_format: PhysicalRecordFormatDeclaration,
        identity: RootRoutingBlockScopeIdentity,
    },
    SegmentMembershipBlock {
        record_format: PhysicalRecordFormatDeclaration,
        identity: SegmentMembershipBlockScopeIdentity,
    },
    InlinePage {
        record_format: PhysicalRecordFormatDeclaration,
        page: PageGenerationCell,
    },
    ExtentManifest {
        record_format: PhysicalRecordFormatDeclaration,
        placement: DurableExtentRecordPlacement,
    },
    ExtentChunk {
        record_format: PhysicalRecordFormatDeclaration,
        coordinate: ExtentChunkCoordinate,
    },
    WalFrame(WalSegmentIdentity),
    CheckpointStreamHeader(CheckpointStreamHeaderScopeIdentity),
    CheckpointDirtyBasis(PhysicalCheckpointIdentity),
    CheckpointBindingCompaction(PhysicalCheckpointIdentity),
    CheckpointBinding(PhysicalCheckpointIdentity),
    CheckpointFooter(PhysicalCheckpointIdentity),
    FreeSpaceHeader {
        record_format: PhysicalRecordFormatDeclaration,
        identity: FreeSpaceHeaderScopeIdentity,
    },
    FreeSpaceMembershipBlock {
        record_format: PhysicalRecordFormatDeclaration,
        identity: FreeSpaceMembershipBlockScopeIdentity,
    },
}
