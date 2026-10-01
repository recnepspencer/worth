#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DurableArtifactFamilyId {
    PhysicalPage,
    PhysicalSegment,
    PhysicalExtent,
    PhysicalRootManifest,
    WalDurableMutationIntent,
    WalHostedRuntimeCommitResult,
    WalBulkCheckpointPublicationIntent,
    WalDurablePublicationProgress,
    WalRecoveryDecision,
    BlobChunk,
    BlobManifest,
    BlobStream,
    ChunkTreeRoot,
    BlobCatalog,
    DedupeIndex,
    ReachabilityEdge,
    RetentionHold,
    ReclaimReceipt,
    ResidencyRecord,
    CorruptionRecord,
    QuarantineRecord,
    RepairRecord,
    ReadmissionRecord,
    SecurityCustodyLookup,
    ExportBundle,
    ImportBundle,
    CapsuleArtifact,
    OfflineVerificationRecord,
    SnapshotArtifact,
    BranchDeltaArtifact,
    CompatibilityCommitEnvelope,
    CompatibilityBranchVersionDagRecord,
    CompatibilityWalRestartRecord,
    CompatibilitySchemaLineageCursorCheckpointSupport,
    CompatibilityEmbeddedCheckpointAuthority,
    CompatibilitySnapshotRecord,
    CompatibilityDeltaRecord,
    CompatibilityLegacyLayoutBlockChunkRecord,
    CompatibilityLegacyBasisContinuationDescriptor,
    CompatibilityLegacyBulkRecord,
    CompatibilityLegacyRetentionRebuildRecord,
    CompatibilityLegacyMaintenanceRecord,
    CompatibilityLegacyTieringRecord,
    MaintenanceSnapshot,
    MaintenanceCompaction,
    MaintenanceReclaim,
    MaintenanceCapsule,
    MaintenanceQueueDeclaration,
    SchedulerReservationIndex,
    TierPlacementManifest,
    ColdRecallQueue,
    RecallAmplificationIndex,
    BackgroundPacingRecord,
    ForegroundInterferenceRecord,
    SupportSchema,
    SupportLineage,
    SupportCursor,
    SupportEmbeddedCheckpoint,
    PlacementAuthoritativeBranchHead,
    PlacementRetainedAuthority,
    PlacementStableBasis,
    PlacementSnapshotFamily,
    PlacementBranchDeltaFamily,
    PlacementLegacyLayoutFamily,
    PublicationWalIntent,
    PublicationWalCanonicalResult,
    PublicationWalPublicationProgress,
    PublicationAuthoritativeCommitAppendUnit,
    PublicationBranchHeadPublication,
    PublicationAcknowledgmentEligibility,
    PublicationSnapshotBasis,
    PublicationSnapshotImage,
    DerivedRetentionLegacyLayoutMaterialization,
    DerivedRetentionLegacyScopeSliceMembership,
    DerivedRetentionLegacyStructuralBlock,
    DerivedRetentionLegacyChunkMembership,
    LayoutCompactionUnit,
}

