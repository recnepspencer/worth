use super::families::{
    bootstrap::IntegrityAdmittedBootstrapCatalog,
    checkpoint::{
        IntegrityAdmittedCheckpointBinding, IntegrityAdmittedCheckpointBindingCompaction,
        IntegrityAdmittedCheckpointDirtyBasis, IntegrityAdmittedCheckpointFooter,
        IntegrityAdmittedCheckpointStreamHeader,
    },
    extent::{IntegrityAdmittedExtentChunkFrame, IntegrityAdmittedExtentManifest},
    free_space::{IntegrityAdmittedFreeSpaceHeader, IntegrityAdmittedFreeSpaceMembershipBlock},
    page::IntegrityAdmittedPageFrame,
    root::{
        IntegrityAdmittedCurrentRootSelector, IntegrityAdmittedPreviousRootSelector,
        IntegrityAdmittedRootManifest, IntegrityAdmittedRootRoutingBlock,
    },
    segment_membership::IntegrityAdmittedSegmentMembershipBlock,
    wal::IntegrityAdmittedWalFrame,
};

pub(crate) enum IntegrityAdmittedRecoveryArtifact<'media> {
    BootstrapCatalog(IntegrityAdmittedBootstrapCatalog<'media>),
    CurrentSelector(IntegrityAdmittedCurrentRootSelector<'media>),
    PreviousSelector(IntegrityAdmittedPreviousRootSelector<'media>),
    RootManifest(IntegrityAdmittedRootManifest<'media>),
    RootRoutingBlock(IntegrityAdmittedRootRoutingBlock<'media>),
    SegmentMembershipBlock(IntegrityAdmittedSegmentMembershipBlock<'media>),
    PageFrame(IntegrityAdmittedPageFrame<'media>),
    ExtentManifest(IntegrityAdmittedExtentManifest<'media>),
    ExtentChunk(IntegrityAdmittedExtentChunkFrame<'media>),
    WalFrame(IntegrityAdmittedWalFrame<'media>),
    CheckpointStreamHeader(IntegrityAdmittedCheckpointStreamHeader<'media>),
    CheckpointDirtyBasis(IntegrityAdmittedCheckpointDirtyBasis<'media>),
    CheckpointBindingCompaction(IntegrityAdmittedCheckpointBindingCompaction<'media>),
    CheckpointBinding(IntegrityAdmittedCheckpointBinding<'media>),
    CheckpointFooter(IntegrityAdmittedCheckpointFooter<'media>),
    FreeSpaceHeader(IntegrityAdmittedFreeSpaceHeader<'media>),
    FreeSpaceMembershipBlock(IntegrityAdmittedFreeSpaceMembershipBlock<'media>),
}
