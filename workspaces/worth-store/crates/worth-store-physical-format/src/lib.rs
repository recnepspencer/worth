//! Store physical format vocabulary.
//!
//! Construction-boundary compile-fail proofs live in [`physical_format_compile_fail`]
//! and the internal `compile_fail` module tree.
#![forbid(unsafe_code)]

pub mod access;
pub mod integrity_declarations;
pub mod physical_work_obligation;
pub mod wal_frame;

mod backup_bundle;
mod binary_format;
mod blob_manifest;
mod bootstrap;
mod canonical_basis;
mod checkpoint;
mod checksum;
#[cfg(feature = "certification-test-authority")]
pub use record_framing::certification_crc32c_invocations;
mod compile_fail;
mod denial;
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
pub use physical_artifact_read_range::{PhysicalArtifactReadRange, PhysicalArtifactReadTarget};
mod placement;
mod record_framing;
mod record_identity;
mod recovery_projection;
mod reference;
mod root_selector;
mod security_metadata;
pub mod store_namespace;

// Lifecycle-ordered public exports (≤12 groups).
pub use access::counters::PhysicalLayoutAccessCounterSnapshot;
pub use access::grammar::{
    PhysicalLayoutAccessConstraint, PhysicalLayoutAccessFamily, PhysicalLayoutAccessPattern,
    UnsupportedPhysicalLayoutAccess,
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
pub use bootstrap::{
    physical_bootstrap_catalog, BootstrapCatalog, BootstrapCatalogDenial, CurrentRootCatalogEntry,
    CurrentRootCatalogGeneration, PhysicalBootstrapCatalogAuthority,
    PhysicalBootstrapCatalogDenial, PhysicalBootstrapCatalogIdentity,
    PhysicalBootstrapCatalogOpenWitness, PhysicalBootstrapCatalogWitness, BOOTSTRAP_CATALOG_BYTES,
};
pub use canonical_basis::{
    prepare_physical_page_header_canonical_basis, PhysicalPageHeaderCanonicalBasisOutcome,
};
pub use checkpoint::{
    checkpoint_stream_encoded_digest, decode_checkpoint_backup_artifact_from_reader,
    decode_checkpoint_binding_record, CheckpointBackupArtifact,
    CheckpointBackupArtifactDecodeDenial, CheckpointBackupArtifactDecodeObservation,
    CheckpointBackupArtifactDecodeRequest, CheckpointBackupArtifactInput,
    CheckpointBindingCompactionEncoder, CheckpointBindingCompactionHeader,
    CheckpointBindingRecordFrameLength, CheckpointDirtyFrameBasis, CheckpointRootBasis,
    CheckpointSelectiveRecordAggregate, CheckpointSelectiveRecordSummary,
    CheckpointStreamDecodeDenial, CheckpointStreamEncoder, CheckpointStreamFooter,
    CheckpointWalSourceRange, DecodedCheckpointBackupArtifact, PersistedCompactionProductRole,
    PhysicalCheckpointIdentity, PhysicalCheckpointSecurityBinding, PhysicalCheckpointSource,
    CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES,
    CHECKPOINT_DIRTY_FRAME_RECORD_BYTES, CHECKPOINT_STREAM_FOOTER_RECORD_BYTES,
    CHECKPOINT_STREAM_HEADER_RECORD_BYTES, MAX_CHECKPOINT_BINDING_RECORD_BYTES,
    PHYSICAL_MUTATION_BINDING_COMPACTION_RECORD_DOMAIN,
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
    maximum_current_root_entries, maximum_segment_manifest_pages, AllocationClassManifestEntry,
    BoundedFreeSpaceMembershipBlockDecodeDenial, BoundedRootRoutingBlockDecodeDenial,
    BoundedSegmentMembershipBlockDecodeDenial, CurrentPhysicalRecordPlacement,
    DurableArtifactCrc32c, DurableExtentManifest, DurableExtentRecordPlacement,
    DurableFreeSpaceManifestHeader, DurableInlineRecordPlacement, DurablePhysicalRootManifest,
    DurablePhysicalRootManifestBuilder, DurableSegmentManifest, ExtentManifestEntry,
    ExtentManifestVocabulary, FreeSpaceBlockReference, FreeSpaceHeaderScopeIdentity, FreeSpaceKey,
    FreeSpaceManifestEntry, FreeSpaceMembershipBlockDecodeLimits,
    FreeSpaceMembershipBlockScopeIdentity, FreeSpaceRoutingDenial, ManifestBlockReference,
    ManifestDiscoveryAuthority, ManifestDiscoveryCounterSnapshot, ManifestDiscoveryDenial,
    ManifestDiscoveryDenialKind, ManifestDiscoveryReport, ManifestVocabularyKind,
    MembershipManifestDenial, PhysicalCurrentReachabilitySource, PhysicalFreeSpaceMembershipBlock,
    PhysicalManifestUniverseBuilder, PhysicalReclaimRegion, PhysicalReclaimRegionDenial,
    PhysicalRootManifest, PhysicalRootManifestRebuildRow, PhysicalRootManifestRebuildSource,
    PhysicalRootManifestRebuildWitness, PhysicalRootManifestVocabulary, PhysicalRootRoutingBlock,
    PhysicalSegmentMembershipBlock, PhysicalTreeIdentity, ReclaimedByteInterpretation,
    RecordAllocationClass, RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry,
    RootManifestDenial, RootRoutingBlockDecodeLimits, RootRoutingBlockDenial,
    RootRoutingBlockScopeIdentity, SegmentManifestBlockReference, SegmentManifestEntry,
    SegmentManifestVocabulary, SegmentMembershipBlockDecodeLimits, SegmentMembershipBlockDenial,
    SegmentMembershipBlockScopeIdentity, SegmentPageKey, SegmentPageManifestEntry,
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
    inspect_inline_page_records, AppendedInlineRecord, InlinePageDenial, InlinePageGeometry,
    InlinePageRecordDescriptor, InlineRecordAppend, InlineRecordRange, PageRecordCounterSnapshot,
    PageRecordDenial, PageRecordDenialKind, PhysicalPageRecordAuthority, RecordAppendReport,
    RecordLocateReport, SlotAppendRequest, SlotDirectory, SlotDirectoryEntry,
    SlotDirectoryEntryState, DURABLE_INLINE_PAGE_PREFIX_BYTES, DURABLE_INLINE_SLOT_BYTES,
};
pub use payload::{PhysicalPayloadView, PhysicalPayloadViewAdmission};
pub use physical_data_frame_identity::{
    certified_absent_prior_image_digest, write_persisted_physical_data_frame_identity,
    PersistedPhysicalDataFrameSubject,
};
pub use physical_work_obligation::PhysicalWorkObligationIdentity;
pub use placement::{RecordArtifactFile, RecordFrameCoordinate};
pub use record_framing::{
    decode_data_frame_page_lsn, durable_artifact_checksum, encode_data_frame_page_lsn,
    DurableFrameDenial, DurableFrameKind, FramedRecordPayload, FramedRecordView, PhysicalPageLsn,
    RecordPagePayload, RecordPlacementClass, RecordPlacementWitness, DURABLE_FRAME_HEADER_BYTES,
};
pub use record_identity::PersistedRecordIdentity;
pub use recovery_projection::{
    PersistedInlineSegmentAllocation, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryManifest, PersistedPhysicalRecoveryProjection,
    PersistedPhysicalRecoveryRootState, PhysicalRecoveryProjectionDecodeLimits,
    PhysicalRecoveryProjectionDenial,
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
pub use wal_frame::WalSegmentIdentity;

#[path = "compile_fail/physical_format_compile_fail.rs"]
#[doc(hidden)]
pub mod physical_format_compile_fail;
