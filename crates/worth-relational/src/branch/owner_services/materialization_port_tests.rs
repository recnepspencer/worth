use super::*;
use crate::identity::data::{KindId, PartitionId};
use crate::mvcc::{RelationalBranchTransactionAdmissionDenial, RelationalTransactionIntent};
use crate::tests::support::{
    changed_entities, changed_relations, persisted_runtime_with_test_schema,
    runtime_with_test_schema,
};
use crate::transactions::data::{
    CreateIntent, CreatedEntityRef, EntityReference, EntitySpec, MaterializationMutationIntent,
    RelationSpec, SuspendEntityMaterializationIntent,
};

#[path = "materialization_port_tests/generated_group.rs"]
mod generated_group;

#[test]
fn external_endpoint_group_restores_only_its_exact_relation_endpoints() {
    let runtime = runtime_with_test_schema();
    let external = crate::tests::support::create_entity(&runtime, "external-endpoint");
    let created = create_group_with_external_endpoint(&runtime, external);
    let internal = changed_entities(&created)[0];
    let relation = changed_relations(&created)[0];
    let identity = runtime.main_branch_identity();
    let services = runtime.owner_component_services();
    let (_, basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the output-owned boundary link has a current basis");
    let suspended = services
        .materialization_port()
        .suspend_generated_materialization(&basis, &[internal])
        .expect("a generated relation may retain an endpoint outside its output group");
    assert!(suspended
        .custody
        .records()
        .contains(&RelationalMaterializationRecord::Relation {
            relation_id: relation,
            kind_id: KindId(2),
            source: external,
            target: internal,
        }));
    assert!(runtime
        .read_truth()
        .read_snapshot(&suspended.commit.snapshot)
        .is_none());

    let (_, unavailable_basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the suspended boundary link retains an owner basis");
    let failed = services
        .materialization_port()
        .rematerialize_generated_materialization(
            &unavailable_basis,
            suspended.custody,
            vec![entity_materialization(
                internal,
                crate::tests::support::name_field_patch("internal-endpoint"),
            )],
            vec![RelationalRelationMaterialization {
                relation_id: relation,
                kind_id: KindId(2),
                source: internal,
                target: external,
                fields: crate::tests::support::relation_label_field_patch("external-edge"),
            }],
        )
        .expect_err("endpoint reversal cannot satisfy exact custody");
    assert!(matches!(
        failed.error,
        RelationalMaterializationError::CandidateManifestMismatch
    ));

    let restored = services
        .materialization_port()
        .rematerialize_generated_materialization(
            &unavailable_basis,
            failed.custody,
            vec![entity_materialization(
                internal,
                crate::tests::support::name_field_patch("internal-endpoint"),
            )],
            vec![RelationalRelationMaterialization {
                relation_id: relation,
                kind_id: KindId(2),
                source: external,
                target: internal,
                fields: crate::tests::support::relation_label_field_patch("external-edge"),
            }],
        )
        .expect("the exact external endpoint relationship restores");
    let read = runtime
        .read_truth()
        .read_snapshot(&restored.snapshot)
        .expect("exact restoration resumes ordinary reads");
    assert_eq!(read.relations()[0].source, external);
    assert_eq!(read.relations()[0].target, internal);
    assert!(runtime
        .read_truth()
        .read_snapshot(&created.snapshot)
        .is_some());
}

#[test]
fn durability_replays_suspended_and_restored_materialization_roots() {
    let runtime = persisted_runtime_with_test_schema();
    let (created, entity_fields, relation_fields) = create_generated_group(&runtime);
    let entities = changed_entities(&created);
    let relation = changed_relations(&created)[0];
    let identity = runtime.main_branch_identity();
    let services = runtime.owner_component_services();
    runtime
        .durability_authority()
        .checkpoint()
        .expect("the create publication is the suspension replay checkpoint");
    let (_, source_basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the durable source has an exact basis");
    let suspended = services
        .materialization_port()
        .suspend_generated_materialization(&source_basis, &entities)
        .expect("the durable source suspends through owner authority");

    let (_, recovered) =
        crate::tests::support::recover_with(&runtime, persisted_runtime_with_test_schema);
    let recovered_identity = recovered.main_branch_identity();
    let (_, recovered_basis) = recovered
        .observe_branch(&recovered_identity)
        .expect("recovery restores the suspended branch root");
    assert_suspended_records(&recovered_basis, &entities, relation);
    assert!(matches!(
        recovered
            .begin_branch_transaction(&recovered_basis, RelationalTransactionIntent::ordinary()),
        Err(RelationalBranchTransactionAdmissionDenial::MaterializationUnavailable)
    ));

    let (_, unavailable_basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the original owner retains the suspended basis");
    runtime
        .durability_authority()
        .checkpoint()
        .expect("the suspended publication is the restoration replay checkpoint");
    let restored = services
        .materialization_port()
        .rematerialize_generated_materialization(
            &unavailable_basis,
            suspended.custody,
            vec![
                entity_materialization(entities[0], entity_fields[0].clone()),
                entity_materialization(entities[1], entity_fields[1].clone()),
            ],
            vec![RelationalRelationMaterialization {
                relation_id: relation,
                kind_id: KindId(2),
                source: entities[0],
                target: entities[1],
                fields: relation_fields,
            }],
        )
        .expect("the original owner restores the exact group");
    runtime
        .snapshots()
        .release_snapshot(&restored.snapshot)
        .expect("the durable test releases its restoration snapshot");

    let (_, recovered_restored) =
        crate::tests::support::recover_with(&runtime, persisted_runtime_with_test_schema);
    let current = recovered_restored.visibility_authority().snapshot();
    let read = recovered_restored
        .read_truth()
        .read_snapshot(&current)
        .expect("recovery restores an ordinarily readable rematerialized root");
    assert_eq!(
        read.entities()
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        entities
    );
    assert_eq!(read.relations()[0].relation_id, relation);
}

fn create_generated_group(
    runtime: &crate::runtime::RelationalRuntime,
) -> (
    crate::transactions::data::CommitResult,
    [crate::transactions::data::AspectFieldPatch; 2],
    crate::transactions::data::AspectFieldPatch,
) {
    let source = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("materialized-source"),
    };
    let target = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("materialized-target"),
    };
    let entity_fields = [
        crate::tests::support::name_field_patch("materialized-source"),
        crate::tests::support::name_field_patch("materialized-target"),
    ];
    let relation_fields = crate::tests::support::relation_label_field_patch("materialized-edge");
    let batch = crate::transactions::data::WorkerIntentBatch::new("generated-group")
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: source.partition_id,
            kind_id: source.kind_id,
            client_key: source.client_key.clone(),
            fields: entity_fields[0].clone(),
        })))
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: entity_fields[1].clone(),
        })))
        .push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("materialized-edge"),
                source: EntityReference::Created(source),
                target: EntityReference::Created(target),
                fields: relation_fields.clone(),
            },
        )));
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(batch)
        .expect("the generated group fits the configured transaction budget");
    (
        transaction
            .commit(runtime)
            .expect("the generated group publishes as one complete create commit"),
        entity_fields,
        relation_fields,
    )
}

