use worth_store::physical_runtime::RecoveryDiscoveryArtifact;
use worth_store_physical_format::{
    integrity_declarations::PhysicalIntegrityArtifactFamily as Family,
    RecordArtifactFile as Artifact, RootSelectorRole,
};
use worth_store_physical_integrity::PhysicalArtifactScope;

pub(super) fn matches_artifact(
    artifact: &RecoveryDiscoveryArtifact,
    scope: PhysicalArtifactScope,
) -> bool {
    let family = scope.artifact_family();
    let RecoveryDiscoveryArtifact::Record(artifact) = artifact else {
        return matches!(artifact, RecoveryDiscoveryArtifact::CurrentCheckpoint)
            && matches!(
                family,
                Family::CheckpointStreamHeader
                    | Family::CheckpointDirtyBasis
                    | Family::CheckpointBindingCompaction
                    | Family::CheckpointBinding
                    | Family::CheckpointFooter
            );
    };
    match *artifact {
        Artifact::BootstrapCatalog | Artifact::CatalogCandidate { .. } => {
            family == Family::BootstrapCatalog
        }
        Artifact::CurrentRootSelector
        | Artifact::RootSelectorCandidate {
            role: RootSelectorRole::Current,
            ..
        } => family == Family::CurrentRootSelector,
        Artifact::PreviousRootSelector
        | Artifact::RootSelectorCandidate {
            role: RootSelectorRole::Previous,
            ..
        } => family == Family::PreviousRootSelector,
        Artifact::RootManifest { generation } => {
            family == Family::RootManifest && scope.root_generation() == Some(generation)
        }
        Artifact::RootRoutingBlock { generation, block } => {
            family == Family::RootRoutingBlock
                && scope.root_routing_block_identity().is_some_and(|identity| {
                    identity.reference().generation() == generation
                        && identity.reference().block() == block
                })
        }
        Artifact::SegmentMembershipBlock { generation, block } => {
            family == Family::SegmentMembership
                && scope
                    .segment_membership_block_identity()
                    .is_some_and(|identity| {
                        identity.reference().generation() == generation
                            && identity.reference().block() == block
                    })
        }
        Artifact::Segment { segment, .. } => {
            family == Family::PageFrame
                && scope
                    .page_identity()
                    .is_some_and(|page| page.segment_id().get() == segment)
        }
        Artifact::ExtentManifest { extent, generation } => {
            family == Family::ExtentManifest
                && scope.extent_manifest_placement().is_some_and(|placement| {
                    placement.extent().get() == extent
                        && placement.extent_generation() == generation
                })
        }
        Artifact::Extent { extent, generation } => {
            family == Family::ExtentChunk
                && scope.extent_chunk_coordinate().is_some_and(|coordinate| {
                    coordinate.extent_cell().extent_id().get() == extent
                        && coordinate.extent_cell().generation().get() == generation
                })
        }
        Artifact::FreeSpaceManifest { generation } => {
            family == Family::FreeSpaceHeader
                && scope
                    .free_space_header_identity()
                    .is_some_and(|identity| identity.generation().get() == generation)
        }
        Artifact::FreeSpaceMembershipBlock { generation, block } => {
            family == Family::FreeSpaceMembershipBlock
                && scope
                    .free_space_membership_block_identity()
                    .is_some_and(|identity| {
                        identity.reference().generation() == generation
                            && identity.reference().block() == block
                    })
        }
        Artifact::SegmentManifest { .. } => false,
    }
}
