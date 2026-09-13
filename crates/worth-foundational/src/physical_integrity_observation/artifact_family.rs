use serde::{Deserialize, Serialize};

/// Canonical persisted granules in the C.9 observation closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhysicalArtifactFamily {
    NamespaceIdentity,
    PhysicalWorkObligation,
    BootstrapCatalog,
    CurrentRootSelector,
    PreviousRootSelector,
    RootManifest,
    RootRoutingBlock,
    SegmentMembershipBlock,
    PageFrame,
    ExtentManifest,
    ExtentChunkFrame,
    FreeSpaceHeader,
    FreeSpaceMembershipBlock,
    WalFrame,
    CheckpointStreamHeader,
    CheckpointDirtyBasis,
    CheckpointBindingCompaction,
    CheckpointBinding,
    CheckpointFooter,
}
