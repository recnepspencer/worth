use crate::facade::transactions::{
    ApplyEntityAspectPatchIntent, EntityReference, MutationIntent, RelationAspectCreateIntent,
};
use crate::tests::support::*;
use worth_foundational::facade::{
    AspectValue, ContractValidationInput, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch, StructAspectValue,
};

#[test]
fn native_struct_reference_and_clear_state_survive_checkpoint_readmission() {
    let summary_binding = entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    let summary_contract = summary_binding.contract.clone();
    let mut fixture = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    fixture.entity_aspects.push(summary_binding);
    let runtime = fixture.build_runtime();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let summary = StructAspectValue::new([
        (field_key("title"), AspectValue::String("durable".into())),
        (field_key("status"), AspectValue::String("transient".into())),
    ])
    .unwrap();
    let label_contract = runtime
        .relation_aspect_plan(KindId(2))
        .unwrap()
        .contract_for(&aspect_key("label"))
        .unwrap();

    let mut initial = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    initial
        .push_batch(
            WorkerIntentBatch::new("native-entity-struct").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: source,
                    aspect_patch: whole_struct_set(&summary_contract, summary),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    initial
        .push_batch(
            WorkerIntentBatch::new("native-relation-reference").push(MutationIntent::Create(
                CreateIntent::RelationAspects(RelationAspectCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: crate::symbols::data::ClientKey::raw("native-durable-relation"),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                    aspect_patch: whole_scalar_set(&label_contract, "durable-edge"),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let initial = initial
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let relation = changed_relations(&initial)[0];

    let status = field_key("status");
    let clear = PortableRecordAspectPatch::new([PortableAspectPatchOperation::PatchFields {
        basis: PortableAspectContractBasis::from_contract(&summary_contract),
        selected_fields: vec![status.clone()],
        field_sets: Vec::new(),
        field_clears: vec![status],
    }]);
    let mut clear_transaction =
        crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    clear_transaction
        .push_batch(
            WorkerIntentBatch::new("native-field-clear").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: source,
                    aspect_patch: clear,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let cleared = clear_transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let (expected_entity_state, expected_relation_state) = {
        let read = runtime
            .read_truth()
            .read_snapshot(&cleared.snapshot)
            .unwrap();
        (
            read.get_entity(source)
                .unwrap()
                .authoritative_aspect_state
                .clone(),
            read.get_relation(relation)
                .unwrap()
                .authoritative_aspect_state
                .clone(),
        )
    };
    let recovery_fixture = fixture.clone();
    let (_, recovered) =
        checkpoint_and_recover_with(&runtime, move || recovery_fixture.build_runtime());
    let read = recovered.read_truth().read_version(cleared.version_id);

    assert_eq!(
        read.get_entity(source).unwrap().authoritative_aspect_state,
        expected_entity_state
    );
    assert_eq!(
        read.get_relation(relation)
            .unwrap()
            .authoritative_aspect_state,
        expected_relation_state
    );
    let relation_state = read
        .get_relation(relation)
        .unwrap()
        .authoritative_aspect_state
        .as_ref()
        .unwrap();
    assert!(relation_state.get(&aspect_key("source")).is_some());
    assert!(relation_state.get(&aspect_key("target")).is_some());
}

fn whole_struct_set(
    contract: &worth_foundational::facade::AspectContract,
    value: StructAspectValue,
) -> PortableRecordAspectPatch {
    PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
        value: ContractValidationInput::Struct(value),
    }])
}

fn whole_scalar_set(
    contract: &worth_foundational::facade::AspectContract,
    value: &str,
) -> PortableRecordAspectPatch {
    PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
        value: ContractValidationInput::Scalar(AspectValue::String(value.into())),
    }])
}
use crate::capabilities::AspectPlanSource;
