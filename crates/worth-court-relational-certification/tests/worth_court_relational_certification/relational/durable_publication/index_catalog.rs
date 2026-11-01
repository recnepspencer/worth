//! Definition identities are source truth; generations remain derived.
use super::*;
use serde_json::Value;
use worth_relational::facade::{
    history::BranchId,
    indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
    runtime::RelationalRuntime,
};
mod recovery;
mod wire;

fn definition(name: &str) -> DerivedIndexDefinition {
    DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: name.into(),
        branch_scoped: true,
        kind: DerivedIndexKind::EntityField {
            field_locator: worth_relational::facade::transactions::planned_single_field_locator(
                worth_foundational::facade::AspectKey::new("name").unwrap(),
                worth_foundational::facade::FieldKey::new("name").unwrap(),
            ),
        },
    }
}
fn delta(result: &worth_relational::facade::transactions::CommitResult) -> Value {
    serde_json::from_str::<Value>(&serde_json::to_string(result.envelope()).unwrap()).unwrap()
        ["index_definition_delta"]
        .clone()
}
fn prepare(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    key: &str,
) -> worth_relational::facade::mvcc::PreparedRelationalCommitCandidate {
    let identity = runtime.branch_identity(branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(batch_create(key), execution.allocation_policy())
        .unwrap();
    let proposal = transaction.validate(execution, runtime).unwrap();
    runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap()
}
fn prepare_update(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    entity: worth_relational::facade::identity::EntityId,
    text: &str,
) -> worth_relational::facade::mvcc::PreparedRelationalCommitCandidate {
    let identity = runtime.branch_identity(branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new(text).push(MutationIntent::Entity(
                worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(
                    worth_relational::facade::transactions::UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields: field_patch("name", "name", text),
                    },
                ),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction.validate(execution, runtime).unwrap();
    runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap()
}
fn publish(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
) -> worth_relational::facade::transactions::CommitResult {
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("real candidate performs")
    };
    runtime.settle_performed_publication(performed).unwrap()
}

#[test]
fn frozen_catalog_overlap_preserves_issued_ids_and_never_regresses_after_reverse_publication() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let seed = create_entity_outcome(execution, &runtime, "seed");
    let RecordRef::Entity(entity) = seed.changed_records[0] else {
        panic!("entity seed")
    };
    release_test_commit_snapshot(&runtime, &seed);
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    runtime
        .fork_branch(execution, BranchId("sibling".into()), source)
        .unwrap();
    let first = runtime
        .index_authority()
        .register(definition("first"))
        .unwrap();
    let older = prepare_update(
        execution,
        &runtime,
        &BranchId("main".into()),
        entity,
        "older",
    );
    let second = runtime
        .index_authority()
        .register(definition("second"))
        .unwrap();
    let newer = prepare_update(
        execution,
        &runtime,
        &BranchId("sibling".into()),
        entity,
        "newer",
    );
    let third = runtime
        .index_authority()
        .register(definition("after-freeze"))
        .unwrap();
    let new_result = publish(execution, &runtime, newer);
    let old_result = publish(execution, &runtime, older);
    assert_eq!(delta(&new_result)["next_index_id"], second.index_id.0 + 1);
    assert_eq!(delta(&old_result)["next_index_id"], first.index_id.0 + 1);
    let next = create_entity_outcome(execution, &runtime, "after-reverse");
    assert_eq!(delta(&next)["base_next_index_id"], second.index_id.0 + 1);
    assert_eq!(delta(&next)["definitions"][0]["index_id"], third.index_id.0);
    assert_eq!(delta(&next)["definitions"].as_array().unwrap().len(), 1);
    for result in [&new_result, &old_result, &next] {
        release_test_commit_snapshot(&runtime, result);
    }
    let bytes = medium
        .offered()
        .iter()
        .map(|artifact| artifact.as_bytes().to_vec())
        .collect::<Vec<_>>();
    drop((old_result, new_result, next, runtime));
    let (recovered, _) = recover(execution, &medium);
    assert_eq!(
        recovered
            .index_access()
            .definition_lookup_snapshot()
            .definition_count(),
        3
    );
    let replayed = recovered
        .durability()
        .canonical_commit_artifacts(&execution)
        .map(|artifact| artifact.unwrap().as_bytes().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(
        replayed, bytes,
        "recovery carries the actual original overlapping payloads"
    );
    let installed = recovered
        .index_authority()
        .register(definition("next-issued"))
        .unwrap();
    assert_eq!(installed.index_id.0, third.index_id.0 + 1);
}

#[test]
fn checkpoint_only_definitions_overlap_tail_and_restored_directory_acknowledges_only_its_capture() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "seed");
    let cp_only = runtime
        .index_authority()
        .register(definition("checkpoint-only"))
        .unwrap();
    let checkpoint = capture(execution, &runtime);
    medium.file_checkpoint(checkpoint.clone());
    let later = runtime
        .index_authority()
        .register(definition("registered-after-capture"))
        .unwrap();
    let overlap = create_entity_outcome(execution, &runtime, "tail-overlap");
    assert_eq!(delta(&overlap)["base_next_index_id"], 1);
    assert_eq!(delta(&overlap)["definitions"].as_array().unwrap().len(), 2);
    release_test_commit_snapshot(&runtime, &overlap);
    let original = medium.offered().last().unwrap().as_bytes().to_vec();
    drop((overlap, runtime));
    let (recovered, _) = recover(execution, &medium);
    let lookup = recovered.index_access().definition_lookup_snapshot();
    assert_eq!(lookup.matching_definition(&cp_only), Some(cp_only));
    assert_eq!(lookup.matching_definition(&later), Some(later.clone()));
    let actual = recovered
        .durability()
        .canonical_commit_artifacts(&execution)
        .last()
        .unwrap()
        .unwrap();
    assert_eq!(actual.as_bytes(), original);
    let unchanged = create_entity_outcome(execution, &recovered, "after-overlap");
    assert!(delta(&unchanged)["definitions"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        delta(&unchanged)["base_next_index_id"],
        later.index_id.0 + 1
    );
    release_test_commit_snapshot(&recovered, &unchanged);
}

#[test]
fn required_catalog_delta_survives_all_projections_and_rejects_missing_or_older_format() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let installed = runtime
        .index_authority()
        .register(definition("required-delta"))
        .unwrap();
    let result = create_entity_outcome(execution, &runtime, "projection");
    let full =
        serde_json::from_str::<Value>(&serde_json::to_string(result.envelope()).unwrap()).unwrap();
    let artifact = medium.offered().pop().unwrap();
    let readmitted = Artifact::readmit(artifact.as_bytes(), execution).unwrap();
    let authoritative =
        serde_json::from_str::<Value>(&serde_json::to_string(readmitted.envelope()).unwrap())
            .unwrap();
    assert_eq!(
        full["index_definition_delta"],
        authoritative["index_definition_delta"]
    );
    assert_eq!(
        full["index_definition_delta"]["definitions"][0]["index_id"],
        installed.index_id.0
    );
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    assert!(checkpoint.bytes().starts_with(b"WORTH-REL-CHK17\n"));
    // The actual embedded checkpoint projection preserves the declaration too.
    let mut restored = memory_runtime();
    restored
        .durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .unwrap();
    assert_eq!(
        restored.index_access().matching_definition(&installed),
        Some(installed)
    );
    for version in [1u16, 2, 3] {
        let mut old = artifact.as_bytes().to_vec();
        old[8..10].copy_from_slice(&version.to_le_bytes());
        assert!(Artifact::readmit(&old, execution).is_err());
    }
    wire::assert_malformed_deltas(execution, &artifact);
    for version in [13, 14, 15, 16] {
        let mut old = checkpoint.bytes().to_vec();
        old[..b"WORTH-REL-CHK17\n".len()]
            .copy_from_slice(format!("WORTH-REL-CHK{version}\n").as_bytes());
        let refused = restored.durability_recovery().restore_native_checkpoint(
            execution,
            &worth_relational::facade::durability::RelationalNativeCheckpoint::from_untrusted_bytes(
                old,
            ),
        );
        assert!(
            matches!(refused, Err(worth_relational::facade::durability::RelationalRecoveryStop::Durability(error))
            if error.class == worth_relational::facade::durability::RecoveryFailureClass::UnsupportedLegacySemantics)
        );
    }
    release_test_commit_snapshot(&runtime, &result);
}
