mod blob_record;
mod btree_node;
pub(crate) mod checkpoint;
mod durable_frame_rejection;
pub(crate) mod extent;
mod extent_arena;
pub(crate) mod free_space;
pub(crate) mod page;
pub(crate) mod physical_work_obligation;
mod release_custody_head;
pub(crate) mod root;
mod segment_membership_block;
mod segment_membership_block_rejection;
mod wal_frame;

pub use blob_record::{
    validate_blob_record, validate_blob_record_payload_only, BlobRecordIntegrityValidation,
    BlobRecordPayloadValidationDenial,
};
pub use btree_node::{validate_btree_node, BTreeNodeIntegrityValidation};
pub use checkpoint::{
    project_checkpoint_binding_frame_length, validate_checkpoint_binding,
    validate_checkpoint_binding_compaction, validate_checkpoint_dirty_basis,
    validate_checkpoint_footer, validate_checkpoint_footer_envelope,
    validate_checkpoint_stream_header, CheckpointBindingCompactionIntegrityValidation,
    CheckpointBindingFrameLengthProjection, CheckpointBindingIntegrityValidation,
    CheckpointDirtyBasisIntegrityValidation, CheckpointFooterEnvelopeIntegrityValidation,
    CheckpointFooterIntegrityValidation, CheckpointFooterValidationBasis,
    CheckpointStreamHeaderIntegrityValidation, ValidatedCheckpointStreamAssembly,
    VerifiedCheckpointCompactionCutover, VerifiedCheckpointFacts, VerifiedCheckpointStream,
    VerifiedCheckpointStreamAssemblyDenial,
};
pub use extent::{
    validate_extent_chunk, validate_extent_chunk_membership, validate_extent_manifest,
    ExtentChunkIntegrityValidation, ExtentManifestIntegrityValidation,
};
pub use extent_arena::{
    validate_extent_arena_frame, ExtentArenaFrameExpectation, ExtentArenaFrameIntegrityValidation,
};
pub use free_space::{
    validate_free_space_header, validate_free_space_membership_block,
    FreeSpaceHeaderIntegrityValidation, FreeSpaceMembershipBlockIntegrityValidation,
};
pub use page::{validate_inline_page, InlinePageIntegrityValidation};
pub use physical_work_obligation::{
    validate_physical_work_obligation, PhysicalWorkObligationIntegrityValidation,
};
#[cfg(any(test, feature = "test-support"))]
pub use release_custody_head::release_custody_head_walk_limit_for_test;
pub use release_custody_head::{
    walk_release_custody_head, walk_release_custody_head_with_port,
    ExceededReleaseCustodyHeadWalkBound, ReleaseCustodyHeadWalkBound, ReleaseCustodyHeadWalkDenial,
    ReleaseCustodyHeadWalkLimitsV1, ReleaseCustodyHeadWalkPort, ReleaseCustodyHeadWalkV1,
};
pub use root::{
    validate_bootstrap_catalog, validate_current_root_selector, validate_previous_root_selector,
    validate_root_manifest, validate_root_routing_block, validate_root_routing_block_borrowed,
    BootstrapCatalogIntegrityValidation, BootstrapCatalogScopeMismatch,
    BootstrapCatalogUnsupportedFormat, BorrowedRootRoutingBlockIntegrityValidation,
    CurrentRootSelectorIntegrityValidation, PreviousRootSelectorIntegrityValidation,
    RootManifestIntegrityValidation, RootRoutingBlockIntegrityValidation,
    RootRoutingCoordinateScratchDenial,
};
pub use segment_membership_block::{
    validate_segment_membership_block, SegmentMembershipBlockIntegrityValidation,
};
pub use wal_frame::{validate_wal_frame, validate_wal_frame_prefix, WalFrameIntegrityValidation};
