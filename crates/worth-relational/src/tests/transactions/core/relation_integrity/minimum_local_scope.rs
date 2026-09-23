use super::fixtures::{commit_source_min_one_runtime, commit_target_min_one_runtime};
use crate::tests::support::*;
use crate::transactions::data::{CreatedEntityRef, CreatedRelationRef, EntityReference};

#[test]
fn adding_an_edge_does_not_rejudge_an_unchanged_neighbor_with_partial_counts() {
    let runtime = commit_source_min_one_runtime();
    let (entities, _) = seed_graph(
        &runtime,
        &["source", "target"],
        &[("ss", 0, 0), ("tt", 1, 1)],
    );

    create_relation(&runtime, entities[0], entities[1], "new-edge");
}

#[test]
fn deleting_one_edge_preserves_a_survivors_other_outgoing_edges() {
    let runtime = commit_source_min_one_runtime();
    let (_, relations) = seed_graph(
        &runtime,
        &["source", "removed-target", "neighbor"],
        &[
            ("removed-edge", 0, 1),
            ("remaining-edge", 0, 2),
            ("tt", 1, 1),
            ("nn", 2, 2),
        ],
    );
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("remove-one").push(MutationIntent::Relation(
            crate::transactions::data::RelationMutationIntent::Delete(
                crate::transactions::data::DeleteRelationIntent {
                    relation_id: relations[0],
                },
            ),
        )),
    )
    .expect("stage one relation deletion");

    let outcome = txn.commit(&runtime).expect("source retains another edge");
    release_test_commit_snapshot(&runtime, &outcome);
}

#[test]
fn deleting_an_endpoint_preserves_a_survivors_other_outgoing_edges() {
    let runtime = commit_source_min_one_runtime();
    let (entities, _) = seed_graph(
        &runtime,
        &["source", "removed", "remaining"],
        &[
            ("removed-edge", 0, 1),
            ("remaining-edge", 0, 2),
            ("rr", 1, 1),
            ("xx", 2, 2),
        ],
    );
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("remove-endpoint").push(MutationIntent::Entity(
            crate::transactions::data::EntityMutationIntent::Delete(
                crate::transactions::data::DeleteEntityIntent {
                    entity_id: entities[1],
                },
            ),
        )),
    )
    .expect("stage endpoint deletion");

    let outcome = txn.commit(&runtime).expect("survivor retains another edge");
    release_test_commit_snapshot(&runtime, &outcome);
}

#[test]
fn deleting_an_endpoint_preserves_a_survivors_unchanged_incoming_edges() {
    let runtime = commit_target_min_one_runtime();
    let (entities, _) = seed_graph(
        &runtime,
        &["source", "removed", "incoming-source"],
        &[
            ("source-to-removed", 0, 1),
            ("incoming-to-source", 2, 0),
            ("rr", 1, 1),
            ("qq", 2, 2),
        ],
    );
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("remove-endpoint").push(MutationIntent::Entity(
            crate::transactions::data::EntityMutationIntent::Delete(
                crate::transactions::data::DeleteEntityIntent {
                    entity_id: entities[1],
                },
            ),
        )),
    )
    .expect("stage endpoint deletion");

    let outcome = txn
        .commit(&runtime)
        .expect("survivor retains incoming edge");
    release_test_commit_snapshot(&runtime, &outcome);
}

#[test]
fn deleting_the_only_incoming_edge_and_its_target_does_not_recheck_the_deleted_target() {
    let runtime = commit_target_min_one_runtime();
    let (entities, relations) = seed_graph(
        &runtime,
        &["survivor", "removed"],
        &[("survivor-loop", 0, 0), ("removed-edge", 0, 1)],
    );
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("remove-edge-and-target")
            .push(MutationIntent::Relation(
                crate::transactions::data::RelationMutationIntent::Delete(
                    crate::transactions::data::DeleteRelationIntent {
                        relation_id: relations[1],
                    },
                ),
            ))
            .push(MutationIntent::Entity(
                crate::transactions::data::EntityMutationIntent::Delete(
                    crate::transactions::data::DeleteEntityIntent {
                        entity_id: entities[1],
                    },
                ),
            )),
    )
    .expect("stage the relation and target deletion together");

    let outcome = txn
        .commit(&runtime)
        .expect("the deleted target has no remaining minimum");
    release_test_commit_snapshot(&runtime, &outcome);
}

#[test]
fn deleting_the_only_incoming_edge_without_its_target_denies_the_target_minimum() {
    let runtime = commit_target_min_one_runtime();
    let (_, relations) = seed_graph(
        &runtime,
        &["survivor", "retained-target"],
        &[("survivor-loop", 0, 0), ("removed-edge", 0, 1)],
    );
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("remove-edge-only").push(MutationIntent::Relation(
            crate::transactions::data::RelationMutationIntent::Delete(
                crate::transactions::data::DeleteRelationIntent {
                    relation_id: relations[1],
                },
            ),
        )),
    )
    .expect("stage the relation deletion without the target deletion");

    let error = txn
        .commit(&runtime)
        .expect_err("the retained target still requires an incoming edge");
    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code() == DiagnosticCode::RelationCardinalityViolation
                && conflict.detail.contains("target_min_one")
                && conflict.detail.contains("0 < 1")
    ));
}

fn seed_graph(
    runtime: &RelationalRuntime,
    names: &[&str],
    edges: &[(&str, usize, usize)],
) -> (Vec<crate::identity::data::EntityId>, Vec<RelationId>) {
    let created_entities = names
        .iter()
        .map(|name| CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: crate::symbols::data::ClientKey::raw(*name),
        })
        .collect::<Vec<_>>();
    let created_relations = edges
        .iter()
        .map(|(key, source, target)| CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: crate::symbols::data::ClientKey::raw(*key),
            source: EntityReference::Created(created_entities[*source].clone()),
            target: EntityReference::Created(created_entities[*target].clone()),
        })
        .collect::<Vec<_>>();
    let mut batch = WorkerIntentBatch::new("seed-valid-graph");
    for created in &created_entities {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: created.partition_id,
                kind_id: created.kind_id,
                client_key: created.client_key.clone(),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        )));
    }
    for created in &created_relations {
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: created.partition_id,
                kind_id: created.kind_id,
                client_key: created.client_key.clone(),
                source: created.source.clone(),
                target: created.target.clone(),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        )));
    }
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(batch).expect("stage valid graph");
    let outcome = txn.commit(runtime).expect("seed valid minimum graph");
    let entities = created_entities
        .iter()
        .map(|created| outcome.created_entity(created).expect("created entity"))
        .collect();
    let relations = created_relations
        .iter()
        .map(|created| outcome.created_relation(created).expect("created relation"))
        .collect();
    release_test_commit_snapshot(runtime, &outcome);
    (entities, relations)
}