fn create_group_with_external_endpoint(
    runtime: &crate::runtime::RelationalRuntime,
    external: crate::identity::data::EntityId,
) -> crate::transactions::data::CommitResult {
    let target = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("internal-endpoint"),
    };
    let batch = crate::transactions::data::WorkerIntentBatch::new("open-generated-group")
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: crate::tests::support::name_field_patch("internal-endpoint"),
        })))
        .push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("external-edge"),
                source: EntityReference::Existing(external),
                target: EntityReference::Created(target),
                fields: crate::tests::support::relation_label_field_patch("external-edge"),
            },
        )));
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(batch)
        .expect("the open generated group fits transaction budgets");
    transaction
        .commit(runtime)
        .expect("the open generated group is otherwise a valid all-create publication")
}

fn entity_materialization(
    entity_id: crate::identity::data::EntityId,
    fields: crate::transactions::data::AspectFieldPatch,
) -> RelationalEntityMaterialization {
    RelationalEntityMaterialization {
        entity_id,
        kind_id: KindId(1),
        fields,
    }
}

fn assert_suspended_records(
    basis: &AdmittedRelationalBranchBasis,
    entities: &[crate::identity::data::EntityId],
    relation: crate::identity::data::RelationId,
) {
    let partition = basis
        .inner
        .root
        .partition_state(PartitionId::main())
        .expect("the suspended root retains its partition identity");
    for entity in entities {
        let slot = partition
            .entity_arena
            .get(entity)
            .expect("the suspended entity retains its exact slot and generation");
        assert!(slot.is_materialization_unavailable());
        assert_eq!(slot.kind_id(), Some(KindId(1)));
        assert!(slot.retired_at().is_none());
    }
    let slot = partition
        .relation_arena
        .get(&relation)
        .expect("the suspended relation retains its exact slot and generation");
    assert!(slot.is_materialization_unavailable());
    assert_eq!(slot.kind_id(), Some(KindId(2)));
    assert!(slot.retired_at().is_none());
}
