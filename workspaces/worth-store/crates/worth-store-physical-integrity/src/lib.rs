//! Pure physical-integrity validation contracts.
//!
//! This crate validates bounded untrusted physical bytes against exact format
//! declarations. Its results are descriptive and sealed: they grant no media,
//! resident-frame, decoder, recovery, quarantine, or repair authority.
#![forbid(unsafe_code)]

mod artifact;
mod localization;
mod observation;
mod quarantine;
mod scrub;
mod validation;

pub use artifact::{
    project_checkpoint_binding_frame_length, validate_blob_record,
    validate_blob_record_payload_only, validate_bootstrap_catalog, validate_btree_node,
    validate_checkpoint_binding, validate_checkpoint_binding_compaction,
    validate_checkpoint_dirty_basis, validate_checkpoint_footer,
    validate_checkpoint_footer_envelope, validate_checkpoint_stream_header,
    validate_current_root_selector, validate_extent_arena_frame, validate_extent_chunk,
    validate_extent_chunk_membership, validate_extent_manifest, validate_free_space_header,
    validate_free_space_membership_block, validate_inline_page, validate_physical_work_obligation,
    validate_previous_root_selector, validate_root_manifest, validate_root_routing_block,
    validate_root_routing_block_borrowed, validate_segment_membership_block, validate_wal_frame,
    validate_wal_frame_prefix, walk_release_custody_head, walk_release_custody_head_with_port,
    BTreeNodeIntegrityValidation, BlobRecordIntegrityValidation, BlobRecordPayloadValidationDenial,
    BootstrapCatalogIntegrityValidation, BootstrapCatalogScopeMismatch,
    BootstrapCatalogUnsupportedFormat, BorrowedRootRoutingBlockIntegrityValidation,
    CheckpointBindingCompactionIntegrityValidation, CheckpointBindingFrameLengthProjection,
    CheckpointBindingIntegrityValidation, CheckpointDirtyBasisIntegrityValidation,
    CheckpointFooterEnvelopeIntegrityValidation, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, CheckpointStreamHeaderIntegrityValidation,
    CurrentRootSelectorIntegrityValidation, ExtentArenaFrameExpectation,
    ExtentArenaFrameIntegrityValidation, ExtentChunkIntegrityValidation,
    ExtentManifestIntegrityValidation, FreeSpaceHeaderIntegrityValidation,
    FreeSpaceMembershipBlockIntegrityValidation, InlinePageIntegrityValidation,
    PhysicalWorkObligationIntegrityValidation, PreviousRootSelectorIntegrityValidation,
    ReleaseCustodyHeadWalkDenial, ReleaseCustodyHeadWalkLimitsV1, ReleaseCustodyHeadWalkPort,
    ReleaseCustodyHeadWalkV1, RootManifestIntegrityValidation, RootRoutingBlockIntegrityValidation,
    RootRoutingCoordinateScratchDenial, SegmentMembershipBlockIntegrityValidation,
    VerifiedCheckpointCompactionCutover, VerifiedCheckpointStream,
    VerifiedCheckpointStreamAssemblyDenial, WalFrameIntegrityValidation,
};
pub use localization::{
    PhysicalBlastRadius, PhysicalByteRange, PhysicalByteRangeDenial, PhysicalDamageCause,
    PhysicalDamageLocalization, PhysicalFormatField,
};
pub use observation::{
    PhysicalIntegrityCounterDenial, PhysicalIntegrityObservationCounters,
    PhysicalIntegrityObservationOutcome, PhysicalIntegrityRejectionClass,
};
pub use quarantine::{PhysicalQuarantineObservation, PhysicalQuarantinePosture};
pub use scrub::{
    inspect_physical_integrity_window, PhysicalIntegrityScrubCounters,
    PhysicalIntegrityScrubInspection, PhysicalIntegrityScrubValidator,
    PhysicalIntegrityScrubWindow, PhysicalIntegrityScrubWindowOutcome,
};
pub use validation::{
    CheckpointBindingPayloadProjectionDenial, CheckpointFooterRoutingProjection,
    CheckpointStreamHeaderScopeIdentity, ExtentChunkProjectionDenial,
    IndeterminatePhysicalIntegrityCause, IndeterminatePhysicalIntegrityPosture,
    InlineRecordProjectionDenial, IntegrityValidatedBTreeNode, IntegrityValidatedBlobRecord,
    IntegrityValidatedBootstrapCatalog, IntegrityValidatedCheckpointBinding,
    IntegrityValidatedCheckpointBindingCompaction,
    IntegrityValidatedCheckpointBindingPayloadProjection, IntegrityValidatedCheckpointDirtyBasis,
    IntegrityValidatedCheckpointFooter, IntegrityValidatedCheckpointFooterEnvelope,
    IntegrityValidatedCheckpointStreamHeader, IntegrityValidatedCurrentRootSelector,
    IntegrityValidatedExtentChunkFrame, IntegrityValidatedExtentChunkProjection,
    IntegrityValidatedExtentManifest, IntegrityValidatedExtentMembership,
    IntegrityValidatedFreeSpaceHeader, IntegrityValidatedFreeSpaceMembershipBlock,
    IntegrityValidatedInlineRecordProjection, IntegrityValidatedPageFrame,
    IntegrityValidatedPhysicalWorkObligation, IntegrityValidatedPreviousRootSelector,
    IntegrityValidatedRootManifest, IntegrityValidatedRootRoutingBlock,
    IntegrityValidatedRootRoutingBlockView, IntegrityValidatedSegmentMembershipBlock,
    IntegrityValidatedSelectedExtentPayload, IntegrityValidatedWalFrame,
    IntegrityValidatedWalPayloadProjection, PhysicalArtifactScope, PhysicalArtifactScopeDenial,
    PhysicalIntegrityArtifactVersionAdapter, PhysicalIntegrityEnvelopeVersionAdapter,
    PhysicalIntegrityRejection, PhysicalIntegritySupportedVersion,
    PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, PhysicalIntegrityVersionAxis,
    PhysicalIntegrityVersionWindowOutcome, SelectedExtentPayloadBuilder,
    UnknownPhysicalIntegrityCause, UnknownPhysicalIntegrityPosture,
    UnsupportedPhysicalIntegrityVersion, UntrustedPhysicalArtifact, WalPayloadProjectionDenial,
};
