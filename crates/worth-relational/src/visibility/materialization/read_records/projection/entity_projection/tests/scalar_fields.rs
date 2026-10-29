//! A selected scalar field is interpreted only by its genuine installed binding.
use super::*;

#[test]
fn scalar_projection_uses_the_bound_field_and_refuses_aliases_and_stale_generations() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let binding = entity_field_aspect(aspect_key("name"), field_key("label"));
    let contract = binding.contract.clone();
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![binding],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("scalar-projection").push(MutationIntent::Create(
                CreateIntent::Entity(crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("scalar-projection"),
                    fields: string_aspect_field_patch([(
                        aspect_key("name"),
                        field_key("label"),
                        "actual",
                    )]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let committed = transaction.commit(execution, &runtime).unwrap();
    let entity = changed_entities(&committed)[0];
    let view = runtime
        .read_truth()
        .project_snapshot(&committed.snapshot)
        .unwrap();
    let scope = ProjectionAspectScope::fields(aspect_key("name"), [field_key("label")]);
    let actual = view
        .entity_record_with_projection_scope(execution, entity, scope.clone(), |record| {
            assert!(record
                .observe_aspect_field(&aspect_key("name"), &field_key("name"))
                .is_none());
            record
                .observe_aspect_field(&aspect_key("name"), &field_key("label"))
                .and_then(|observed| observed.value().cloned())
        })
        .unwrap();
    assert_eq!(actual, Some(string_aspect_value("actual")));
    let mut touched = false;
    let denied = view.entity_record_with_projection_scope(
        execution,
        entity,
        ProjectionAspectScope::fields(aspect_key("name"), [field_key("name")]),
        |_| {
            touched = true;
            Some(())
        },
    );
    let Err(crate::execution::RelationalExecutionDenialCause::ProjectionContract(denial)) = denied
    else {
        panic!("a wrong physical scalar field must refuse projection admission");
    };
    assert_eq!(denial.refusal,
        crate::visibility::materialization::read_records::projection::contracts::ProjectionContractRefusal::MaskRejected);
    assert!(
        !touched,
        "an aspect-name alias never reaches the scalar callback"
    );
    let stale = EntityId::new(
        entity.partition_id,
        entity.local_slot_value(),
        entity.generation_value() + 1,
    );
    assert_eq!(
        view.entity_record_with_projection_scope(execution, stale, scope, |_| Some(()))
            .unwrap(),
        None
    );
    for value in ["changed", "actual"] {
        use worth_foundational::facade::{
            CanonicalFieldPath, ContractValidationInput, PortableAspectContractBasis,
            PortableAspectPatchOperation, PortableRecordAspectPatch,
        };
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new("scalar-whole-touch").push(MutationIntent::Entity(
                    crate::transactions::data::EntityMutationIntent::ApplyAspectPatch(
                        crate::transactions::data::ApplyEntityAspectPatchIntent {
                            entity_id: entity,
                            aspect_patch: PortableRecordAspectPatch::new([
                                PortableAspectPatchOperation::SetWhole {
                                    basis: PortableAspectContractBasis::from_contract(&contract),
                                    value: ContractValidationInput::Scalar(string_aspect_value(
                                        value,
                                    )),
                                },
                            ]),
                        },
                    ),
                )),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let validated = transaction.validate(execution, &runtime).unwrap();
        let touches = validated.mutation_touches(execution).unwrap();
        assert!(touches.struct_touches().is_empty());
        assert_eq!(
            touches.touches(),
            &[crate::mvcc::ValidatedMutationTouch::WriteEntityField {
                kind: KindId(1),
                locator: crate::transactions::data::planned_aspect_field_locator(
                    aspect_key("name"),
                    CanonicalFieldPath::single(field_key("label")),
                ),
            }],
            "whole scalar sets, including equal bytes, touch only the actual bound field"
        );
    }
}
