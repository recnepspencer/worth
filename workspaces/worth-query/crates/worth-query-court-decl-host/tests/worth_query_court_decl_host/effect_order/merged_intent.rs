//! Native readmission checks compound intent material without trusting its producer.
use super::*;
use worth_execution::ExecutionAllocationPolicy;

fn merged(id: EntityId) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
        ApplyEntityAspectPatchIntent {
            entity_id: id,
            aspect_patch: PortableRecordAspectPatch::new([
                PortableAspectPatchOperation::ClearWhole {
                    basis: schema::basis("Alias"),
                },
                PortableAspectPatchOperation::PatchFields {
                    basis: schema::basis("Marker"),
                    selected_fields: vec![FieldKey::new("Count").unwrap()],
                    field_sets: vec![PortableAspectFieldSet::new(
                        FieldKey::new("Count").unwrap(),
                        AspectValue::UInt64(23),
                    )],
                    field_clears: vec![],
                },
            ]),
        },
    ))
}

#[test]
fn tampered_merged_intent_refuses_readmission_without_applying_its_valid_removal() {
    let mut runtime = schema::runtime();
    let request = world::request();
    let execution = ExecutionRequest::serial(&request);
    let seed = world::commit(
        &mut runtime,
        &execution,
        vec![world::create("a", "token", 7)],
    )
    .unwrap();
    let id = seed.created_entity(&world::created("a")).unwrap();
    let valid = merged(id);
    let mut corrupted = valid.clone();
    let MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(patch)) = &mut corrupted
    else {
        unreachable!();
    };
    let mut operations = patch.aspect_patch.operations().to_vec();
    let PortableAspectPatchOperation::PatchFields { basis, .. } = &mut operations[1] else {
        unreachable!();
    };
    *basis = PortableAspectContractBasis::new(
        basis.key().clone(),
        basis.identity(),
        AspectContractRevision(2),
    );
    patch.aspect_patch = PortableRecordAspectPatch::new(operations);
    let denial = world::commit(&mut runtime, &execution, vec![corrupted]).unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = denial else {
        panic!("{denial:?}");
    };
    let ConflictClass::RecordAspectPatchDenied { target, denial } = error.class else {
        panic!("native readmission must refuse: {error:?}");
    };
    // The public ConflictClass carries serializable target/cause values whose
    // concrete nested types are not re-exported by the transaction facade.
    assert_eq!(
        serde_json::to_value(target).unwrap(),
        serde_json::json!({
            "Entity": { "entity_id": id, "kind_id": schema::KIND }
        })
    );
    assert_eq!(
        serde_json::to_value(denial).unwrap(),
        serde_json::json!({
            "ReadmissionDenied": PortableAspectReadmissionDenial::ContractRevisionMismatch {
                key: AspectKey::new("Marker").unwrap(),
                expected: AspectContractRevision(1), found: AspectContractRevision(2),
            }
        })
    );
    let current = world::unchanged_head(&runtime, &seed);
    assert_eq!(
        world::field(&runtime, &execution, &current, id, "Alias", "Unique"),
        Some(AspectValue::String("token".into()))
    );
    assert_eq!(
        world::field(&runtime, &execution, &current, id, "Marker", "Count"),
        Some(AspectValue::UInt64(7))
    );
    runtime.snapshots().release_snapshot(&current).unwrap();

    // The unchanged counterpart reaches the same native boundary and commits.
    let result = world::commit(&mut runtime, &execution, vec![valid]).unwrap();
    assert_eq!(
        world::field(
            &runtime,
            &execution,
            &result.snapshot,
            id,
            "Alias",
            "Unique"
        ),
        None
    );
    assert_eq!(
        world::field(
            &runtime,
            &execution,
            &result.snapshot,
            id,
            "Marker",
            "Count"
        ),
        Some(AspectValue::UInt64(23))
    );
    runtime
        .snapshots()
        .release_snapshot(&result.snapshot)
        .unwrap();
    runtime
        .snapshots()
        .release_snapshot(&seed.snapshot)
        .unwrap();
}

#[test]
fn disjoint_removal_and_write_from_independent_workers_still_conflict() {
    for removal_first in [false, true] {
        let mut runtime = schema::runtime();
        let request = world::request();
        let execution = ExecutionRequest::serial(&request);
        let seed = world::commit(
            &mut runtime,
            &execution,
            vec![world::create("a", "token", 7)],
        )
        .unwrap();
        let id = seed.created_entity(&world::created("a")).unwrap();
        let removal = WorkerIntentBatch::new("remove-producer").push(world::clear(id, &["Alias"]));
        let write = WorkerIntentBatch::new("write-producer").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: id,
                fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                    schema::locator("Marker", "Count"),
                    AspectValue::UInt64(23),
                )])),
            }),
        ));
        let basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap();
        let mut transaction = runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        let batches = if removal_first {
            [removal, write]
        } else {
            [write, removal]
        };
        for batch in batches {
            transaction
                .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
        }
        let denial = transaction.commit(&execution, &runtime).unwrap_err();
        let TransactionCommitError::Conflict { error, .. } = denial else {
            panic!("{denial:?}");
        };
        let ConflictClass::ConflictingIntent { target } = error.class else {
            panic!("independent intents must conflict: {error:?}");
        };
        assert_eq!(
            serde_json::to_value(target).unwrap(),
            serde_json::json!({ "Entity": id })
        );
        let current = world::unchanged_head(&runtime, &seed);
        assert_eq!(
            world::field(&runtime, &execution, &current, id, "Alias", "Unique"),
            Some(AspectValue::String("token".into()))
        );
        assert_eq!(
            world::field(&runtime, &execution, &current, id, "Marker", "Count"),
            Some(AspectValue::UInt64(7))
        );
        runtime.snapshots().release_snapshot(&current).unwrap();
        runtime
            .snapshots()
            .release_snapshot(&seed.snapshot)
            .unwrap();
    }
}
