//! Closed inventory of original Full-read consumers.
#[derive(Clone, Copy, Debug)]
pub(super) enum FullReadDoor {
    ProjectionScan(super::full_projection_scans::ProjectionScanDoor, bool),
    StorageOutgoing,
    StorageIncoming,
    StorageAll,
    IndexCommitBuild,
    IndexBasisBuild,
    MemoryCheckpoint,
    EmbeddedCheckpoint,
    GraphInspection,
    SnapshotRead,
    SnapshotCapabilityPin,
    ViewCapabilityPin,
    SnapshotInspection,
    SnapshotPinInspection,
    ConnectivityInspection,
    NeighborInspection,
    StructuralInspection,
    RetentionSummary,
    RecordRetention,
    RetentionInspection,
    RetentionPass,
    StorageStatistics,
    PartitionStatistics,
    ChunkSummary,
    ChunkDiagnostics,
    Entity,
    EntityAspects,
    KindScan,
    PartitionKindScan,
    BoundedKindScan,
    RelationKindScan,
    BoundedRelationKindScan,
    Outgoing,
    Incoming,
    BoundedOutgoing,
    BoundedIncoming,
    OutgoingFrontier,
    IncomingFrontier,
    Observation,
    Version,
}

impl FullReadDoor {
    pub(super) const ALL: [Self; 47] = [
        Self::ProjectionScan(
            super::full_projection_scans::ProjectionScanDoor::EntityKind,
            false,
        ),
        Self::ProjectionScan(
            super::full_projection_scans::ProjectionScanDoor::RelationKind,
            false,
        ),
        Self::ProjectionScan(
            super::full_projection_scans::ProjectionScanDoor::EntityRecords,
            false,
        ),
        Self::ProjectionScan(
            super::full_projection_scans::ProjectionScanDoor::RelationRecords,
            false,
        ),
        Self::ProjectionScan(
            super::full_projection_scans::ProjectionScanDoor::EntityKind,
            true,
        ),
        Self::ProjectionScan(
            super::full_projection_scans::ProjectionScanDoor::RelationKind,
            true,
        ),
        Self::ProjectionScan(
            super::full_projection_scans::ProjectionScanDoor::EntityRecords,
            true,
        ),
        Self::ProjectionScan(
            super::full_projection_scans::ProjectionScanDoor::RelationRecords,
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

    pub(super) fn is_current_projection_scan(self) -> bool {
        matches!(self, Self::ProjectionScan(_, false))
    }
    pub(super) fn is_checkpoint(self) -> bool {
        matches!(self, Self::MemoryCheckpoint | Self::EmbeddedCheckpoint)
    }
    pub(super) fn prepare(self, runtime: &worth_relational::facade::runtime::RelationalRuntime) {
        if matches!(self, Self::IndexCommitBuild | Self::IndexBasisBuild) {
            super::index_build_reads::IndexBuildDoor::register(runtime);
        }
    }
}
