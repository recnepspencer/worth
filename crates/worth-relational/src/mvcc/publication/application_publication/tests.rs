use crate::identity::data::{KindId, PartitionId};
use crate::tests::support::*;
use crate::transactions::data::{
    CreatedEntityRef, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};
use worth_foundational::facade::AspectBinding;

#[test]
fn native_performed_origin_is_exact_and_an_ordinary_equal_write_revokes_retirement() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let mut runtime = runtime_with_test_schema();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let target = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("output"),
    };
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("output"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let proposal = runtime
        .validate_branch_transaction(execution, transaction)
        .unwrap();
    let proposal = proposal
        .bind_required_output_publication(
            &installed,
            owner.admit_required_output("installed-producer", [3; 32]),
            vec![target.clone()],
        )
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("genuine native output publication must perform");
    };
    let origin = performed
        .canonical_commit()
        .required_output_origin
        .as_ref()
        .unwrap();
    assert_eq!(origin.producer, "installed-producer");
    assert_eq!(origin.dependency_identity, [3; 32]);
    let entity = origin.outputs[0];
    let mut copied = performed.canonical_commit().clone();
    copied.commit.commit_id.0 += 1;
    let partition = performed
        .next_basis()
        .inner
        .root
        .partition_state(entity.partition_id)
        .unwrap();
    let record = partition.entity_arena.get(&entity).unwrap();
    let payload = record.extra().authoritative_aspect_state.as_ref().unwrap();
    let copied_refusal = execution
        .in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    copied.readmit_required_output_payload(entity, payload, context)
                })
        })
        .unwrap()
        .unwrap()
        .0;
    assert!(
        copied_refusal.is_err(),
        "copied descriptive producer origin must not authenticate another native commit"
    );
    let result = runtime.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(&runtime, &result);
    let loci = [crate::runtime::CodecFieldLocus {
        kind_id: KindId(1),
        aspect_key: aspect_key("name"),
        field_key: field_key("name"),
        binding: AspectBinding::EntityField {
            field: field_key("name"),
        },
    }];
    let basis = test_owner_main_basis(&runtime).unwrap();
    assert_eq!(
        crate::history::data::derive_codec_retirement_closure(execution, &runtime, &basis, &loci)
            .unwrap()
            .entities,
        vec![entity]
    );
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("ordinary-equal").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: name_field_patch("output"),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let result = transaction.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &result);
    let basis = test_owner_main_basis(&runtime).unwrap();
    assert!(
        crate::history::data::derive_codec_retirement_closure(execution, &runtime, &basis, &loci)
            .is_err(),
        "equal supplied bytes must not preserve the prior producer's migration permission"
    );
}

#[test]
fn foreign_owner_and_unclassified_same_kind_rows_cannot_borrow_output_origin() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let mut runtime = runtime_with_test_schema();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (_, installed) = receipt.issue_application_publication_owner();
    let foreign = worth_proof::InstalledApplicationPublicationOwner::fresh();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("authored"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let proposal = runtime
        .validate_branch_transaction(execution, transaction)
        .unwrap();
    assert!(proposal
        .bind_required_output_publication(
            &installed,
            foreign.admit_required_output("installed-producer", [3; 32]),
            Vec::new()
        )
        .is_err());
    create_entity(execution, &runtime, "authored");
    let basis = test_owner_main_basis(&runtime).unwrap();
    let loci = [crate::runtime::CodecFieldLocus {
        kind_id: KindId(1),
        aspect_key: aspect_key("name"),
        field_key: field_key("name"),
        binding: AspectBinding::EntityField {
            field: field_key("name"),
        },
    }];
    assert!(crate::history::data::derive_codec_retirement_closure(
        execution, &runtime, &basis, &loci
    )
    .is_err());
}

#[test]
fn producer_publication_preserves_unclassified_rows_without_granting_retirement() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let mut runtime = runtime_with_test_schema();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let target = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("selected-output"),
    };
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    for name in ["selected-output", "system-recorded"] {
        transaction
            .push_batch(
                batch_create(name),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
    }
    let proposal = runtime
        .validate_branch_transaction(execution, transaction)
        .unwrap()
        .bind_required_output_publication(
            &installed,
            owner.admit_required_output("producer", [8; 32]),
            vec![target],
        )
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("ordinary producer publication must preserve its unclassified system rows");
    };
    assert_eq!(
        performed
            .canonical_commit()
            .required_output_origin
            .as_ref()
            .unwrap()
            .outputs
            .len(),
        1
    );
    let committed = runtime.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    let basis = test_owner_main_basis(&runtime).unwrap();
    let loci = [crate::runtime::CodecFieldLocus {
        kind_id: KindId(1),
        aspect_key: aspect_key("name"),
        field_key: field_key("name"),
        binding: AspectBinding::EntityField {
            field: field_key("name"),
        },
    }];
    assert!(crate::history::data::derive_codec_retirement_closure(execution,&runtime,&basis,&loci).is_err(),
        "one selected output must never grant kind-wide permission over its system-recorded sibling");
}
