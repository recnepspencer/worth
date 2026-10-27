//! Closed public Full-read operation inventory for residency failure courts.
use super::*;
use crate::execution::RelationalExecutionDenialCause;

#[derive(Clone, Copy, Debug)]
pub(super) enum ReadDoor {
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

impl ReadDoor {
    pub(super) const ALL: [Self; 36] = [
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

    pub(super) fn is_checkpoint(self) -> bool {
        matches!(self, Self::MemoryCheckpoint | Self::EmbeddedCheckpoint)
    }
    pub(super) fn prepare(self, runtime: &crate::runtime::RelationalRuntime) {
        if matches!(self, Self::IndexCommitBuild | Self::IndexBasisBuild) {
            super::cold_index_build_door::IndexBuildDoor::register(runtime);
        }
    }
    pub(super) fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
        entity: crate::identity::data::EntityId,
    ) -> Result<(), RelationalExecutionDenialCause> {
        let version = runtime.current_version_id();
        let reader = runtime.read_truth();
        match self {
            Self::IndexCommitBuild | Self::IndexBasisBuild => {
                let door = if matches!(self, Self::IndexCommitBuild) { super::cold_index_build_door::IndexBuildDoor::Commit } else { super::cold_index_build_door::IndexBuildDoor::Basis };
                door.read(runtime, execution, runtime.index_access().definitions_snapshot().iter().find(|definition| definition.name == "residency-read-matrix-build").expect("the matrix setup registered its actual definition").index_id)
            }
            Self::MemoryCheckpoint => runtime.durability_authority().checkpoint(execution)
                .map(|image| { assert!(!image.partition_images.is_empty()); })
                .map_err(|stop| match stop {
                    RelationalRecoveryStop::Durability(error) => RelationalExecutionDenialCause::StorageUnavailable(error.storage_unavailable.expect("cold memory-image refusal remains typed")),
                    other => panic!("funded whole-image operation: {other:?}"),
                }),
            Self::EmbeddedCheckpoint => runtime.durability_authority().native_checkpoint(execution.allocation_policy())
                .map(|image| { assert!(!image.bytes().is_empty()); })
                .map_err(|stop| match stop {
                    crate::durability::data::RelationalNativeCheckpointCaptureDenial::Durability(error) => RelationalExecutionDenialCause::StorageUnavailable(error.storage_unavailable.expect("cold embedded-image refusal remains typed")),
                    other => panic!("funded whole-image operation: {other:?}"),
                }),
            Self::SnapshotRead
            | Self::SnapshotCapabilityPin
            | Self::SnapshotInspection
            | Self::SnapshotPinInspection => {
                let (_, basis) = runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap();
                let handle = runtime
                    .snapshots()
                    .snapshot_for_observation(&basis.observation())
                    .unwrap();
                let answer = match self {
                    Self::SnapshotRead => reader.read_snapshot(&handle, execution).map(|value| {
                        assert!(value.is_some(), "the owner-issued handle must resolve")
                    }),
                    Self::SnapshotCapabilityPin => reader
                        .pin_read_capability(&handle, &[], execution)
                        .map(|_| ())
                        .map_err(pin_denial),
                    Self::SnapshotInspection => {
                        reader.inspect_snapshot(execution, &handle).map(|value| {
                            assert!(value.is_some(), "the owner-issued handle must resolve")
                        })
                    }
                    Self::SnapshotPinInspection => runtime
                        .inspect_what_happened()
                        .inspect_snapshot_pinning(execution, &handle)
                        .map(|value| {
                            assert!(value.is_some(), "the owner-issued handle must resolve")
                        }),
                    _ => unreachable!("the operation selected a snapshot read"),
                };
                runtime.snapshots().release_snapshot(&handle).unwrap();
                answer
            }
            Self::ViewCapabilityPin => {
                let (_, basis) = runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap();
                let view = reader.project_observation(&basis.observation()).unwrap();
                reader
                    .pin_view_read_capability(&view, &[], execution)
                    .map(|_| ())
                    .map_err(pin_denial)
            }
            Self::GraphInspection => runtime
                .inspect_what_happened()
                .graph_summary(
                    execution,
                    &crate::inspection::data::GraphInspectionRequest {
                        scope: crate::inspection::data::InspectionScope::Current,
                        partition_scope: None,
                        relation_kind_scope: None,
                        summary_only: true,
                        budget: crate::tests::support::default_graph_budget(),
                    },
                )
                .map(|_| ()),
            Self::ConnectivityInspection => runtime
                .inspect_what_happened()
                .connectivity_summary(
                    execution,
                    &crate::inspection::data::ConnectivityInspectionRequest {
                        scope: crate::inspection::data::InspectionScope::Current,
                        partition_scope: None,
                        relation_kind_scope: None,
                        include_members: false,
                        budget: crate::tests::support::default_connectivity_budget(),
                    },
                )
                .map(|_| ()),
            Self::NeighborInspection => runtime
                .inspect_what_happened()
                .neighbors(
                    execution,
                    crate::inspection::data::InspectionScope::Current,
                    entity,
                )
                .map(|_| ()),
            Self::StructuralInspection => runtime
                .inspect_what_happened()
                .structural_identity(
                    execution,
                    crate::inspection::data::InspectionScope::Current,
                    crate::transactions::data::RecordRef::Entity(entity),
                )
                .map(|_| ()),
            Self::RetentionSummary => runtime
                .inspect_what_happened()
                .retention_summary(
                    execution,
                    &crate::tests::support::default_retention_request(),
                )
                .map(|_| ()),
            Self::RecordRetention => runtime
                .inspect_what_happened()
                .inspect_record_retention(
                    execution,
                    crate::transactions::data::RecordRef::Entity(entity),
                )
                .map(|_| ()),
            Self::RetentionInspection => runtime.retention().inspect_plan(execution).map(|_| ()),
            Self::RetentionPass => runtime.retention().run_pass(execution).map(|_| ()),
            Self::StorageStatistics => runtime
                .storage_access()
                .storage_stats(execution)
                .map(|_| ()),
            Self::PartitionStatistics => runtime
                .storage_access()
                .partition_storage_stats(execution)
                .map(|_| ()),
            Self::ChunkSummary => runtime
                .storage_access()
                .chunked_storage_summary(execution, version)
                .map(|_| ()),
            Self::ChunkDiagnostics => runtime
                .storage_access()
                .chunk_diagnostics(execution, version)
                .map(|_| ()),
            Self::Entity => reader
                .visible_entity_at_version(execution, entity, version)
                .map(|_| ()),
            Self::EntityAspects => reader
                .entity_aspects_at_version(execution, entity, version)
                .map(|_| ()),
            Self::KindScan => reader
                .visible_entities_of_kind(execution, crate::identity::data::KindId(1), version)
                .map(|_| ()),
            Self::PartitionKindScan => reader
                .visible_entities_of_kind_in_partition(
                    execution,
                    entity.partition_id,
                    crate::identity::data::KindId(1),
                    version,
                )
                .map(|_| ()),
            Self::BoundedKindScan => reader
                .bounded_visible_entities_of_kind(
                    execution,
                    crate::identity::data::KindId(1),
                    version,
                    1024,
                )
                .map(|_| ()),
            Self::RelationKindScan => reader
                .visible_relations_of_kind(execution, crate::identity::data::KindId(1), version)
                .map(|_| ()),
            Self::BoundedRelationKindScan => reader
                .bounded_visible_relations_of_kind(
                    execution,
                    crate::identity::data::KindId(1),
                    version,
                    1024,
                )
                .map(|_| ()),
            Self::Outgoing => reader
                .outgoing_relations_of_kind_at_version(
                    execution,
                    entity,
                    crate::identity::data::KindId(1),
                    version,
                )
                .map(|_| ()),
            Self::Incoming => reader
                .incoming_relations_of_kind_at_version(
                    execution,
                    entity,
                    crate::identity::data::KindId(1),
                    version,
                )
                .map(|_| ()),
            Self::BoundedOutgoing => reader
                .bounded_outgoing_relations_of_kind_at_version(
                    execution,
                    entity,
                    crate::identity::data::KindId(1),
                    version,
                    1024,
                )
                .map(|_| ()),
            Self::BoundedIncoming => reader
                .bounded_incoming_relations_of_kind_at_version(
                    execution,
                    entity,
                    crate::identity::data::KindId(1),
                    version,
                    1024,
                )
                .map(|_| ()),
            Self::OutgoingFrontier => reader
                .bounded_outgoing_relations_for_frontier_at_version(
                    execution,
                    &std::collections::BTreeSet::from([entity]),
                    crate::identity::data::KindId(1),
                    version,
                    1024,
                )
                .map(|_| ()),
            Self::IncomingFrontier => reader
                .bounded_incoming_relations_for_frontier_at_version(
                    execution,
                    &std::collections::BTreeSet::from([entity]),
                    crate::identity::data::KindId(1),
                    version,
                    1024,
                )
                .map(|_| ()),
            Self::Version => reader.try_read_version(execution, version).map(|_| ()),
            Self::Observation => {
                let transaction = test_owner_begin_transaction_for_main(runtime);
                reader
                    .read_observation(execution, &transaction.basis.observation())
                    .map(|_| ())
            }
        }
    }
}

fn pin_denial(
    denial: crate::runtime::RelationalPinnedReadAdmissionDenial,
) -> RelationalExecutionDenialCause {
    match denial {
        crate::runtime::RelationalPinnedReadAdmissionDenial::Request(cause) => cause,
        other => panic!("the producer supplied an exact owner-issued basis: {other:?}"),
    }
}
