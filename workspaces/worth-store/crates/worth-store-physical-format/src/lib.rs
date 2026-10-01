//! Store physical format vocabulary.
//!
//! Construction-boundary compile-fail proofs live in [`physical_format_compile_fail`]
//! and the internal `compile_fail` module tree.
#![forbid(unsafe_code)]

pub mod access;
pub mod integrity_declarations;
pub mod physical_work_obligation;
pub mod wal_frame;

mod arena_frame;
mod backup_bundle;
mod binary_format;
mod blob_manifest;
mod blob_manifest_residue_cleanup;
mod blob_manifest_residue_cleanup_v2;
mod blob_record;
mod bootstrap;
mod btree_node;
mod canonical_basis;
mod canonical_redo;
mod checkpoint;
mod checksum;
#[cfg(feature = "certification-test-authority")]
pub use record_framing::certification_crc32c_invocations;
mod compile_fail;
mod denial;
mod derived_family_root_directory;
mod extent_record;
mod format_identity;
mod generation;
mod header;
mod in_memory_physical_format_model;
mod manifest;
mod offline_verifier;
mod page_record;
mod payload;
mod physical_artifact_read_range;
mod physical_data_frame_identity;
mod tier_epoch_activation;
pub use physical_artifact_read_range::{PhysicalArtifactReadRange, PhysicalArtifactReadTarget};
mod extent_copy;
mod placement;
mod record_framing;
mod record_identity;
mod recovery_projection;
mod reference;
mod rewrite_redo;
pub use extent_copy::{
    payload_is_extent_copy_any, PhysicalExtentCopyDenial, PhysicalExtentCopyIntent,
    PhysicalExtentCopyRecord, PhysicalExtentCopyResolution, PhysicalExtentCopyResolutionKind,
    EXTENT_COPY_DOMAIN,
};
mod root_selector;
mod security_metadata;
pub mod store_namespace;

