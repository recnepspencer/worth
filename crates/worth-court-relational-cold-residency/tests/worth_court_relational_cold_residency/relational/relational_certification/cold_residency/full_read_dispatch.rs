//! Dispatch the exact selected Full operation through its ordinary public owner.
use super::{full_read_inventory::FullReadDoor, pin_read_refusal::pin_denial};
use worth_relational::facade::transactions::RelationalExecutionDenialCause;
impl FullReadDoor {
    pub(super) fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        entity: worth_relational::facade::identity::EntityId,
    ) -> Result<(), RelationalExecutionDenialCause> {
        let version = super::current_read_basis::main_selected_version(runtime);
        let reader = runtime.read_truth();
        match self {
            Self::StorageOutgoing => runtime
                .storage_access()
                .outgoing_relations_for_entity(execution, entity, version)
                .map(|_| ()),
            Self::StorageIncoming => runtime
                .storage_access()
                .incoming_relations_for_entity(execution, entity, version)
                .map(|_| ()),
            Self::StorageAll => runtime
                .storage_access()
                .all_relations_for_entity(execution, entity, version)
                .map(|_| ()),
            Self::ProjectionScan(door, historical) => door.run(historical, execution, runtime),
            Self::IndexCommitBuild | Self::IndexBasisBuild => {
                let door = if matches!(self, Self::IndexCommitBuild) {
                    super::index_build_reads::IndexBuildDoor::Commit
                } else {
                    super::index_build_reads::IndexBuildDoor::Basis
                };
                door.read(
                    runtime,
                    execution,
                    runtime
                        .index_access()
                        .matching_definition(&super::index_build_reads::IndexBuildDoor::definition())
                        .expect("the matrix setup registered its actual definition")
                        .index_id,
                )
            }
            Self::MemoryCheckpoint => runtime
                .durability_authority()
                .checkpoint(execution)
                .map(|image| {
                    assert!(!image.partition_images.is_empty());
                })
                .map_err(super::checkpoint_read_refusal::memory_stop),
            Self::EmbeddedCheckpoint => runtime
                .durability_authority()
                .native_checkpoint(execution, execution.allocation_policy())
                .map(|image| {
                    assert!(!image.bytes().is_empty());
                })
                .map_err(super::checkpoint_read_refusal::native_stop),
            Self::SnapshotRead => super::full_snapshot_reads::read(
                execution,
                runtime,
                super::full_snapshot_reads::SnapshotRead::Read,
            ),
            Self::SnapshotCapabilityPin => super::full_snapshot_reads::read(
                execution,
                runtime,
                super::full_snapshot_reads::SnapshotRead::CapabilityPin,
            ),
            Self::SnapshotInspection => super::full_snapshot_reads::read(
                execution,
                runtime,
                super::full_snapshot_reads::SnapshotRead::Inspection,
            ),
            Self::SnapshotPinInspection => super::full_snapshot_reads::read(
                execution,
                runtime,
                super::full_snapshot_reads::SnapshotRead::PinInspection,
            ),
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
                    &worth_relational::facade::inspection::GraphInspectionRequest {
                        scope: worth_relational::facade::inspection::InspectionScope::Current,
                        partition_scope: None,
                        relation_kind_scope: None,
                        summary_only: true,
                        budget: super::inspection_read_limits::graph_budget(),
                    },
                )
                .map(|_| ()),
            Self::ConnectivityInspection => runtime
                .inspect_what_happened()
                .connectivity_summary(
                    execution,
                    &worth_relational::facade::inspection::ConnectivityInspectionRequest {
                        scope: worth_relational::facade::inspection::InspectionScope::Current,
                        partition_scope: None,
                        relation_kind_scope: None,
                        include_members: false,
                        budget: super::inspection_read_limits::connectivity_budget(),
                    },
                )
                .map(|_| ()),
            Self::NeighborInspection => runtime
                .inspect_what_happened()
                .neighbors(
                    execution,
                    worth_relational::facade::inspection::InspectionScope::Current,
                    entity,
                )
                .map(|_| ()),
            Self::StructuralInspection => runtime
                .inspect_what_happened()
                .structural_identity(
                    execution,
                    worth_relational::facade::inspection::InspectionScope::Current,
                    worth_relational::facade::transactions::RecordRef::Entity(entity),
                )
                .map(|_| ()),
            Self::RetentionSummary => runtime
                .inspect_what_happened()
                .retention_summary(
                    execution,
                    &super::inspection_read_limits::retention_request(),
                )
                .map(|_| ()),
            Self::RecordRetention => runtime
                .inspect_what_happened()
                .inspect_record_retention(
                    execution,
                    worth_relational::facade::transactions::RecordRef::Entity(entity),
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
                .visible_entities_of_kind(
                    execution,
                    worth_relational::facade::identity::KindId(1),
                    version,
                )
                .map(|_| ()),
            Self::PartitionKindScan => reader
                .visible_entities_of_kind_in_partition(
                    execution,
                    entity.partition_id,
                    worth_relational::facade::identity::KindId(1),
                    version,
                )
                .map(|_| ()),
            Self::BoundedKindScan => reader
                .bounded_visible_entities_of_kind(
                    execution,
                    worth_relational::facade::identity::KindId(1),
                    version,
                    1024,
                )
                .map(|_| ()),
            Self::RelationKindScan => reader
                .visible_relations_of_kind(
                    execution,
                    worth_relational::facade::identity::KindId(1),
                    version,
                )
                .map(|_| ()),
            Self::BoundedRelationKindScan => reader
                .bounded_visible_relations_of_kind(
                    execution,
                    worth_relational::facade::identity::KindId(1),
                    version,
                    1024,
                )
                .map(|_| ()),
            Self::Outgoing => reader
                .outgoing_relations_of_kind_at_version(
                    execution,
                    entity,
                    worth_relational::facade::identity::KindId(1),
                    version,
                )
                .map(|_| ()),
            Self::Incoming => reader
                .incoming_relations_of_kind_at_version(
                    execution,
                    entity,
                    worth_relational::facade::identity::KindId(1),
                    version,
                )
                .map(|_| ()),
            Self::BoundedOutgoing => reader
                .bounded_outgoing_relations_of_kind_at_version(
                    execution,
                    entity,
                    worth_relational::facade::identity::KindId(1),
                    version,
                    1024,
                )
                .map(|_| ()),
            Self::BoundedIncoming => reader
                .bounded_incoming_relations_of_kind_at_version(
                    execution,
                    entity,
                    worth_relational::facade::identity::KindId(1),
                    version,
                    1024,
                )
                .map(|_| ()),
            Self::OutgoingFrontier => reader
                .bounded_outgoing_relations_for_frontier_at_version(
                    execution,
                    &std::collections::BTreeSet::from([entity]),
                    worth_relational::facade::identity::KindId(1),
                    version,
                    1024,
                )
                .map(|_| ()),
            Self::IncomingFrontier => reader
                .bounded_incoming_relations_for_frontier_at_version(
                    execution,
                    &std::collections::BTreeSet::from([entity]),
                    worth_relational::facade::identity::KindId(1),
                    version,
                    1024,
                )
                .map(|_| ()),
            Self::Version => reader.try_read_version(execution, version).map(|_| ()),
            Self::Observation => {
                let (_, basis) = runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap();
                reader
                    .read_observation(execution, &basis.observation())
                    .map(|_| ())
            }
        }
    }
}
