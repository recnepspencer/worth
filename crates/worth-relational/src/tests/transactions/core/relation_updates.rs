use crate::facade::transactions::{CreatedEntityRef, EntityReference, EntitySpec, RelationSpec};
use crate::tests::support::*;
use crate::transactions::data::ConflictClass;

#[test]
fn relation_endpoint_update_preserves_relation_identity_and_rewrites_endpoints() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let original_target = create_entity(&runtime, "original-target");
    let new_target = create_entity(&runtime, "new-target");
    let relation = create_relation(&runtime, source, original_target, "edge");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("rewire-relation").push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: relation,
                kind_id: KindId(2),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(new_target),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(&runtime).expect("relation update should commit");
    let read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .expect("updated snapshot should read");
    let updated = read
        .relations()
        .iter()
        .find(|record| record.relation_id == relation)
        .expect("updated relation should remain visible");

    assert_eq!(updated.relation_id, relation);
    assert_eq!(updated.source, source);
    assert_eq!(updated.target, new_target);
    assert!(runtime
        .storage_access()
        .all_relations_for_entity(source, outcome.version_id)
        .contains(&relation));
    assert!(!runtime
        .storage_access()
        .all_relations_for_entity(original_target, outcome.version_id)
        .contains(&relation));
    assert!(runtime
        .storage_access()
        .all_relations_for_entity(new_target, outcome.version_id)
        .contains(&relation));
}

#[test]
fn relation_endpoint_update_rejects_duplicate_relation_identity() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let left = create_entity(&runtime, "left");
    let right = create_entity(&runtime, "right");
    let first = create_relation(&runtime, source, left, "edge-left");
    let _second = create_relation(&runtime, source, right, "edge-right");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("duplicate-rewire").push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: first,
                kind_id: KindId(2),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(right),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn
        .commit(&runtime)
        .expect_err("duplicate relation identity should deny");

    match error {
        TransactionCommitError::Conflict { error, .. } => {
            assert!(matches!(
                error.class,
                ConflictClass::DuplicateRelationIdentity { .. }
            ));
        }
        other => panic!("expected duplicate relation identity conflict, got {other:?}"),
    }
}

#[test]
fn relation_endpoint_update_accepts_same_batch_created_target() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let old_target = create_entity(&runtime, "old-target");
    let relation = create_relation(&runtime, source, old_target, "edge");
    let created_target = CreatedEntityRef {
        partition_id: PartitionId(1),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("same-batch-target"),
    };

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("rewire-to-created")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: created_target.partition_id,
                kind_id: created_target.kind_id,
                client_key: created_target.client_key.clone(),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            })))
            .push(MutationIntent::Relation(
                RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                    relation_id: relation,
                    kind_id: KindId(2),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Created(created_target.clone()),
                }),
            )),
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(&runtime)
        .expect("relation update to same-batch created target should commit");
    let read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .expect("updated snapshot should read");
    let updated = read
        .relations()
        .iter()
        .find(|record| record.relation_id == relation)
        .expect("updated relation should remain visible");
    let created_target_id = updated.target;

    assert_eq!(updated.relation_id, relation);
    assert_eq!(updated.source, source);
    assert_eq!(updated.target, created_target_id);
    assert!(runtime
        .storage_access()
        .all_relations_for_entity(created_target_id, outcome.version_id)
        .contains(&relation));
}

#[test]
fn relation_endpoint_update_to_same_batch_created_target_survives_old_target_retirement() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let old_target = create_entity(&runtime, "old-target");
    let relation = create_relation(&runtime, source, old_target, "edge");
    let created_target = CreatedEntityRef {
        partition_id: PartitionId(1),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("rewired-target"),
    };

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("rewire-and-retire-old-target")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: created_target.partition_id,
                kind_id: created_target.kind_id,
                client_key: created_target.client_key.clone(),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            })))
            .push(MutationIntent::Relation(
                RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                    relation_id: relation,
                    kind_id: KindId(2),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Created(created_target),
                }),
            ))
            .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent {
                    entity_id: old_target,
                },
            ))),
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(&runtime)
        .expect("moved relation should survive retirement of its old endpoint");
    let read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .expect("updated snapshot should read");
    let updated = read
        .relations()
        .iter()
        .find(|record| record.relation_id == relation)
        .expect("moved relation should remain visible after old target retirement");
    let created_target_id = updated.target;

    assert_eq!(updated.source, source);
    assert_ne!(updated.target, old_target);
    assert!(runtime
        .storage_access()
        .all_relations_for_entity(source, outcome.version_id)
        .contains(&relation));
    assert!(runtime
        .storage_access()
        .all_relations_for_entity(created_target_id, outcome.version_id)
        .contains(&relation));
    assert!(!runtime
        .storage_access()
        .all_relations_for_entity(old_target, outcome.version_id)
        .contains(&relation));
    assert!(read
        .entities()
        .iter()
        .all(|record| record.entity_id != old_target));
}

#[test]
fn relation_endpoint_update_to_same_batch_created_source_survives_old_source_retirement() {
    let runtime = runtime_with_test_schema();
    let old_source = create_entity(&runtime, "old-source");
    let target = create_entity(&runtime, "target");
    let relation = create_relation(&runtime, old_source, target, "edge");
    let created_source = CreatedEntityRef {
        partition_id: PartitionId(1),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("rewired-source"),
    };

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("rewire-and-retire-old-source")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: created_source.partition_id,
                kind_id: created_source.kind_id,
                client_key: created_source.client_key.clone(),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            })))
            .push(MutationIntent::Relation(
                RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                    relation_id: relation,
                    kind_id: KindId(2),
                    source: EntityReference::Created(created_source),
                    target: EntityReference::Existing(target),
                }),
            ))
            .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent {
                    entity_id: old_source,
                },
            ))),
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(&runtime)
        .expect("moved relation should survive retirement of its old source");
    let read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .expect("updated snapshot should read");
    let updated = read
        .relations()
        .iter()
        .find(|record| record.relation_id == relation)
        .expect("moved relation should remain visible after old source retirement");
    let created_source_id = updated.source;

    assert_eq!(updated.target, target);
    assert_ne!(updated.source, old_source);
    assert!(runtime
        .storage_access()
        .all_relations_for_entity(created_source_id, outcome.version_id)
        .contains(&relation));
    assert!(runtime
        .storage_access()
        .all_relations_for_entity(target, outcome.version_id)
        .contains(&relation));
    assert!(!runtime
        .storage_access()
        .all_relations_for_entity(old_source, outcome.version_id)
        .contains(&relation));
    assert!(read
        .entities()
        .iter()
        .all(|record| record.entity_id != old_source));
}

#[test]
fn relation_identity_can_be_replaced_in_one_transaction() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let retired = create_relation(&runtime, source, target, "retired-edge");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("replace-relation-identity")
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent {
                    relation_id: retired,
                },
            )))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: crate::symbols::data::ClientKey::raw("replacement-edge"),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                },
            ))),
    )
    .expect("test staging stays within configured resource budgets");

    let outcome = txn
        .commit(&runtime)
        .expect("retiring an identity permits its same-transaction replacement");
    let read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .expect("replacement snapshot should read");
    let replacements = read
        .relations()
        .iter()
        .filter(|record| {
            record.kind.kind_id == KindId(2) && record.source == source && record.target == target
        })
        .collect::<Vec<_>>();

    assert_eq!(replacements.len(), 1);
    assert_ne!(replacements[0].relation_id, retired);
}