// Lifecycle-ordered public exports (≤12 groups).
pub use access::counters::PhysicalLayoutAccessCounterSnapshot;
pub use access::grammar::{
    PhysicalLayoutAccessConstraint, PhysicalLayoutAccessFamily, PhysicalLayoutAccessPattern,
    UnsupportedPhysicalLayoutAccess,
};
pub use arena_frame::{
    ExtentArenaFrameLayout, ExtentArenaId, ExtentArenaRange, EXTENT_ARENA_MANIFEST_FRAME_BYTES,
};
pub use backup_bundle::{
    backup_canonical_artifact_closure_digest, BackupBundleArtifactCoverage,
    BackupBundleArtifactFamily, BackupBundleArtifactFormat, BackupBundleArtifactManifestRow,
    BackupBundleFormatAuthority, BackupBundleFormatDenial, BackupBundleManifest,
    BackupBundleManifestConstructionDenial, BackupBundleManifestDeclaration,
    BackupBundleManifestIdentity, BackupBundleManifestReadLimits,
    BackupBundleManifestReadObservation, BackupBundlePhysicalOwner,
    BackupBundleRecoveryCoordinates, MaterializedBackupBundle,
};
pub use binary_format::{
    AllocationClassKind, FreeSpaceMapVocabulary, PhysicalAlgorithmReviewConclusion,
    PhysicalAlgorithmReviewEvidence, PhysicalAlignmentClass, PhysicalAlignmentSite,
    PhysicalBinaryEncodingWitness, PhysicalBinaryFormatError, PhysicalByteOrder,
    PhysicalByteOrderDeclaration, PhysicalComplexityStatus, PhysicalFieldWidth,
    PhysicalFieldWidthKind, PhysicalForegroundBoundednessOutcome,
    PhysicalForegroundBoundednessReport, PhysicalFormatAuthoritySource, PhysicalFormatDeclaration,
    PhysicalFormatDeclarationBuilder, PhysicalFormatEvolutionPosture, PhysicalFormatIdentity,
    PhysicalForwardCompatibilityDeclaration, PhysicalForwardCompatibilityPolicy,
    PhysicalFragmentationPressureReport, PhysicalFreeSpaceSearchPolicy,
    PhysicalGoldenFormatHeaderFixture, PhysicalLocalityClass, PhysicalOperationComplexityContract,
    PhysicalOperationCounterRow, PhysicalOperationCounterSnapshot,
    PhysicalOperationEvidenceRequirement, PhysicalOperationKind, PhysicalPageSizeClass,
    PhysicalRecordByteOrder, PhysicalRecordFormatDeclaration,
    PhysicalRecordFormatDeclarationBuilder, PhysicalRecordFormatDenial,
    PhysicalRecordFormatVersion, PhysicalRecordIntegrity, PhysicalRecordRootProtocol,
    PhysicalReservedFieldPolicy, PhysicalReservedFieldPolicyDeclaration,
};
pub use blob_manifest::{
    BlobPhysicalManifestDenial, BlobPhysicalManifestDenialKind, BlobPhysicalManifestRow,
    BlobPhysicalManifestRowKind, BlobPhysicalManifestValidation,
};
pub use blob_manifest_residue_cleanup::{
    payload_is_blob_manifest_residue_cleanup, BlobManifestResidueCleanupDenial,
    BlobManifestResidueCleanupPhaseV1, BlobManifestResidueCleanupV1,
    BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN,
};
pub use blob_manifest_residue_cleanup_v2::{
    payload_is_blob_manifest_residue_cleanup_any, payload_is_blob_manifest_residue_cleanup_v2,
    BlobManifestResidueCleanup, BlobManifestResidueCleanupV2, OriginalDropProofV1,
    ReservedDropRecordV1, BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN,
};
pub use blob_record::{
    decode_blob_record, BlobAbandonmentReasonV1, BlobChunkFrameV1, BlobChunkOccurrenceV1,
    BlobChunkReuseClaimV1, BlobChunkReuseClaimV2, BlobDedupeQuarantineV1,
    BlobGenerationPublicationV1, BlobReclaimDescriptorV1, BlobReclaimDescriptorV2,
    BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1, BlobReclaimSourceKind, BlobRecordDenial,
    BlobRecordKind, BlobRecordV1, BlobSessionAbandonedV1, BlobSessionDeclarationV1,
    BlobSessionFrontierV1, BlobTreeEntryV1, BlobTreeNodeKind, BlobTreeNodeV1, BlobTreeOccurrenceV1,
    DecodedBlobChunkFrameV1, DropSetManifestV1, DropSetManifestV2, DropSetManifestV3,
    DropSetManifestV3View, FailedIngestReclaimBasisV1, OriginalDropReservationRequestV1,
    OriginalDropReservedV1, ReleasedDropCustodyV1, ReleasedDropPredecessorV1,
    ReleasedGenerationReclaimBasisV1, BLOB_CHUNK_FRAME_MAX_BYTES, BLOB_CONTROL_FRAME_MAX_BYTES,
    BLOB_RECORD_HEADER_BYTES, BLOB_RECORD_VERSION, BLOB_TREE_NODE_FRAME_MAX_BYTES,
    MAXIMUM_DROP_SET_RECORDS,
};
pub use bootstrap::{
    physical_bootstrap_catalog, BootstrapCatalog, BootstrapCatalogDenial, CurrentRootCatalogEntry,
    CurrentRootCatalogGeneration, PhysicalBootstrapCatalogAuthority,
    PhysicalBootstrapCatalogDenial, PhysicalBootstrapCatalogIdentity,
    PhysicalBootstrapCatalogOpenWitness, PhysicalBootstrapCatalogWitness, BOOTSTRAP_CATALOG_BYTES,
};
pub use btree_node::{
    BTreeNodeCellV1, BTreeNodeDenial, BTreeNodeKind, BTreeNodeV1, BTREE_NODE_HEADER_BYTES,
    BTREE_NODE_VERSION,
};
pub use canonical_basis::{
    prepare_physical_page_header_canonical_basis, PhysicalPageHeaderCanonicalBasisOutcome,
};
pub use canonical_redo::{
    decode_canonical_redo_v3, CanonicalRedoExtentCoordinate, CanonicalRedoTarget,
    CanonicalRedoTargetIdentity, CanonicalRedoWireDenial, CanonicalRedoWireRecord,
    CANONICAL_REDO_V3_DOMAIN,
};
pub use checkpoint::{
    checkpoint_certificate_frame_bytes, checkpoint_stream_encoded_digest,
    decode_checkpoint_backup_artifact_from_reader, decode_checkpoint_binding_record,
    decode_checkpoint_certificate, encode_checkpoint_certificate,
    release_checkpoint_batch_records_digest_v1, CheckpointBackupArtifact,
    CheckpointBackupArtifactDecodeDenial, CheckpointBackupArtifactDecodeObservation,
    CheckpointBackupArtifactDecodeRequest, CheckpointBackupArtifactInput,
    CheckpointBindingCompactionEncoder, CheckpointBindingCompactionHeader,
    CheckpointBindingRecordFrameLength, CheckpointCertificateKind, CheckpointDirtyFrameBasis,
    CheckpointRootBasis, CheckpointSelectiveRecordAggregate, CheckpointSelectiveRecordSummary,
    CheckpointStreamDecodeDenial, CheckpointStreamEncoder, CheckpointStreamFooter,
    CheckpointWalSourceRange, DecodedCheckpointBackupArtifact, PersistedCompactionProductRole,
    PhysicalCheckpointIdentity, PhysicalCheckpointSecurityBinding, PhysicalCheckpointSource,
    ReleaseCheckpointAccumulatorV1, ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1,
    ReleaseCheckpointCertificateDenial, ReleaseCheckpointCertificateV1,
    ReleaseCheckpointNoReleaseV1, ReleasedDropCumulativeEvidenceV1, ReleasedDropTipProvenanceV1,
    ReleasedDropWalFateWitnessV1, TierEpochCheckpointCertificateDenial,
    TierEpochCheckpointCertificateV1, TierEpochWalFrameWitnessV1,
    CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES,
    CHECKPOINT_CERTIFICATE_PREFIX_BYTES, CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES,
    CHECKPOINT_CERTIFIED_SCHEMA, CHECKPOINT_DIRTY_FRAME_RECORD_BYTES,
    CHECKPOINT_STREAM_FOOTER_RECORD_BYTES, CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
    MAX_CHECKPOINT_BINDING_RECORD_BYTES, MAX_CHECKPOINT_CERTIFICATE_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_RECORDS, PHYSICAL_MUTATION_BINDING_COMPACTION_RECORD_DOMAIN,
    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES, RELEASE_CHECKPOINT_ACCUMULATOR_WIRE_BYTES,
    RELEASE_CHECKPOINT_BATCH_WIRE_BYTES, RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES,
    RELEASE_CHECKPOINT_TIP_WIRE_BYTES, TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES,
};
pub use checksum::{
    physical_format_required_covered_header_fields, ChecksumCompatibilityFieldPosture,
    ChecksumCoverageAuthoritySource, ChecksumCoverageDisposition, ChecksumCoverageEncoding,
    ChecksumCoverageMap, ChecksumCoverageMapBuilder, ChecksumCoverageMapDenial,
    ChecksumCoverageRegion, ChecksumFieldHandling, ChecksumGenerationFieldPosture,
    ChecksumHeaderField, ChecksumLengthFieldPosture, ChecksumPaddingPosture, ChecksumPayloadRegion,
    ChecksumReservedFieldPosture, ChecksumUnknownFieldPosture, PhysicalChunkChecksum,
    PhysicalChunkChecksumAlgorithm, PhysicalChunkChecksumAuthority, PhysicalChunkChecksumDenial,
    PhysicalChunkChecksumWitness, PhysicalChunkPayloadIntegrityWitness,
    StorePhysicalChunkWriteReceipt, StorePhysicalChunkWriteSource,
};
pub use denial::{
    PhysicalShortcutBoundary, PhysicalShortcutBoundaryDenial, PhysicalVocabularyError,
};
pub use derived_family_root_directory::{
    DerivedFamilyDirectoryDenial, DerivedFamilyRootDirectoryV1, DerivedFamilyRootEntry,
    MAX_DERIVED_FAMILY_ROOTS,
};
pub use extent_record::{
    decode_extent_chunk, encode_extent_chunk, prepare_extent_chunk, prepare_extent_chunk_reusing,
    ExtentBackedRecordPlacement, ExtentBackedRecordView, ExtentChunkCoordinate, ExtentFrameDenial,
    ExtentMembership, ExtentRecordAppendReport, ExtentRecordAppendRequest,
    ExtentRecordCounterSnapshot, ExtentRecordDenial, ExtentRecordDenialKind,
    ExtentRecordLocateReport, PhysicalExtentRecordAuthority, DURABLE_EXTENT_FRAME_HEADER_BYTES,
    EXTENT_CHUNK_METADATA_BYTES,
};
pub use format_identity::{
    PhysicalEpoch, PhysicalExtentId, PhysicalFormatMagic, PhysicalFormatVersion,
    PhysicalFormatVocabulary, PhysicalFrameId, PhysicalGeneration, PhysicalPageId,
    PhysicalRecordSlot, PhysicalRootReference, PhysicalSegmentId, PhysicalVocabularyTerm,
};
pub use generation::{
    ExtentGenerationCell, ExtentGenerationCellBuilder, FreeSpaceReuseAddress, FreeSpaceReuseCell,
    FreeSpaceReuseCellBuilder, PageGenerationCell, PageGenerationCellBuilder,
    PhysicalCellReuseDomain, PhysicalGenerationAuthority, PhysicalGenerationAuthorityScope,
    PhysicalGenerationOwner, RecordExtentGenerationCell, RecordExtentGenerationCellBuilder,
    RootPublicationCell, RootPublicationCellBuilder, SegmentGenerationCell,
    SegmentGenerationCellBuilder, SlotGenerationCell, SlotGenerationCellBuilder,
};
pub use header::{
    PhysicalDecodedHeader, PhysicalFrameHeader, PhysicalFrameKind, PhysicalHeaderAuthority,
    PhysicalHeaderAuthorityScope, PhysicalHeaderDecodeCounterSnapshot, PhysicalHeaderDecodeDenial,
    PhysicalHeaderDecodeDenialKind, PhysicalHeaderDecodeReport, PhysicalHeaderDecodeWitness,
    PhysicalHeaderKind, PhysicalHeaderReservedField, PhysicalHeaderReservedFields,
    PhysicalPageHeader, PhysicalPageKind, PhysicalPublicationState, PHYSICAL_HEADER_LENGTH,
};
pub use in_memory_physical_format_model::{
    InMemoryPhysicalFormatModel, InMemoryPhysicalFormatModelCounterSnapshot,
    InMemoryPhysicalFormatModelDenial, InMemoryPhysicalFormatModelDenialKind,
    InMemoryPhysicalFormatModelEvidence, InMemoryPhysicalFormatModelOperation,
    InMemoryPhysicalFormatModelRequest, InMemoryPhysicalFormatModelVocabulary,
    InMemoryPhysicalFormatReplayArtifact, PhysicalStoreIdentity, PlatformPhysicalAppendReport,
    PlatformPhysicalAppendRequest, PlatformPhysicalDegradedExactScanReady,
    PlatformPhysicalDegradedExactScanReceipt, PlatformPhysicalDegradedExecutionObservation,
    PlatformPhysicalFramedRecord, PlatformPhysicalHiddenScanDenialReceipt,
    PlatformPhysicalLayoutAccessIntent, PlatformPhysicalLayoutAccessRequest,
    PlatformPhysicalModelLayoutReport, PlatformPhysicalModelOperation,
    PlatformPhysicalModelOutcome, PlatformPhysicalModelReceipt, PlatformPhysicalModelReceiptDenial,
    PlatformPhysicalModelStrategy, PlatformPhysicalOperationAdmissionDenial,
    PlatformPhysicalRecordTarget, PlatformPhysicalRootPublicationObservation,
    PlatformPhysicalRootPublicationReady, PlatformPhysicalRootPublicationReport,
    PlatformPhysicalScanReport,
};
pub use manifest::{
    arena_tier_at_epoch, maximum_current_root_entries, maximum_segment_manifest_pages,
    required_tree_level, verify_release_custody_head_controls,
    verify_release_custody_head_controls_view, verify_release_custody_head_successor,
    AllocationClassManifestEntry, BoundedFreeSpaceMembershipBlockDecodeDenial,
    BoundedRootRoutingBlockDecodeDenial, BoundedSegmentMembershipBlockDecodeDenial,
    CurrentPhysicalRecordPlacement, DerivedFamilyRootDirectoryBinding, DurableArtifactCrc32c,
    DurableExtentManifest, DurableExtentRecordPlacement, DurableFreeSpaceManifestHeader,
    DurableInlineRecordPlacement, DurablePhysicalRootManifest, DurablePhysicalRootManifestBuilder,
    DurableSegmentManifest, ExtentManifestEntry, ExtentManifestVocabulary, FreeSpaceBlockReference,
    FreeSpaceHeaderScopeIdentity, FreeSpaceKey, FreeSpaceManifestEntry,
    FreeSpaceMembershipBlockDecodeLimits, FreeSpaceMembershipBlockScopeIdentity,
    FreeSpaceRoutingDenial, IndexedThroughBlobPublication, InlinePageFreeFrontier,
    ManifestBlockReference, ManifestDiscoveryAuthority, ManifestDiscoveryCounterSnapshot,
    ManifestDiscoveryDenial, ManifestDiscoveryDenialKind, ManifestDiscoveryReport,
    ManifestVocabularyKind, MembershipManifestDenial, PhysicalCurrentReachabilitySource,
    PhysicalFreeSpaceMembershipBlock, PhysicalInventoryTranscriptBuilderV1,
    PhysicalInventoryTranscriptDenial, PhysicalInventoryTranscriptV1,
    PhysicalManifestUniverseBuilder, PhysicalReclaimRegion, PhysicalReclaimRegionDenial,
    PhysicalRootManifest, PhysicalRootManifestRebuildRow, PhysicalRootManifestRebuildSource,
    PhysicalRootManifestRebuildWitness, PhysicalRootManifestVocabulary, PhysicalRootRoutingBlock,
    PhysicalRootRoutingBlockView, PhysicalSegmentMembershipBlock, PhysicalTierClass,
    PhysicalTreeIdentity, ReclaimedByteInterpretation, RecordAllocationClass,
    RecordFreeSpaceManifestEntry, RecordFreeSpaceRegion, RecordSegmentPageManifestEntry,
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadBlockV1, ReleaseCustodyHeadBlockViewV1,
    ReleaseCustodyHeadControlIdentityV1, ReleaseCustodyHeadDenial, ReleaseCustodyHeadEntryV1,
    ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadNodeWriteV1,
    ReleaseCustodyHeadPathNodeV1, ReleaseCustodyHeadRosterDigestV1,
    ReleaseCustodyHeadTransitionLimitsV1, ReleaseCustodyHeadTransitionV1, RootManifestDenial,
    RootRoutingBlockDecodeLimits, RootRoutingBlockDenial, RootRoutingBlockPreflight,
    RootRoutingBlockScopeIdentity, RootRoutingCoordinateKey, SegmentManifestBlockReference,
    SegmentManifestEntry, SegmentManifestVocabulary, SegmentMembershipBlockDecodeLimits,
    SegmentMembershipBlockDenial, SegmentMembershipBlockScopeIdentity, SegmentPageKey,
    SegmentPageManifestEntry, SelectedRecordContentClass, SelectedRecordRouteMetadata,
};
pub use offline_verifier::{
    InMemoryModelLayoutObservation, InMemoryModelLayoutObservationSource, ManifestTraversalReport,
    MinimalManifestVerifierReport, OfflineManifestCodec, OfflinePhysicalVerifier,
    OfflineVerifierCounterSnapshot, OfflineVerifierDenial, OfflineVerifierDenialKind,
    OfflineVerifierLayoutObservation, OfflineVerifierObservationSource, PersistedExtentBytes,
    PersistedPageBytes, PersistedPhysicalLayout, PersistedPhysicalLayoutBuilder,
    PhysicalLayoutReport,
};
pub use page_record::{
    append_inline_records_owned, decode_inline_record, encode_inline_page, inspect_inline_page,
    inspect_inline_page_records, restamp_inline_page_generation, AppendedInlineRecord,
    InlinePageDenial, InlinePageGeometry, InlinePageRecordDescriptor, InlineRecordAppend,
    InlineRecordRange, PageRecordCounterSnapshot, PageRecordDenial, PageRecordDenialKind,
    PhysicalPageRecordAuthority, RecordAppendReport, RecordLocateReport, SlotAppendRequest,
    SlotDirectory, SlotDirectoryEntry, SlotDirectoryEntryState, DURABLE_INLINE_PAGE_PREFIX_BYTES,
    DURABLE_INLINE_SLOT_BYTES,
};
pub use payload::{PhysicalPayloadView, PhysicalPayloadViewAdmission};
pub use physical_data_frame_identity::{
    certified_absent_prior_image_digest, write_persisted_physical_data_frame_identity,
    PersistedPhysicalDataFrameSubject,
};
pub use physical_work_obligation::PhysicalWorkObligationIdentity;
pub use placement::{RecordArtifactFile, RecordArtifactFileName, RecordFrameCoordinate};
pub use record_framing::{
    decode_data_frame_page_lsn, durable_artifact_checksum, encode_data_frame_page_lsn,
    DurableFrameDenial, DurableFrameKind, FramedRecordPayload, FramedRecordView, PhysicalPageLsn,
    RecordPagePayload, RecordPlacementClass, RecordPlacementWitness, DURABLE_FRAME_HEADER_BYTES,
};
pub use record_identity::PersistedRecordIdentity;
pub use recovery_projection::{
    PersistedBlobSemanticRecordBinding, PersistedDerivedDirectoryRecordBinding,
    PersistedDerivedDirectoryRetirement, PersistedExtentCopyRecipe,
    PersistedInlineSegmentAllocation, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryManifest, PersistedPhysicalRecoveryOperation,
    PersistedPhysicalRecoveryPayload, PersistedPhysicalRecoveryProjection,
    PersistedPhysicalRecoveryRootState, PersistedReleaseCustodyHeadEffectV1,
    PhysicalRecoveryProjectionDecodeLimits, PhysicalRecoveryProjectionDenial,
};
pub use reference::{
    CheckpointAdjacencyPosture, CurrentRootManifestAdmission, ManifestMembershipDenial,
    ManifestMembershipProof, PhysicalFutureChunkId, PhysicalFutureChunkReference,
    PhysicalReference, PhysicalReferenceAdmissionWitness, PhysicalReferenceAuthority,
    PhysicalReferenceAuthorityScope, PhysicalReferenceDenialKind, PhysicalReferenceKind,
    PhysicalReferenceScope, PhysicalReferenceValidationCounterSnapshot,
    PhysicalReferenceValidationDenial, PhysicalReferenceValidationWitness, PhysicalScopeFamily,
    RootManifestIntegrityPosture, RootPublicationValidationWitness, StalePhysicalReference,
};
pub use rewrite_redo::{
    PhysicalExtentArenaRewrite, PhysicalRewriteRedo, PhysicalRewriteRedoDenial, REWRITE_REDO_DOMAIN,
};
pub use root_selector::{
    DurableRootSelector, RootSelectorDecodeDenial, RootSelectorIdentity, RootSelectorRole,
    ROOT_SELECTOR_BYTES,
};
pub use security_metadata::{
    AllocationClassSecurityMetadataEnvelope, ExtentSecurityMetadataEnvelope,
    FreeSpaceSecurityMetadataEnvelope, PhysicalAuthenticityIdentity,
    PhysicalSecurityMetadataDeclaration, PhysicalSecurityMetadataDeclarationKind,
    PhysicalSecurityMetadataEnvelope, PhysicalSecurityMetadataResultExclusion,
    SegmentPageSecurityMetadataEnvelope, SegmentSecurityMetadataEnvelope,
};
pub use tier_epoch_activation::{
    payload_is_tier_epoch_activation, tier_epoch_anchor, TierEpochActivationDenial,
    TierEpochActivationPhaseV1, TierEpochActivationV1, TIER_EPOCH_ACTIVATION_DOMAIN,
    TIER_EPOCH_ACTIVATION_WIRE_BYTES,
};
pub use wal_frame::WalSegmentIdentity;

#[path = "compile_fail/physical_format_compile_fail.rs"]
#[doc(hidden)]
pub mod physical_format_compile_fail;
