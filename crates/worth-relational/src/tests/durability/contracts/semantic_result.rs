//! Real owner semantics are captured before sealing and recovered as data.
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Artifact;
use crate::facade::mvcc::RelationalPublicationOutcome;
use crate::tests::support::*;
use crate::transactions::data::{
    CreatedEntityRef, EntityReference, RelationalCreatedEndpoint as Endpoint,
};
use worth_execution::ExecutionRequest;

mod bulk;
mod custom_versions;

#[test]
fn semantic_result_preserves_original_create_bindings_and_validation_meaning() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let source = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: "semantic-source".into(),
    };
    let target = CreatedEntityRef {
        client_key: "semantic-target".into(),
        ..source.clone()
    };
    let batch = WorkerIntentBatch::new("semantic-original")
        .push(batch_create("semantic-source").intents.pop().unwrap())
        .push(batch_create("semantic-target").intents.pop().unwrap())
        .push(MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: "semantic-edge".into(),
                source: EntityReference::Created(source.clone()),
                target: EntityReference::Created(target.clone()),
                fields: relation_label_field_patch("semantic-edge"),
            },
        )));
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let proposal = transaction.validate(execution, &runtime).unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("real candidate performs")
    };
    let envelope = performed.canonical_commit().clone();
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let original = committed.semantic_result();
    assert!(std::ptr::eq(original, envelope.semantic_result()));
    assert_eq!(original.receipt(), &committed.commit);
    assert_eq!(
        original.changed_records(),
        committed.changed_records.as_slice()
    );
    assert_eq!(original.created_entities().len(), 2);
    let assigned_source = committed.created_entity(&source).unwrap();
    let assigned_target = committed.created_entity(&target).unwrap();
    for (text, assigned) in [
        ("semantic-source", assigned_source),
        ("semantic-target", assigned_target),
    ] {
        let binding = original
            .created_entities()
            .iter()
            .find(|entry| entry.key.client_key == text)
            .unwrap();
        assert_eq!(binding.key.partition, PartitionId::main());
        assert_eq!(binding.key.kind, KindId(1));
        assert_eq!(binding.assigned, assigned);
    }
    let relation = &original.created_relations()[0];
    assert_eq!(relation.key.client_key, "semantic-edge");
    assert_eq!(relation.key.kind, KindId(2));
    let Endpoint::Created(source_key) = &relation.key.source else {
        panic!("original created endpoint")
    };
    let Endpoint::Created(target_key) = &relation.key.target else {
        panic!("original created endpoint")
    };
    assert_eq!(source_key.client_key, "semantic-source");
    assert_eq!(target_key.client_key, "semantic-target");
    assert!(committed
        .changed_records
        .as_slice()
        .contains(&RecordRef::Relation(relation.assigned)));
    assert_eq!(
        original.publication().patch_record_count,
        committed.patch().len()
    );
    assert_eq!(
        original.validation_summary(),
        committed.validation().summary()
    );
    let encoded = serde_json::to_value(original).unwrap();
    for excluded in [
        "transaction_id",
        "proposal_identity",
        "snapshot",
        "phase_timing",
        "complexity_delta",
        "work_units",
        "traversal",
        "publication_status",
        "patch_position",
        "payload_digest",
    ] {
        assert!(
            !encoded.to_string().contains(&format!("\"{excluded}\"")),
            "live-only {excluded}"
        );
    }
    let artifact = attached_test_medium(&runtime).offered().pop().unwrap();
    assert_eq!(Artifact::FORMAT_VERSION, 3);
    let readmitted = Artifact::readmit(artifact.as_bytes(), execution).unwrap();
    assert_eq!(readmitted.envelope().semantic_result(), original);
    release_test_commit_snapshot(&runtime, &committed);
}

#[test]
fn semantic_result_is_required_even_after_a_corrupt_frame_is_resealed() {
    use sha2::{Digest as _, Sha256};
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &runtime, "required-semantic-image");
    let artifact = attached_test_medium(&runtime).offered().pop().unwrap();
    let encoded_image = rmp_serde::to_vec_named(committed.semantic_result()).unwrap();
    let mut bytes = artifact.as_bytes().to_vec();
    let offsets = bytes
        .windows(encoded_image.len())
        .enumerate()
        .filter_map(|(offset, data)| (data == encoded_image).then_some(offset))
        .collect::<Vec<_>>();
    assert_eq!(
        offsets.len(),
        1,
        "the real artifact contains the original image once"
    );
    bytes.splice(offsets[0]..offsets[0] + encoded_image.len(), [0xc0]);
    let length = (bytes.len() - 66) as u64;
    bytes[58..66].copy_from_slice(&length.to_le_bytes());
    let digest = Sha256::digest(&bytes[66..]);
    bytes[26..58].copy_from_slice(&digest);
    assert!(
        Artifact::from_untrusted_bytes(bytes.clone()).is_ok(),
        "frame proof alone is insufficient"
    );
    assert!(matches!(
        Artifact::readmit(&bytes, execution),
        Err(
            crate::facade::durability::RelationalCanonicalCommitArtifactRefusal::PayloadDecode { .. }
        )
    ));
    release_test_commit_snapshot(&runtime, &committed);
}

#[test]
fn semantic_result_discovery_must_be_present_in_each_actual_index_entry() {
    use crate::history::RelationalCommitIndexEntry;
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &runtime, "mandatory-index-discovery");
    let entry = runtime
        .history
        .commit_artifact(committed.commit.commit_id)
        .unwrap()
        .envelope_residency()
        .journal_index()
        .unwrap();
    assert_eq!(
        entry.association_discovery,
        crate::history::data::RelationalAssociationDiscovery::Unassociated
    );
    let mut wire = serde_json::to_value(entry.as_ref()).unwrap();
    assert_eq!(
        serde_json::from_value::<RelationalCommitIndexEntry>(wire.clone()).unwrap(),
        *entry
    );
    wire.as_object_mut()
        .unwrap()
        .remove("association_discovery")
        .unwrap();
    let failure = serde_json::from_value::<RelationalCommitIndexEntry>(wire).unwrap_err();
    assert!(
        failure.to_string().contains("association_discovery"),
        "{failure}"
    );
    release_test_commit_snapshot(&runtime, &committed);
}
