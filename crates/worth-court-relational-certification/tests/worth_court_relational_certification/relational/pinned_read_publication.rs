//! A pinned exact frontier preserves the audit record without reviving its endpoint.
use std::collections::BTreeSet;
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_foundational::facade::{
    AspectFieldLocator, CanonicalFieldPath, ContractValidatedAspectValueView, LocatorAuthority,
    RelationDirection,
};
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    history::BranchId,
    identity::{KindId, PartitionId, VersionId},
    indexes::{
        BoundedEntityFieldLookupDenialKind, BoundedIndexParityMode, DerivedIndexBuildRequest,
        DerivedIndexDefinition, DerivedIndexDiscardRequest, DerivedIndexGenerationId,
        DerivedIndexId, DerivedIndexKind,
    },
    query::*,
    runtime::{
        BoundedFrontierReadStop, PinnedStructuralWalkScope, RelationalRuntime, RelationalRuntimeApi,
    },
    schema::{
        EndpointDeletionIntegrityDeclaration, EndpointDeletionIntegrityMode,
        EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
        RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
    snapshots::SnapshotHandle,
    storage::RecordLifecycleState,
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, DeleteEntityIntent, EntityMutationIntent, EntityReference,
        EntitySpec, MutationIntent, RecordRef, RelationSpec, WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key, field_key,
        publications::{begin_main_transaction, changed_entities, string_aspect_value},
    },
    cold_recovery::update_entity,
    durable_publication::{create_entity_outcome, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

fn snapshot_for_main(runtime: &RelationalRuntime) -> SnapshotHandle {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .expect("main remains owner-admitted");
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .expect("the exact main observation opens its snapshot")
}

#[test]
fn pinned_audit_frontier_retains_deleted_target_identity_without_reviving_live_visibility() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let policy = CascadeDeletePolicy::RetainDanglingForAudit;
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".into(),
            schema_id: SchemaId("test".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "test.relation".into(),
            schema_id: SchemaId("test".into()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: policy,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![EndpointDeletionIntegrityDeclaration {
                    contract_id: "endpoint_delete".into(),
                    mode: EndpointDeletionIntegrityMode::RequireRelationRetirement,
                }],
            ),
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .cascade_delete_policy(policy)
        .build()
        .unwrap();
    let publish = |intent| {
        let mut transaction = begin_main_transaction(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new("audit-fixture").push(MutationIntent::Create(intent)),
                execution.allocation_policy(),
            )
            .unwrap();
        transaction.commit(&execution, &runtime).unwrap()
    };
    let create_entity = |key| {
        changed_entities(&publish(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw(key),
            fields: AspectFieldPatch::default(),
        })))[0]
    };
    let source = create_entity("source");
    let target = create_entity("target");
    let created = publish(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: ClientKey::raw("audit-edge"),
        source: EntityReference::Existing(source),
        target: EntityReference::Existing(target),
        fields: AspectFieldPatch::default(),
    }));
    let [RecordRef::Relation(relation)] = created.changed_records.as_slice() else {
        panic!("fixture creates one relation");
    };
    let relation = *relation;
    let before = runtime
        .read_truth()
        .pin_read_capability(&snapshot_for_main(&runtime), &[], &execution)
        .unwrap();
    let frontier = BTreeSet::from([source]);
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("retire-target").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: target }),
            )),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(&execution, &runtime).unwrap();
    let after = runtime
        .read_truth()
        .pin_read_capability(&snapshot_for_main(&runtime), &[], &execution)
        .unwrap();
    let walk_scope = || {
        PinnedStructuralWalkScope::new(
            std::slice::from_ref(&source),
            KindId(1),
            KindId(2),
            RelationDirection::Forward,
        )
    };
    let old_walk = before
        .complete_relation_walk(&execution, walk_scope())
        .unwrap();
    let new_walk = after
        .complete_relation_walk(&execution, walk_scope())
        .unwrap();
    assert_eq!(old_walk.visits().len(), 2);
    assert_eq!(old_walk.relations().len(), 1);
    assert_eq!(new_walk.visits().len(), 1);
    assert!(new_walk.relations().is_empty());
    let old = before
        .bounded_outgoing_relations_for_frontier(&execution, &frontier, KindId(2), Some(8))
        .unwrap()
        .into_records();
    assert_eq!(old.len(), 1);
    assert_eq!(old[0].relation_id, relation);
    assert_eq!(old[0].lifecycle, RecordLifecycleState::Live);
    assert!(after
        .bounded_outgoing_relations_for_frontier(&execution, &frontier, KindId(2), Some(8))
        .unwrap()
        .into_records()
        .is_empty());
    let retained = after
        .bounded_outgoing_relations_including_audit_for_frontier(
            &execution,
            &frontier,
            KindId(2),
            Some(8),
        )
        .unwrap()
        .into_records();
    assert_eq!(retained.len(), 1);
    assert_eq!(
        (
            retained[0].relation_id,
            retained[0].source,
            retained[0].target
        ),
        (relation, source, target)
    );
    assert_eq!(
        retained[0].lifecycle,
        RecordLifecycleState::RetainedDanglingForAudit
    );
    assert!(matches!(
        after.bounded_outgoing_relations_including_audit_for_frontier(
            &execution,
            &frontier,
            KindId(2),
            Some(1)
        ),
        Err(BoundedFrontierReadStop::Limit(_))
    ));
}

