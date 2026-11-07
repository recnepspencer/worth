//! Request-funded retired peer history, independent of endpoint-owned records.
use crate::relational::durable_publication::source_initial_inputs::{
    checkpoint_admission::cold_materialization::{read_head, recover, update},
    selected_source::SelectedSource,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot},
    durable_publication::{create_entity_outcome, field_patch},
    initial_input_source::{capture_external, initialized, receiver},
};

#[test]
fn retired_peer_history_keeps_its_funding_through_edits_and_returns_real_historical_neighbors() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&setup);
    let mut measured = Vec::new();
    for count in [2, 64] {
        let (runtime, medium) = initialized(request);
        let endpoint = create_entity_outcome(request, &runtime, "peer-endpoint");
        let entity = endpoint.semantic_result().created_entities()[0].assigned;
        release_test_commit_snapshot(&runtime, &endpoint);
        drop(endpoint);
        let editable = create_entity_outcome(request, &runtime, "local-editable");
        let editable_id = editable.semantic_result().created_entities()[0].assigned;
        release_test_commit_snapshot(&runtime, &editable);
        drop(editable);
        let mut transaction = begin_main_transaction(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new("remote-target").push(MutationIntent::Create(
                    CreateIntent::Entity(EntitySpec {
                        partition_id: PartitionId::new(1),
                        kind_id: KindId(1),
                        client_key: "remote".into(),
                        fields: field_patch("name", "name", "remote"),
                    }),
                )),
                request.allocation_policy(),
            )
            .unwrap();
        let target = transaction.commit(request, &runtime).unwrap();
        let target_id = target.semantic_result().created_entities()[0].assigned;
        release_test_commit_snapshot(&runtime, &target);
        drop(target);
        // Identity is kind plus endpoints, so retire each edge before creating
        // the next. Reuse the same vocabulary; only peer history grows here.
        let mut witnesses = Vec::new();
        let mut deleted_version = None;
        for _ in 0..count {
            let batch = WorkerIntentBatch::new("peer-relation").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::new(2),
                    kind_id: KindId(2),
                    client_key: "peer".into(),
                    source: EntityReference::Existing(entity),
                    target: EntityReference::Existing(target_id),
                    fields: field_patch("label", "label", "peer"),
                }),
            ));
            let mut transaction = begin_main_transaction(&runtime);
            transaction
                .push_batch(batch, request.allocation_policy())
                .unwrap();
            let created = transaction.commit(request, &runtime).unwrap();
            let relation_id = created.semantic_result().created_relations()[0].assigned;
            witnesses.push((created.version_id, relation_id));
            release_test_commit_snapshot(&runtime, &created);
            drop(created);
            let batch =
                WorkerIntentBatch::new("retire-peer-relation").push(MutationIntent::Relation(
                    RelationMutationIntent::Delete(DeleteRelationIntent { relation_id }),
                ));
            let mut transaction = begin_main_transaction(&runtime);
            transaction
                .push_batch(batch, request.allocation_policy())
                .unwrap();
            let deleted = transaction.commit(request, &runtime).unwrap();
            deleted_version = Some(deleted.version_id);
            release_test_commit_snapshot(&runtime, &deleted);
            drop(deleted);
        }
        let image = capture_external(&runtime, &medium, request);
        medium.file_checkpoint(image.checkpoint().clone());
        drop(runtime);
        let source = SelectedSource::new(medium.clone());
        let runtime = recover(&image, source.clone(), receiver(&medium), request);
        let budget = SerialMemoryBudget::new(None);
        let caller =
            SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
        read_head(&ExecutionRequest::serial(&caller), &runtime, entity).unwrap();
        update(request, &runtime, editable_id);
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        drop((runtime, caller));
        measured.push(budget.charged_memory_bytes());
        assert!(*measured.last().unwrap() > 0);
        drop(basis);
        assert_eq!(budget.charged_memory_bytes(), 0);

        let runtime = recover(&image, source, receiver(&medium), request);
        for &(version, relation_id) in [witnesses.first().unwrap(), witnesses.last().unwrap()] {
            let historical = runtime
                .read_truth()
                .outgoing_relations_of_kind_at_version(request, entity, KindId(2), version)
                .unwrap();
            assert_eq!(
                historical
                    .iter()
                    .map(|record| record.relation_id)
                    .collect::<Vec<_>>(),
                vec![relation_id]
            );
        }
        assert!(runtime
            .storage_access()
            .outgoing_relations_for_entity(request, entity, deleted_version.unwrap())
            .unwrap()
            .is_empty());
    }
    let peer_identity_bytes =
        (64 - 2) * std::mem::size_of::<worth_relational::facade::identity::RelationId>();
    assert!(
        measured[1] > measured[0] + peer_identity_bytes as u64,
        "retired peer index backing is funded, not only its constant endpoint record: {measured:?}"
    );
}
