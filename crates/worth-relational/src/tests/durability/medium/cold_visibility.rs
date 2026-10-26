//! Ordinary visibility doors must refuse cold damage before yielding evidence.
use super::*;
use crate::execution::RelationalExecutionDenialCause;

#[derive(Clone, Copy, Debug)]
pub(super) enum ReadDoor {
    GraphInspection,
    SnapshotRead,
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
    pub(super) const ALL: [Self; 30] = [
        Self::GraphInspection,
        Self::SnapshotRead,
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

    pub(super) fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
        entity: crate::identity::data::EntityId,
    ) -> Result<(), RelationalExecutionDenialCause> {
        let version = runtime.current_version_id();
        let reader = runtime.read_truth();
        match self {
            Self::SnapshotRead | Self::SnapshotInspection | Self::SnapshotPinInspection => {
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

#[test]
fn cold_visibility_doors_preserve_storage_failure_and_never_advance_publication() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in ReadDoor::ALL {
        let (source, counter, entity) = fixture(execution, false);
        let mut runtime = durable_runtime_with_test_schema();
        let plan = runtime
            .durability()
            .recovery_plan_from_source(
                execution,
                source,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        runtime
            .durability_recovery()
            .recover(execution, plan)
            .unwrap();
        let before = runtime.history().latest_patch_stream_position();
        let failure = door.run(execution, &runtime, entity).unwrap_err();
        assert!(
            matches!(failure, RelationalExecutionDenialCause::StorageUnavailable(
            crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(partition),
                cause: Cause::IntegrityMismatch,
            }) if partition == entity.partition_id),
            "{door:?}: {failure:?}"
        );
        let reads = counter.reads.load(Ordering::Relaxed);
        assert!(reads > 0);
        counter.damaged.store(false, Ordering::Relaxed);
        assert_eq!(
            door.run(execution, &runtime, entity).unwrap_err(),
            failure,
            "{door:?}"
        );
        assert_eq!(counter.reads.load(Ordering::Relaxed), reads);
        assert_eq!(runtime.history().latest_patch_stream_position(), before);
        assert!(attached_test_medium(&runtime).offered().is_empty());
    }
}

#[test]
fn cold_and_warm_visibility_preserve_an_observation_across_a_later_commit() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let reference = runtime_with_test_schema();
    let expected_entity = create_entity(execution, &reference, "member-before");
    let expected = reference
        .read_truth()
        .visible_entity_at_version(execution, expected_entity, reference.current_version_id())
        .unwrap()
        .unwrap();
    let (source, counter, entity) = fixture(execution, false);
    counter.damaged.store(false, Ordering::Relaxed);
    // Reopening a writer requires the medium and recovery source to describe
    // the same durable prefix. Publish the original artifacts, not synthetic
    // positions or a manually advanced frontier, into this test attachment.
    let attachment = TestDurableMedium::new().attachment();
    for bytes in counter.commits.values() {
        let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            attachment.publish(&artifact),
            crate::durability::medium::RelationalDurablePublication::Committed
        ));
    }
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .build_durable(attachment);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let transaction = test_owner_begin_transaction_for_main(&runtime);
    let observation = transaction.basis.observation();
    let cold = runtime
        .read_truth()
        .visible_entity_at_version(execution, entity, runtime.current_version_id())
        .unwrap()
        .unwrap();
    assert_eq!(cold, expected);
    let reads = counter.reads.load(Ordering::Relaxed);
    let warm = runtime
        .read_truth()
        .visible_entity_at_version(execution, entity, runtime.current_version_id())
        .unwrap()
        .unwrap();
    assert_eq!(warm, expected);
    assert_eq!(counter.reads.load(Ordering::Relaxed), reads);
    update_entity_and_release_snapshot(execution, &runtime, entity, "member-after");
    let retained = runtime
        .read_truth()
        .read_observation(execution, &observation)
        .unwrap()
        .unwrap();
    assert_eq!(retained.entities, vec![expected]);
}
