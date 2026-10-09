use crate::catalog::ArtifactFamilyInventory;
use worth_store_contracts::DurableArtifactFamilyId as Family;

#[test]
fn canonical_inventory_preserves_family_order() {
    let actual: Vec<_> = ArtifactFamilyInventory::current()
        .rows()
        .iter()
        .map(|row| row.declaration().family_id())
        .collect();
    assert_eq!(actual, EXPECTED_ORDER);
}

const EXPECTED_ORDER: &[Family] = &[
    Family::PhysicalPage,
    Family::PhysicalSegment,
    Family::PhysicalExtent,
    Family::PhysicalRootManifest,
    Family::WalDurableMutationIntent,
    Family::WalHostedRuntimeCommitResult,
    Family::WalBulkCheckpointPublicationIntent,
    Family::WalDurablePublicationProgress,
    Family::WalRecoveryDecision,
    Family::BlobChunk,
    Family::BlobManifest,
    Family::BlobStream,
    Family::ChunkTreeRoot,
    Family::DedupeIndex,
    Family::ReachabilityEdge,
    Family::RetentionHold,
    Family::ReclaimReceipt,
    Family::PlacementAuthoritativeBranchHead,
    Family::PlacementRetainedAuthority,
    Family::PlacementStableBasis,
    Family::PlacementSnapshotFamily,
    Family::PlacementBranchDeltaFamily,
    Family::PlacementLegacyLayoutFamily,
    Family::ResidencyRecord,
    Family::CorruptionRecord,
    Family::QuarantineRecord,
    Family::RepairRecord,
    Family::ReadmissionRecord,
    Family::SecurityCustodyLookup,
    Family::ExportBundle,
    Family::ImportBundle,
    Family::CapsuleArtifact,
    Family::OfflineVerificationRecord,
    Family::MaintenanceSnapshot,
    Family::MaintenanceCompaction,
    Family::MaintenanceReclaim,
    Family::MaintenanceCapsule,
    Family::MaintenanceQueueDeclaration,
    Family::SchedulerReservationIndex,
    Family::TierPlacementManifest,
    Family::ColdRecallQueue,
    Family::RecallAmplificationIndex,
    Family::BackgroundPacingRecord,
    Family::ForegroundInterferenceRecord,
    Family::SupportSchema,
    Family::SupportLineage,
    Family::SupportCursor,
    Family::SupportEmbeddedCheckpoint,
    Family::PublicationWalIntent,
    Family::PublicationWalCanonicalResult,
    Family::PublicationWalPublicationProgress,
    Family::PublicationAuthoritativeCommitAppendUnit,
    Family::PublicationBranchHeadPublication,
    Family::PublicationAcknowledgmentEligibility,
    Family::PublicationSnapshotBasis,
    Family::PublicationSnapshotImage,
    Family::DerivedRetentionLegacyLayoutMaterialization,
    Family::DerivedRetentionLegacyScopeSliceMembership,
    Family::DerivedRetentionLegacyStructuralBlock,
    Family::DerivedRetentionLegacyChunkMembership,
    Family::LayoutCompactionUnit,
    Family::SnapshotArtifact,
    Family::BranchDeltaArtifact,
];