#[test]
fn pinned_read_keeps_rows_and_generation_across_a_commit_and_discard() {
    let host = serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let created = create_entity_outcome(&request, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key("name"),
        CanonicalFieldPath::single(field_key("name")),
    );
    let definition = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(17),
            name: "commit-control".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: locator.clone(),
            },
            branch_scoped: false,
        })
        .expect("index fixture has identity capacity");
    let snapshot = snapshot_for_main(&runtime);
    let cold = runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[definition.index_id], &request)
        .unwrap();
    let (_, admitted) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let build = runtime.index_authority().build_for_basis(
        &request,
        DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![definition.index_id],
        },
        &admitted,
    );
    assert!(build.failed_indexes.is_empty());
    assert_eq!(
        cold.prepare_entity_field_lookup(definition.index_id, KindId(1), &locator, &request)
            .unwrap_err()
            .kind(),
        BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
    );
    let read = runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[definition.index_id], &request)
        .unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "commit-control".into(),
                context_id: read.query_context().clone(),
                scope: QueryScope::ExplicitTargets {
                    targets: vec![RecordRef::Entity(entity)].into(),
                },
                locality: QueryLocalityClass::SinglePartition {
                    partition_id: entity.partition_id,
                },
                ordering: QueryOrderingContract::CanonicalRecordRefOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(91765),
                target_count_hint: 1,
            },
        )
        .unwrap();
    update_entity(&request, &runtime, entity, "after");
    runtime
        .index_authority()
        .discard_generations(DerivedIndexDiscardRequest::all_bases(definition.index_id))
        .unwrap();
    let late_view = runtime.read_truth().project_snapshot(&snapshot).unwrap();
    assert_eq!(
        runtime
            .index_access()
            .prepare_entity_field_lookup(
                &late_view,
                definition.index_id,
                KindId(1),
                &locator,
                &request
            )
            .unwrap_err()
            .kind(),
        BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
    );
    let rows = read
        .execute_query_plan(plan, &request)
        .unwrap()
        .unwrap()
        .outcome
        .result
        .entities;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].entity_id, entity);
    let value = rows[0]
        .authoritative_aspect_state
        .as_ref()
        .and_then(|state| state.get(&aspect_key("name")))
        .expect("the reached entity carries its name");
    assert_eq!(
        value.view(),
        ContractValidatedAspectValueView::Scalar(&string_aspect_value("before"))
    );
    let prepared = read
        .prepare_entity_field_lookup(definition.index_id, KindId(1), &locator, &request)
        .unwrap();
    let before = prepared
        .execute_prepared_entity_field_lookup(
            &string_aspect_value("before"),
            2,
            BoundedIndexParityMode::Certification,
            &request,
            || Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(before.candidate_entity_ids(), &[entity]);
    assert_eq!(before.generation_id(), DerivedIndexGenerationId(1));
    let after = prepared
        .execute_prepared_entity_field_lookup(
            &string_aspect_value("after"),
            2,
            BoundedIndexParityMode::Production,
            &request,
            || Ok::<_, ()>(()),
        )
        .unwrap();
    assert!(after.candidate_entity_ids().is_empty());
    assert_eq!(
        runtime
            .read_truth()
            .query_plan_context(&snapshot_for_main(&runtime))
            .unwrap()
            .version_id,
        VersionId(2)
    );
}
