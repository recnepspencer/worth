//! The closed public read roster and its declared residency scope.
use super::*;
impl ReadDoor {
    pub(in super::super) const ALL: [Self; 47] = [
        Self::ProjectionScan(
            super::super::cold_projection_scan_door::ProjectionScanDoor::EntityKind,
            false,
        ),
        Self::ProjectionScan(
            super::super::cold_projection_scan_door::ProjectionScanDoor::RelationKind,
            false,
        ),
        Self::ProjectionScan(
            super::super::cold_projection_scan_door::ProjectionScanDoor::EntityRecords,
            false,
        ),
        Self::ProjectionScan(
            super::super::cold_projection_scan_door::ProjectionScanDoor::RelationRecords,
            false,
        ),
        Self::ProjectionScan(
            super::super::cold_projection_scan_door::ProjectionScanDoor::EntityKind,
            true,
        ),
        Self::ProjectionScan(
            super::super::cold_projection_scan_door::ProjectionScanDoor::RelationKind,
            true,
        ),
        Self::ProjectionScan(
            super::super::cold_projection_scan_door::ProjectionScanDoor::EntityRecords,
            true,
        ),
        Self::ProjectionScan(
            super::super::cold_projection_scan_door::ProjectionScanDoor::RelationRecords,
            true,
        ),
        Self::StorageOutgoing,
        Self::StorageIncoming,
        Self::StorageAll,
        Self::IndexCommitBuild,
        Self::IndexBasisBuild,
        Self::MemoryCheckpoint,
        Self::EmbeddedCheckpoint,
        Self::GraphInspection,
        Self::SnapshotRead,
        Self::SnapshotCapabilityPin,
        Self::ViewCapabilityPin,
        Self::SnapshotInspection,
        Self::SnapshotPinInspection,
        Self::ConnectivityInspection,
        Self::NeighborInspection,
        Self::StructuralInspection,
        Self::RetentionSummary,
        Self::RecordRetention,
        Self::RetentionInspection,
        Self::RetentionPass,
        Self::StorageStatistics,
        Self::PartitionStatistics,
        Self::ChunkSummary,
        Self::ChunkDiagnostics,
        Self::Entity,
        Self::EntityAspects,
        Self::KindScan,
        Self::PartitionKindScan,
        Self::BoundedKindScan,
        Self::RelationKindScan,
        Self::BoundedRelationKindScan,
        Self::Outgoing,
        Self::Incoming,
        Self::BoundedOutgoing,
        Self::BoundedIncoming,
        Self::OutgoingFrontier,
        Self::IncomingFrontier,
        Self::Observation,
        Self::Version,
    ];

    pub(in super::super) fn is_current_projection_scan(self) -> bool {
        matches!(self, Self::ProjectionScan(_, false))
    }
    pub(in super::super) fn is_current_relation_scan(self) -> bool {
        matches!(
            self,
            Self::ProjectionScan(
                super::super::cold_projection_scan_door::ProjectionScanDoor::RelationKind
                    | super::super::cold_projection_scan_door::ProjectionScanDoor::RelationRecords,
                false
            )
        )
    }
    pub(in super::super) fn is_metadata_only_read(self) -> bool {
        matches!(
            self,
            Self::SnapshotCapabilityPin
                | Self::ViewCapabilityPin
                | Self::SnapshotInspection
                | Self::SnapshotPinInspection
        )
    }
    pub(in super::super) fn is_checkpoint(self) -> bool {
        matches!(self, Self::MemoryCheckpoint | Self::EmbeddedCheckpoint)
    }
}
