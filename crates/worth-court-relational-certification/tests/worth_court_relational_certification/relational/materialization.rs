//! Physical payload restoration with Native orbit and publication storage installed.
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AggregateDefinitionIdentity, AspectIdentity};
use worth_relational::facade::{
    branch::*,
    config::{CascadeDeletePolicy, CrossContextPolicy},
    identity::{KindId, PartitionId},
    mvcc::{
        PreparedRelationalCommitCandidate, RelationalPublicationOutcome,
        RelationalTransactionIntent,
    },
    runtime::*,
    schema::*,
    symbols::ClientKey,
    transactions::*,
};

fn registry() -> RelationalSchemaRegistry {
    let schema = SchemaId("test".into());
    let version = SchemaVersionId(1);
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".into(),
            schema_id: schema.clone(),
            schema_version_id: version,
            aspect_contract_declarations: Default::default(),
        })
        .unwrap()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "test.successor".into(),
            schema_id: schema.clone(),
            schema_version_id: version,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: Default::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![EndpointKindContractDeclaration {
                    contract_id: "successor:endpoints".into(),
                    allowed_source_kinds: vec![KindId(1)],
                    allowed_target_kinds: vec![KindId(1)],
                    self_edges_allowed: true,
                    cross_context_policy: CrossContextPolicy::AllowExplicit,
                }],
                vec![CardinalityContractDeclaration {
                    contract_id: "successor:orbit".into(),
                    source_min: Some(1),
                    source_max: Some(1),
                    target_min: Some(1),
                    target_max: Some(1),
                    pair_min: None,
                    pair_max: None,
                    pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
                    minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
                }],
                vec![],
                vec![],
                vec![],
            ),
        })
        .unwrap();
    let descriptor = AggregateIdentityContractDeclaration {
        definition: AggregateDefinitionIdentity::from_bytes([7; 32]),
        source: AggregateIdentitySource::Orbit(KindId(2)),
        member_kind: KindId(1),
        membership_kind: KindId(5),
        storage: AggregateIdentityStorage {
            aggregate_kind: KindId(3),
            lineage_kind: KindId(4),
            aggregate_aspect: AspectIdentity(3001),
            lineage_aspect: AspectIdentity(3002),
            projected: None,
        },
    };
    let registry =
        register_aggregate_identity_storage(registry, &schema, version, descriptor.storage)
            .unwrap();
    let registry = register_aggregate_membership(registry, &schema, version, descriptor).unwrap();
    register_current_publication_storage(registry, &schema, version).unwrap()
}

fn created(index: usize) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(format!("member-{index}")),
    }
}

#[test]
fn generated_payload_with_business_edges_restores_with_current_publication_storage() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry())
        .build()
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let mut batch = WorkerIntentBatch::new("orbit-birth");
    for index in 0..3 {
        let reference = created(index);
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: reference.partition_id,
            kind_id: reference.kind_id,
            client_key: reference.client_key,
            fields: Default::default(),
        })));
    }
    for (index, target) in [1, 2, 0].into_iter().enumerate() {
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw(format!("successor-{index}")),
                source: EntityReference::Created(created(index)),
                target: EntityReference::Created(created(target)),
                fields: Default::default(),
            },
        )));
    }
    tx.push_batch(batch, execution.allocation_policy()).unwrap();
    let born = tx.commit(execution, &runtime).unwrap();
    let members: Vec<_> = (0..3)
        .map(|i| born.created_entity(&created(i)).unwrap())
        .collect();
    let read = runtime
        .read_truth()
        .read_snapshot(&born.snapshot, execution)
        .unwrap()
        .unwrap();
    // Compare the public current image, excluding lifecycle versions advanced by restoration.
    let original_entities: Vec<_> = read
        .entities()
        .iter()
        .map(|e| {
            (
                e.entity_id,
                e.kind.clone(),
                e.authoritative_aspect_state.clone(),
            )
        })
        .collect();
    let original_relations: Vec<_> = read
        .relations()
        .iter()
        .map(|r| {
            (
                r.relation_id,
                r.kind.clone(),
                r.source,
                r.target,
                r.authoritative_aspect_state.clone(),
            )
        })
        .collect();
    let business: Vec<_> = read
        .relations()
        .iter()
        .filter(|r| r.kind.kind_id == KindId(2))
        .map(|r| RelationalRelationMaterialization {
            relation_id: r.relation_id,
            kind_id: KindId(2),
            source: r.source,
            target: r.target,
            fields: AspectFieldPatch::default(),
        })
        .collect();
    assert_eq!(business.len(), 3);
    assert_eq!(
        read.relations()
            .iter()
            .filter(|r| r.kind.kind_id == KindId(5))
            .count(),
        3
    );
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let port = runtime.owner_component_services().materialization_port();
    let prepared = port
        .prepare_generated_materialization_suspension(execution, &basis, &members)
        .unwrap();
    let (candidate, completion) = prepared.into_parts();
    let suspended = completion
        .complete(publish(&runtime, execution, candidate))
        .unwrap();
    assert_eq!(suspended.custody.records().len(), 6);
    assert!(suspended.custody.records().iter().all(|r| match r {
        RelationalMaterializationRecord::Entity { kind_id, .. } => *kind_id == KindId(1),
        RelationalMaterializationRecord::Relation { kind_id, .. } => *kind_id == KindId(2),
    }));
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let entities = members
        .into_iter()
        .map(|entity_id| RelationalEntityMaterialization {
            entity_id,
            kind_id: KindId(1),
            fields: AspectFieldPatch::default(),
        })
        .collect();
    let prepared = port
        .prepare_generated_rematerialization(
            execution,
            &basis,
            suspended.custody,
            entities,
            business,
        )
        .unwrap_or_else(|f| panic!("restore the exact authored payload: {:?}", f.error));
    let (candidate, completion, _) = prepared.into_parts();
    let restored = publish(&runtime, execution, candidate);
    let restored = completion.complete(restored).unwrap();
    let read = runtime
        .read_truth()
        .read_snapshot(&restored.snapshot, execution)
        .unwrap()
        .unwrap();
    let entities: Vec<_> = read
        .entities()
        .iter()
        .map(|e| {
            (
                e.entity_id,
                e.kind.clone(),
                e.authoritative_aspect_state.clone(),
            )
        })
        .collect();
    let relations: Vec<_> = read
        .relations()
        .iter()
        .map(|r| {
            (
                r.relation_id,
                r.kind.clone(),
                r.source,
                r.target,
                r.authoritative_aspect_state.clone(),
            )
        })
        .collect();
    assert_eq!(
        entities, original_entities,
        "all payload and Native aggregate records retain identity and meaning"
    );
    assert_eq!(
        relations, original_relations,
        "business edges and Native membership links restore exactly"
    );
}

fn publish(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    candidate: PreparedRelationalCommitCandidate,
) -> CommitResult {
    let services = runtime.owner_component_services();
    let RelationalPublicationOutcome::Performed(performed) = services
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("owner candidate must publish")
    };
    services
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap()
}