impl DurableArtifactFamilyId {
    pub const fn label(self) -> &'static str {
        match self {
            Self::PhysicalPage => "physical_page",
            Self::PhysicalSegment => "physical_segment",
            Self::PhysicalExtent => "physical_extent",
            Self::PhysicalRootManifest => "physical_root_manifest",
            Self::WalDurableMutationIntent => "wal_durable_mutation_intent",
            Self::WalHostedRuntimeCommitResult => "wal_hosted_runtime_commit_result",
            Self::WalBulkCheckpointPublicationIntent => "wal_bulk_checkpoint_publication_intent",
            Self::WalDurablePublicationProgress => "wal_durable_publication_progress",
            Self::WalRecoveryDecision => "wal_recovery_decision",
            Self::BlobChunk => "blob_chunk",
            Self::BlobManifest => "blob_manifest",
            Self::BlobStream => "blob_stream",
            Self::ChunkTreeRoot => "chunk_tree_root",
            Self::BlobCatalog => "blob_catalog",
            Self::DedupeIndex => "dedupe_index",
            Self::ReachabilityEdge => "reachability_edge",
            Self::RetentionHold => "retention_hold",
            Self::ReclaimReceipt => "reclaim_receipt",
            Self::ResidencyRecord => "residency_record",
            Self::CorruptionRecord => "corruption_record",
            Self::QuarantineRecord => "quarantine_record",
            Self::RepairRecord => "repair_record",
            Self::ReadmissionRecord => "readmission_record",
            Self::SecurityCustodyLookup => "security_custody_lookup",
            Self::ExportBundle => "export_bundle",
            Self::ImportBundle => "import_bundle",
            Self::CapsuleArtifact => "capsule_artifact",
            Self::OfflineVerificationRecord => "offline_verification_record",
            Self::SnapshotArtifact => "snapshot_artifact",
            Self::BranchDeltaArtifact => "branch_delta_artifact",
            Self::CompatibilityCommitEnvelope => "compatibility_commit_envelope",
            Self::CompatibilityBranchVersionDagRecord => "compatibility_branch_version_dag_record",
            Self::CompatibilityWalRestartRecord => "compatibility_wal_restart_record",
            Self::CompatibilitySchemaLineageCursorCheckpointSupport => {
                "compatibility_schema_lineage_cursor_checkpoint_support"
            }
            Self::CompatibilityEmbeddedCheckpointAuthority => {
                "compatibility_embedded_checkpoint_authority"
            }
            Self::CompatibilitySnapshotRecord => "compatibility_snapshot_record",
            Self::CompatibilityDeltaRecord => "compatibility_delta_record",
            Self::CompatibilityLegacyLayoutBlockChunkRecord => {
                "compatibility_milestone_6_layout_block_chunk_record"
            }
            Self::CompatibilityLegacyBasisContinuationDescriptor => {
                "compatibility_milestone_8_basis_continuation_descriptor"
            }
            Self::CompatibilityLegacyBulkRecord => "compatibility_milestone_9_bulk_record",
            Self::CompatibilityLegacyRetentionRebuildRecord => {
                "compatibility_milestone_10_retention_rebuild_record"
            }
            Self::CompatibilityLegacyMaintenanceRecord => {
                "compatibility_milestone_11_maintenance_record"
            }
            Self::CompatibilityLegacyTieringRecord => "compatibility_milestone_13_tiering_record",
            Self::MaintenanceSnapshot => "maintenance_snapshot",
            Self::MaintenanceCompaction => "maintenance_compaction",
            Self::MaintenanceReclaim => "maintenance_reclaim",
            Self::MaintenanceCapsule => "maintenance_capsule",
            Self::MaintenanceQueueDeclaration => "maintenance_queue_declaration",
            Self::SchedulerReservationIndex => "scheduler_reservation_index",
            Self::TierPlacementManifest => "tier_placement_manifest",
            Self::ColdRecallQueue => "cold_recall_queue",
            Self::RecallAmplificationIndex => "recall_amplification_index",
            Self::BackgroundPacingRecord => "background_pacing_record",
            Self::ForegroundInterferenceRecord => "foreground_interference_record",
            Self::SupportSchema => "support_schema",
            Self::SupportLineage => "support_lineage",
            Self::SupportCursor => "support_cursor",
            Self::SupportEmbeddedCheckpoint => "support_embedded_checkpoint",
            Self::PlacementAuthoritativeBranchHead => "placement_authoritative_branch_head",
            Self::PlacementRetainedAuthority => "placement_retained_authority",
            Self::PlacementStableBasis => "placement_stable_basis",
            Self::PlacementSnapshotFamily => "placement_snapshot_family",
            Self::PlacementBranchDeltaFamily => "placement_branch_delta_family",
            Self::PlacementLegacyLayoutFamily => "placement_milestone6_layout_family",
            Self::PublicationWalIntent => "publication_wal_intent",
            Self::PublicationWalCanonicalResult => "publication_wal_canonical_result",
            Self::PublicationWalPublicationProgress => "publication_wal_publication_progress",
            Self::PublicationAuthoritativeCommitAppendUnit => {
                "publication_authoritative_commit_append_unit"
            }
            Self::PublicationBranchHeadPublication => "publication_branch_head_publication",
            Self::PublicationAcknowledgmentEligibility => "publication_acknowledgment_eligibility",
            Self::PublicationSnapshotBasis => "publication_snapshot_basis",
            Self::PublicationSnapshotImage => "publication_snapshot_image",
            Self::DerivedRetentionLegacyLayoutMaterialization => {
                "derived_retention_milestone_6_layout_materialization"
            }
            Self::DerivedRetentionLegacyScopeSliceMembership => {
                "derived_retention_milestone_6_scope_slice_membership"
            }
            Self::DerivedRetentionLegacyStructuralBlock => {
                "derived_retention_milestone_6_structural_block"
            }
            Self::DerivedRetentionLegacyChunkMembership => {
                "derived_retention_milestone_6_chunk_membership"
            }
            Self::LayoutCompactionUnit => "layout_compaction_unit",
        }
    }
}
