use crate::facade::transactions::{
    ApplyEntityAspectPatchIntent, ApplyRelationAspectPatchIntent, CommitConflict, ConflictClass,
    EntityAspectCreateIntent, MutationIntent, TransactionCommitError,
};
use crate::tests::support::*;
use worth_foundational::facade::{
    AspectValue, ContractValidatedAspectValueView, ContractValidationInput,
    PortableAspectContractBasis, PortableAspectPatchOperation, PortableRecordAspectPatch,
};

#[test]
fn native_same_record_updates_conflict_before_truth_or_publication_changes() {
    for (clear_second, reverse_order) in
        [(false, false), (false, true), (true, false), (true, true)]
    {
        let runtime =
            runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
        let entity = create_entity(&runtime, "before");
        let contract = entity_name_contract(&runtime);
        let before_snapshot = runtime.visibility_authority().snapshot();
        let before_patch = runtime.publication().latest_bundle().unwrap().patch.clone();
        let first = entity_patch_intent(entity, whole_set(&contract, "first"));
        let second_patch = if clear_second {
            whole_clear(&contract)
        } else {
            whole_set(&contract, "second")
        };
        let second = entity_patch_intent(entity, second_patch);

        let mut transaction =
            crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        let batches = if reverse_order {
            [("second", second), ("first", first)]
        } else {
            [("first", first), ("second", second)]
        };
        for (label, intent) in batches {
            transaction
                .push_batch(
                    WorkerIntentBatch::new(label).push(intent),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("test staging stays within configured resource budgets");
        }

        let error = transaction
            .commit(
                &runtime,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect_err("same-record updates must conflict");
        assert!(matches!(
            error,
            TransactionCommitError::Conflict {
                error: CommitConflict {
                    class: ConflictClass::ConflictingIntent { .. },
                    ..
                },
                ..
            }
        ));
        let after_snapshot = runtime.visibility_authority().snapshot();
        let after_patch = runtime.publication().latest_bundle().unwrap().patch.clone();
        assert_eq!(after_snapshot.version_id, before_snapshot.version_id);
        assert_eq!(after_patch, before_patch);

        let read = runtime
            .read_truth()
            .read_snapshot(&after_snapshot)
            .expect("unchanged snapshot");
        let value = read
            .get_entity(entity)
            .unwrap()
            .authoritative_aspect_state
            .as_ref()
            .unwrap()
            .get(contract.key())
            .unwrap();
        assert!(matches!(
            value.view(),
            ContractValidatedAspectValueView::Scalar(AspectValue::String(value))
                if value == &"before".into()
        ));
        drop(read);
        runtime
            .snapshots()
            .release_snapshot(&before_snapshot)
            .unwrap();
        runtime
            .snapshots()
            .release_snapshot(&after_snapshot)
            .unwrap();
    }
}

#[test]
fn native_create_merge_is_stable_across_batch_permutations() {
    let runtime_a =
        runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let contract_a = entity_name_contract(&runtime_a);
    let mut transaction_a = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime_a);
        runtime_a
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction_a
        .push_batch(
            native_create_batch("zeta", &contract_a),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction_a
        .push_batch(
            native_create_batch("alpha", &contract_a),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let intents_a = transaction_a
        .merged_plan(
            &runtime_a,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .merged_intents
        .clone();

    let runtime_b =
        runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let contract_b = entity_name_contract(&runtime_b);
    let mut transaction_b = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime_b);
        runtime_b
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction_b
        .push_batch(
            native_create_batch("alpha", &contract_b),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction_b
        .push_batch(
            native_create_batch("zeta", &contract_b),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let intents_b = transaction_b
        .merged_plan(
            &runtime_b,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .merged_intents
        .clone();

    assert_eq!(intents_a, intents_b);
    assert_eq!(intents_a.len(), 2);
    for intent in intents_a {
        let MutationIntent::Create(CreateIntent::EntityAspects(intent)) = intent else {
            panic!("merged plan replaced native create authority");
        };
        assert_eq!(intent.aspect_patch.operations().len(), 1);
    }
}

#[test]
fn compatibility_and_native_scalar_authoring_publish_identical_patch_meaning() {
    let compatibility_runtime =
        runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let compatibility_entity = create_entity(&compatibility_runtime, "before");
    let compatibility = update_entity(&compatibility_runtime, compatibility_entity, "after");

    let native_runtime =
        runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let native_entity = create_entity(&native_runtime, "before");
    let contract = entity_name_contract(&native_runtime);
    let mut transaction =
        crate::tests::support::test_owner_begin_transaction_for_main(&native_runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("native-equivalent").push(entity_patch_intent(
                native_entity,
                whole_set(&contract, "after"),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let native = transaction
        .commit(
            &native_runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    assert_eq!(
        compatibility.patch()[0].authoritative_patch,
        native.patch()[0].authoritative_patch
    );
    assert_eq!(
        compatibility.patch()[0].authoritative_changed_aspects(),
        native.patch()[0].authoritative_changed_aspects()
    );
}

#[test]
fn compatibility_and_native_updates_on_one_target_have_one_conflict_law() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let entity = create_entity(&runtime, "before");
    let contract = entity_name_contract(&runtime);
    let before_snapshot = runtime
        .publication()
        .latest_bundle()
        .unwrap()
        .snapshot
        .clone();
    let compatibility = MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id: entity,
            fields: name_field_patch("compatibility"),
        },
    ));
    let native = entity_patch_intent(entity, whole_set(&contract, "native"));
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("compatibility").push(compatibility),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .push_batch(
            WorkerIntentBatch::new("native").push(native),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");

    assert!(matches!(
        transaction.commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        ),
        Err(TransactionCommitError::Conflict {
            error: CommitConflict {
                class: ConflictClass::ConflictingIntent { .. },
                ..
            },
            ..
        })
    ));
    assert_eq!(
        runtime.publication().latest_bundle().unwrap().snapshot,
        before_snapshot
    );
}

#[test]
fn mixed_native_entity_and_relation_updates_share_one_atomic_commit() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let relation = create_relation(&runtime, source, target, "before");
    let entity_contract = entity_name_contract(&runtime);
    let relation_contract = runtime
        .relation_aspect_plan(KindId(2))
        .unwrap()
        .contract_for(&aspect_key("label"))
        .unwrap();
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("entity").push(entity_patch_intent(
                source,
                whole_set(&entity_contract, "source-after"),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .push_batch(
            WorkerIntentBatch::new("relation").push(MutationIntent::Relation(
                RelationMutationIntent::ApplyAspectPatch(ApplyRelationAspectPatchIntent {
                    relation_id: relation,
                    aspect_patch: whole_set(&relation_contract, "relation-after"),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");

    let committed = transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert_eq!(committed.patch().len(), 2);
    assert!(committed.patch().iter().any(
        |record: &crate::publication::patch::data::PublishedAuthoritativeRecordPatch| {
            record.authoritative_changed_aspects() == vec![aspect_key("name")]
        }
    ));
    assert!(committed.patch().iter().any(
        |record: &crate::publication::patch::data::PublishedAuthoritativeRecordPatch| {
            record.authoritative_changed_aspects() == vec![aspect_key("label")]
        }
    ));
}

fn native_create_batch(
    client_key: &str,
    contract: &worth_foundational::facade::AspectContract,
) -> WorkerIntentBatch {
    WorkerIntentBatch::new(client_key).push(MutationIntent::Create(CreateIntent::EntityAspects(
        EntityAspectCreateIntent {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: crate::symbols::data::ClientKey::raw(client_key),
            aspect_patch: whole_set(contract, client_key),
        },
    )))
}

fn entity_patch_intent(
    entity_id: crate::facade::identity::EntityId,
    aspect_patch: PortableRecordAspectPatch,
) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
        ApplyEntityAspectPatchIntent {
            entity_id,
            aspect_patch,
        },
    ))
}

fn entity_name_contract(runtime: &RelationalRuntime) -> worth_foundational::facade::AspectContract {
    runtime
        .entity_aspect_plan(KindId(1))
        .unwrap()
        .contract_for(&aspect_key("name"))
        .unwrap()
}

fn whole_set(
    contract: &worth_foundational::facade::AspectContract,
    value: &str,
) -> PortableRecordAspectPatch {
    PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
        value: ContractValidationInput::Scalar(AspectValue::String(value.into())),
    }])
}

fn whole_clear(contract: &worth_foundational::facade::AspectContract) -> PortableRecordAspectPatch {
    PortableRecordAspectPatch::new([PortableAspectPatchOperation::ClearWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
    }])
}
use crate::capabilities::AspectPlanSource;
