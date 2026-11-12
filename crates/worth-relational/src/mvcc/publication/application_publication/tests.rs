use crate::identity::data::{KindId, PartitionId};
use crate::tests::support::*;
use crate::transactions::data::{
    CreatedEntityRef, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};
use worth_foundational::facade::AspectBinding;

#[test]
fn native_performed_origin_is_exact_and_an_ordinary_equal_write_revokes_retirement() {
    let producer_budget = worth_execution::SerialMemoryBudget::new(Some(4 * 1024 * 1024));
    let producer_cancel = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(
        producer_budget.clone(),
        producer_cancel.token(),
        None,
    );
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
            crate::mvcc::native_birth_fixture(
                owner.admit_required_output("installed-producer", [3; 32]),
                vec![target.clone()],
            ),
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
    assert_eq!(*origin.dependency_identity.bytes(), [3; 32]);
    let entity = origin.outputs[0];
    let mut copied = performed
        .canonical_commit()
        .copy_original_for_test(execution);
    copied.body_mut_for_test().commit.commit_id.0 += 1;
    let copied_refusal = execution
        .run_native_with_checkpoint(|context| {
            copied
                .validate_output_origins(context)
                .and_then(|view| view.validate_output_membership_header())
        })
        .unwrap()
        .0;
    assert!(
        copied_refusal.is_err(),
        "copied descriptive producer origin must not authenticate another native commit"
    );
    let result = runtime.settle_performed_publication(performed).unwrap();
    let public_copy = runtime
        .read_truth()
        .project_snapshot(&result.snapshot)
        .unwrap()
        .all_authoritative_entity_records(execution)
        .unwrap()
        .into_iter()
        .find(|record| record.entity_id == entity)
        .unwrap();
    release_test_commit_snapshot(&runtime, &result);
    assert!(producer_budget.charged_memory_bytes() > 0, "the actual published row retains its original application bill after result snapshot release");
    let loci = [crate::runtime::CodecFieldLocus {
        kind_id: KindId(1),
        aspect_key: aspect_key("name"),
        field_key: field_key("name"),
        binding: AspectBinding::EntityField {
            field: field_key("name"),
        },
    }];
    // The genuine producer above uses its own finite original request. Actual
    // cold derive/readmission/application below uses this finite original request.
    let budget = worth_execution::SerialMemoryBudget::new(Some(1024 * 1024));
    let cancellation = worth_execution::CancellationSource::new();
    let bounded =
        worth_execution::SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let bounded_request = worth_execution::ExecutionRequest::serial(&bounded);
    let basis = test_owner_main_basis(&runtime).unwrap();
    assert_eq!(
        crate::history::data::derive_codec_retirement_closure(
            &bounded_request,
            &runtime,
            &basis,
            &loci
        )
        .unwrap()
        .entities,
        vec![entity]
    );
    let original_basis = basis;
    let original_partition = original_basis
        .inner
        .root
        .partition_state(entity.partition_id)
        .unwrap()
        .unwrap();
    let original_payload = rmp_serde::to_vec(
        &original_partition
            .entity_arena
            .get(&entity)
            .unwrap()
            .extra()
            .authoritative_aspect_state(),
    )
    .unwrap();
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
        crate::history::data::derive_codec_retirement_closure(
            &bounded_request,
            &runtime,
            &basis,
            &loci
        )
        .is_err(),
        "equal supplied bytes must not preserve the prior producer's migration permission"
    );
    let changed = update_entity(execution, &runtime, entity, "different");
    release_test_commit_snapshot(&runtime, &changed);
    let current_basis = test_owner_main_basis(&runtime).unwrap();
    let current_partition = current_basis
        .inner
        .root
        .partition_state(entity.partition_id)
        .unwrap()
        .unwrap();
    let old_record = original_partition.entity_arena.get(&entity).unwrap();
    let new_record = current_partition.entity_arena.get(&entity).unwrap();
    assert_eq!(
        rmp_serde::to_vec(&old_record.extra().authoritative_aspect_state()).unwrap(),
        original_payload
    );
    assert_ne!(
        rmp_serde::to_vec(&new_record.extra().authoritative_aspect_state()).unwrap(),
        original_payload
    );
    assert_ne!(
        old_record.retained_extra().id(),
        new_record.retained_extra().id()
    );
    assert_eq!(read_entity_name(&public_copy).as_deref(), Some("output"));
    let latest = runtime
        .read_truth()
        .read_version(execution, changed.version_id)
        .unwrap();
    assert_eq!(
        latest
            .get_entity(entity)
            .and_then(read_entity_name)
            .as_deref(),
        Some("different")
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
            crate::mvcc::native_birth_fixture(
                foreign.admit_required_output("installed-producer", [3; 32]),
                Vec::new()
            )
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
            crate::mvcc::native_birth_fixture(
                owner.admit_required_output("producer", [8; 32]),
                vec![target],
            ),
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
