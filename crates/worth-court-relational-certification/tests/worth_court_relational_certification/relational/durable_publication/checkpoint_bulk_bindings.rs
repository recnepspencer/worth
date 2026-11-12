//! Actual interned bulk publications preserve every original endpoint binding.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    config::RelationalRuntimeProfile,
    durability::RelationalCanonicalCommitArtifact as Artifact,
    identity::{KindId, PartitionId},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    symbols::{ClientKey, ClientKeySymbolPolicy},
    transactions::{
        BulkEntityCreateIntent, BulkRelationCreateIntent, CommitResult, CreateIntent,
        CreatedEntityRef, EntityReference, MutationIntent, RelationalCreatedEndpoint as Endpoint,
        WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot},
    durable_medium::TestDurableMedium,
    durable_publication::{field_patch, schema},
};
fn publication(
    execution: &ExecutionRequest<'_, '_>,
    count: usize,
) -> (
    std::sync::Arc<TestDurableMedium>,
    RelationalRuntime,
    CommitResult,
) {
    let medium = TestDurableMedium::new();
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema())
        .client_key_symbol_policy(ClientKeySymbolPolicy::PreferInterned)
        .build_durable(medium.attachment())
        .unwrap();
    let entity_keys = (0..count)
        .map(|index| ClientKey::raw(format!("bulk-input-{index:02}")))
        .collect::<Vec<_>>();
    let entity_reference = |index: usize| {
        EntityReference::Created(CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: entity_keys[index].clone(),
        })
    };
    let batch = WorkerIntentBatch::new("indexed-original-correspondences")
        .push(MutationIntent::Create(CreateIntent::BulkEntities(
            BulkEntityCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_keys: entity_keys.clone(),
                field_patches: (0..count)
                    .map(|index| field_patch("name", "name", &format!("bulk-value-{index:02}")))
                    .collect(),
            },
        )))
        .push(MutationIntent::Create(CreateIntent::BulkRelations(
            BulkRelationCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_keys: (0..count - 1)
                    .map(|index| ClientKey::raw(format!("bulk-edge-{index:02}")))
                    .collect(),
                endpoints: (0..count - 1)
                    .map(|index| (entity_reference(index), entity_reference(index + 1)))
                    .collect(),
                field_patches: (0..count - 1)
                    .map(|index| field_patch("label", "label", &format!("edge-value-{index:02}")))
                    .collect(),
            },
        )));
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let committed = transaction.commit(execution, &runtime).unwrap();
    (medium, runtime, committed)
}

#[test]
fn semantic_result_bulk_entities_and_relations_seal_and_readmit_original_bindings() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (medium, runtime, committed) = publication(execution, 32);
    let semantic = committed.semantic_result();
    assert_eq!(semantic.created_entities().len(), 32);
    assert_eq!(semantic.created_relations().len(), 31);
    for index in [0, 16, 31] {
        let key = format!("bulk-input-{index:02}");
        let assigned = committed
            .created_entity(&CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: key.clone().into(),
            })
            .unwrap();
        assert_eq!(semantic.created_entities()[index].key.client_key, key);
        assert_eq!(semantic.created_entities()[index].assigned, assigned);
    }
    for (index, relation) in semantic.created_relations().iter().enumerate() {
        assert_eq!(relation.key.client_key, format!("bulk-edge-{index:02}"));
        let Endpoint::Created(source) = &relation.key.source else {
            panic!("original create endpoint")
        };
        let Endpoint::Created(target) = &relation.key.target else {
            panic!("original create endpoint")
        };
        assert_eq!(source.client_key, format!("bulk-input-{index:02}"));
        assert_eq!(target.client_key, format!("bulk-input-{:02}", index + 1));
    }
    let artifact = medium.offered().pop().unwrap();
    let admitted = Artifact::readmit(artifact.as_bytes(), execution).unwrap();
    assert_eq!(admitted.envelope().semantic_result(), semantic);
    release_test_commit_snapshot(&runtime, &committed);
}
