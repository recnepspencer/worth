use crate::capabilities::AspectPlanSource;
use crate::facade::identity::EntityId;
use crate::facade::transactions::{
    ApplyEntityAspectPatchIntent, ApplyRelationAspectPatchIntent, CommitConflict, ConflictClass,
    EntityAspectCreateIntent, EntityReference, MutationIntent, RelationAspectCreateIntent,
    TransactionCommitError,
};
use crate::tests::support::*;
use worth_foundational::facade::{
    aspects, AbsenceLaw, AspectEquivalenceBasis, AspectEvolutionPolicy, AspectIdentity,
    AspectMaskContract, AspectValue, ContractValidatedAspectValueView, ContractValidationInput,
    PortableAspectContractBasis, PortableAspectPatchOperation, PortableRecordAspectPatch,
    ScalarAspectType, StructAspectValue,
};

#[test]
fn native_entity_patch_supports_optional_whole_clear() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let note = optional_string_contract("note");
    let fixture = AspectSchemaFixture {
        entity_aspects: vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: field_key("note"),
            },
            contract: note.clone(),
        }],
        ..AspectSchemaFixture::default()
    };
    let runtime = fixture.build_runtime();
    let created = commit_entity_create(execution, &runtime, whole_set(&note, text("remember")));
    let entity = changed_entities(&created)[0];

    let cleared = commit_entity_patch(execution, &runtime, entity, whole_clear(&note));
    let read = runtime
        .read_truth()
        .read_snapshot(&cleared.snapshot, execution)
        .expect("read request remains live")
        .expect("cleared snapshot");
    let state = read
        .get_entity(entity)
        .expect("entity")
        .authoritative_aspect_state
        .as_ref();
    assert!(state.is_none_or(|state| state.get(note.key()).is_none()));
    assert_eq!(
        cleared.patch()[0].authoritative_changed_aspects(),
        vec![note.key().clone()]
    );
}

#[test]
fn native_struct_field_clear_uses_exact_contract_basis() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let binding = entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    let contract = binding.contract.clone();
    let fixture = AspectSchemaFixture {
        entity_aspects: vec![binding],
        ..AspectSchemaFixture::default()
    };
    let runtime = fixture.build_runtime();
    let value = StructAspectValue::new([
        (field_key("title"), text("before")),
        (field_key("status"), text("open")),
    ])
    .expect("struct value");
    let created = commit_entity_create(
        execution,
        &runtime,
        PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&contract),
            value: ContractValidationInput::Struct(value),
        }]),
    );
    let entity = changed_entities(&created)[0];
    let status = field_key("status");
    let updated = commit_entity_patch(
        execution,
        &runtime,
        entity,
        PortableRecordAspectPatch::new([PortableAspectPatchOperation::PatchFields {
            basis: PortableAspectContractBasis::from_contract(&contract),
            selected_fields: vec![status.clone()],
            field_sets: Vec::new(),
            field_clears: vec![status.clone()],
        }]),
    );
    let read = runtime
        .read_truth()
        .read_snapshot(&updated.snapshot, execution)
        .expect("read request remains live")
        .unwrap();
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
        ContractValidatedAspectValueView::Struct(value) if value.get(&status).is_none()
    ));
}

#[test]
fn native_relation_creation_synthesizes_endpoint_aspects_and_publishes_updates() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let fixture = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let runtime = fixture.build_runtime();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let label = contract_for_relation(&runtime, "label");
    let created = commit_relation_create(
        execution,
        &runtime,
        source,
        target,
        whole_set(&label, text("before")),
    );
    let relation = changed_relations(&created)[0];
    assert_eq!(
        created.patch()[0].authoritative_changed_aspects(),
        ordered_aspect_keys([
            aspect_key("label"),
            aspect_key("lifecycle"),
            aspect_key("source"),
            aspect_key("target"),
        ])
    );
    let read = runtime
        .read_truth()
        .read_snapshot(&created.snapshot, execution)
        .expect("read request remains live")
        .unwrap();
    let state = read
        .get_relation(relation)
        .unwrap()
        .authoritative_aspect_state
        .as_ref()
        .unwrap();
    assert!(state.get(&aspect_key("source")).is_some());
    assert!(state.get(&aspect_key("target")).is_some());

    let updated = commit_relation_patch(
        execution,
        &runtime,
        relation,
        whole_set(&label, text("after")),
    );
    assert_eq!(
        updated.patch()[0].authoritative_changed_aspects(),
        vec![aspect_key("label")]
    );
}

fn commit_entity_create(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    aspect_patch: PortableRecordAspectPatch,
) -> CommitResult {
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("native-entity-create").push(MutationIntent::Create(
                CreateIntent::EntityAspects(EntityAspectCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("native-entity"),
                    aspect_patch,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction.commit(execution, runtime).unwrap()
}

fn commit_entity_patch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: EntityId,
    aspect_patch: PortableRecordAspectPatch,
) -> CommitResult {
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("native-entity-patch").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id,
                    aspect_patch,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction.commit(execution, runtime).unwrap()
}

fn commit_relation_create(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    aspect_patch: PortableRecordAspectPatch,
) -> CommitResult {
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("native-relation-create").push(MutationIntent::Create(
                CreateIntent::RelationAspects(RelationAspectCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: crate::symbols::data::ClientKey::raw("native-relation"),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                    aspect_patch,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction.commit(execution, runtime).unwrap()
}

fn commit_relation_patch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    relation_id: RelationId,
    aspect_patch: PortableRecordAspectPatch,
) -> CommitResult {
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("native-relation-patch").push(MutationIntent::Relation(
                RelationMutationIntent::ApplyAspectPatch(ApplyRelationAspectPatchIntent {
                    relation_id,
                    aspect_patch,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction.commit(execution, runtime).unwrap()
}

fn whole_set(
    contract: &worth_foundational::facade::AspectContract,
    value: AspectValue,
) -> PortableRecordAspectPatch {
    PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
        value: ContractValidationInput::Scalar(value),
    }])
}

fn whole_clear(contract: &worth_foundational::facade::AspectContract) -> PortableRecordAspectPatch {
    PortableRecordAspectPatch::new([PortableAspectPatchOperation::ClearWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
    }])
}

fn optional_string_contract(name: &str) -> worth_foundational::facade::AspectContract {
    let key = aspect_key(name);
    aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(AspectIdentity(991))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar_with(
            ScalarAspectType::String,
            AspectMaskContract::scalar(),
            AbsenceLaw::Optional,
            AspectEquivalenceBasis::ExactCanonicalValue,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
}

fn contract_for_entity(
    runtime: &RelationalRuntime,
    name: &str,
) -> worth_foundational::facade::AspectContract {
    runtime
        .entity_aspect_plan(KindId(1))
        .unwrap()
        .contract_for(&aspect_key(name))
        .unwrap()
}

fn contract_for_relation(
    runtime: &RelationalRuntime,
    name: &str,
) -> worth_foundational::facade::AspectContract {
    runtime
        .relation_aspect_plan(KindId(2))
        .unwrap()
        .contract_for(&aspect_key(name))
        .unwrap()
}

fn text(value: &str) -> AspectValue {
    AspectValue::String(value.into())
}

#[path = "native_aspect_mutations/checkpoint_readmission.rs"]
mod checkpoint_readmission;

#[path = "native_aspect_mutations/stale_contract.rs"]
mod stale_contract;
