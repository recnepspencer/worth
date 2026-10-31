//! Real bulk preparation and source refusal exercise indexed correspondence checks.
use super::*;
use crate::symbols::data::ClientKey;
use crate::transactions::data::{
    BulkRelationCreateIntent, RelationalCommitSemanticResultRefusal as SemanticRefusal,
};

fn publication(
    execution: &ExecutionRequest<'_, '_>,
    count: usize,
) -> (RelationalRuntime, CommitResult) {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .client_key_symbol_policy(ClientKeySymbolPolicy::PreferInterned)
        .build_durable(TestDurableMedium::new().attachment())
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
                    .map(|index| name_field_patch(&format!("bulk-value-{index:02}")))
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
                    .map(|index| relation_label_field_patch(&format!("edge-value-{index:02}")))
                    .collect(),
            },
        )));
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let committed = transaction.commit(execution, &runtime).unwrap();
    (runtime, committed)
}

#[test]
fn semantic_result_bulk_entities_and_relations_seal_and_readmit_original_bindings() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (runtime, committed) = publication(execution, 32);
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
    let artifact = attached_test_medium(&runtime).offered().pop().unwrap();
    let admitted = Artifact::readmit(artifact.as_bytes(), execution).unwrap();
    assert_eq!(admitted.envelope().semantic_result(), semantic);
    assert!(
        admitted
            .envelope()
            .symbol_texts
            .entries
            .iter()
            .any(|(_, text)| text.as_str() == "bulk-input-00"),
        "the actual symbol dictionary participates in this bulk source"
    );
    release_test_commit_snapshot(&runtime, &committed);
}

#[test]
fn semantic_result_bulk_malformed_correspondence_refuses_after_frame_reseal() {
    use sha2::{Digest as _, Sha256};
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (runtime, committed) = publication(execution, 8);
    let artifact = attached_test_medium(&runtime).offered().pop().unwrap();
    let original = rmp_serde::to_vec_named(committed.semantic_result()).unwrap();
    let mut malformed = committed.semantic_result().clone();
    malformed.created_relations[0]
        .key
        .client_key
        .push_str("-foreign");
    let replacement = rmp_serde::to_vec_named(&malformed).unwrap();
    let mut bytes = artifact.as_bytes().to_vec();
    let offsets = bytes
        .windows(original.len())
        .enumerate()
        .filter_map(|(offset, found)| (found == original).then_some(offset))
        .collect::<Vec<_>>();
    assert_eq!(offsets.len(), 1);
    bytes.splice(offsets[0]..offsets[0] + original.len(), replacement);
    let length = (bytes.len() - 66) as u64;
    bytes[58..66].copy_from_slice(&length.to_le_bytes());
    let digest = Sha256::digest(&bytes[66..]);
    bytes[26..58].copy_from_slice(&digest);
    assert!(Artifact::from_untrusted_bytes(bytes.clone()).is_ok());
    assert!(matches!(
        Artifact::readmit(&bytes, execution),
        Err(
            crate::facade::durability::RelationalCanonicalCommitArtifactRefusal::PositionAdmission(
                crate::runtime::RelationalCanonicalPositionAdmissionDenial::SemanticResult(
                    SemanticRefusal::ConflictingCreateCorrespondence
                )
            )
        )
    ));
    release_test_commit_snapshot(&runtime, &committed);
}
