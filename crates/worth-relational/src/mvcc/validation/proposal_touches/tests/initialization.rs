//! The sealed native basis distinguishes initialization from replacement.
use super::*;
use crate::config::data::CascadeDeletePolicy;
use crate::schema::data::DeclaredAspectContractBinding;
use crate::tests::support::{aspect_key, field_key, AspectSchemaFixture};
use crate::transactions::data::{ApplyEntityAspectPatchIntent, CommitResult};
use worth_foundational::facade::{
    aspects, AspectBinding, AspectContract, AspectIdentity, AspectValue, ContractValidationInput,
    PortableAspectContractBasis, PortableAspectPatchOperation, PortableRecordAspectPatch,
    ScalarAspectType, StructAspectValue,
};

#[test]
fn optional_aspect_initialization_and_present_empty_replacement_have_exact_presence() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (runtime, contract) = world();
    let born = crate::tests::support::create_entity_outcome(&execution, &runtime, "source");
    let entity = crate::tests::support::changed_entities(&born)[0];
    {
        let read = runtime
            .read_truth()
            .read_snapshot(&born.snapshot, &execution)
            .unwrap()
            .unwrap();
        let record = read.get_entity(entity).unwrap();
        assert!(
            record
                .authoritative_aspect_state
                .as_ref()
                .and_then(|state| state.get(contract.key()))
                .is_none(),
            "the native birth has no assessment aspect"
        );
    }
    crate::tests::support::release_test_commit_snapshot(&runtime, &born);
    let initial = validate(
        &execution,
        &runtime,
        [set(entity, &contract, &[("a", "first")])],
    );
    let projected = initial.mutation_touches(&execution).unwrap();
    assert!(projected.touches().is_empty());
    assert_eq!(
        projected
            .struct_touches()
            .iter()
            .map(|touch| (
                touch.kind(),
                touch.contract(),
                touch.field(),
                touch.before_present(),
                touch.after_present()
            ))
            .collect::<Vec<_>>(),
        vec![(KindId(1), &contract, &field_key("a"), false, true)]
    );
    // Materialize a valid empty aspect through the native mutation authority.
    // Absence of selected fields must not be mistaken for aspect absence.
    let committed = commit(&execution, &runtime, set(entity, &contract, &[]));
    let read = runtime
        .read_truth()
        .read_snapshot(&committed.snapshot, &execution)
        .unwrap()
        .unwrap();
    let record = read.get_entity(entity).unwrap();
    let present = record
        .authoritative_aspect_state
        .as_ref()
        .unwrap()
        .get(contract.key())
        .unwrap();
    assert_eq!(present.contract(), &contract);
    let worth_foundational::facade::ContractValidatedAspectValueView::Struct(empty) =
        present.view()
    else {
        panic!("native committed aspect must be a struct");
    };
    assert!(empty.is_empty(), "the prior aspect is present and empty");
    let replacement = validate(
        &execution,
        &runtime,
        [set(entity, &contract, &[("a", "next")])],
    );
    let projected = replacement.mutation_touches(&execution).unwrap();
    assert!(projected.touches().is_empty());
    assert_eq!(
        projected
            .struct_touches()
            .iter()
            .map(|touch| (
                touch.kind(),
                touch.contract(),
                touch.field(),
                touch.before_present(),
                touch.after_present()
            ))
            .collect::<Vec<_>>(),
        vec![(KindId(1), &contract, &field_key("a"), true, true)]
    );
}

#[test]
fn optional_aspect_initialization_rejects_conflicting_repeated_sets_before_effects() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (runtime, contract) = world();
    let entity = create_entity(&execution, &runtime, "source");
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("conflicting-initializations")
                .push(set(entity, &contract, &[("a", "removed")]))
                .push(set(entity, &contract, &[("b", "kept")])),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let denied = transaction.validate(&execution, &runtime).unwrap_err();
    assert!(
        matches!(denied, crate::transactions::data::TransactionCommitError::Conflict { error, .. }
        if matches!(error.class, crate::transactions::data::ConflictClass::ConflictingIntent { .. }))
    );
}

#[test]
fn optional_aspect_initialization_cannot_omit_required_siblings() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let shape = aspects()
        .struct_fields()
        .required("required", ScalarAspectType::String)
        .optional("a", ScalarAspectType::String)
        .finish()
        .unwrap();
    let contract = aspects()
        .contract()
        .for_key(aspect_key("required-assessment"))
        .identified_by(AspectIdentity(0x917_5a12))
        .at_revision(aspects().vocabulary().revision(1))
        .struct_aspect(shape);
    let runtime = runtime_for(&contract);
    let entity = create_entity(&execution, &runtime, "source");
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("incomplete-initialization").push(set(
                entity,
                &contract,
                &[("a", "invalid")],
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let denied = transaction.validate(&execution, &runtime).unwrap_err();
    assert!(
        matches!(denied, crate::transactions::data::TransactionCommitError::Conflict { error, .. }
        if matches!(error.class, crate::transactions::data::ConflictClass::RecordAspectPatchDenied {
            denial: crate::transactions::data::RecordAspectPatchDenial::ReadmissionDenied(
                worth_foundational::facade::PortableAspectReadmissionDenial::ValueValidation {
                    denial: worth_foundational::facade::ContractValidationDenial::MissingRequiredField(ref field), ..
                }), ..
        } if field.as_str() == "required"))
    );
}

#[test]
fn optional_aspect_initialization_cannot_publish_over_a_concurrent_sibling() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (runtime, contract) = world();
    let entity = create_entity(&execution, &runtime, "source");
    let candidate = validate(
        &execution,
        &runtime,
        [set(entity, &contract, &[("a", "stale")])],
    );
    let projected = candidate.mutation_touches(&execution).unwrap();
    assert!(projected.touches().is_empty());
    assert_eq!(
        projected
            .struct_touches()
            .iter()
            .map(|touch| (
                touch.kind(),
                touch.contract(),
                touch.field(),
                touch.before_present(),
                touch.after_present()
            ))
            .collect::<Vec<_>>(),
        vec![(KindId(1), &contract, &field_key("a"), false, true)]
    );
    drop(projected);
    let committed = commit(
        &execution,
        &runtime,
        set(entity, &contract, &[("b", "winner")]),
    );
    let denied = runtime
        .prepare_validated_proposal(&execution, candidate)
        .unwrap_err();
    assert!(
        matches!(denied, crate::transactions::data::TransactionCommitError::Conflict { error, .. }
        if matches!(error.class, crate::transactions::data::ConflictClass::StaleValidationBasis { .. }))
    );
    let read = runtime
        .read_truth()
        .read_snapshot(&committed.snapshot, &execution)
        .unwrap()
        .unwrap();
    let record = read.get_entity(entity).unwrap();
    let value = record
        .authoritative_aspect_state
        .as_ref()
        .unwrap()
        .get(contract.key())
        .unwrap();
    let worth_foundational::facade::ContractValidatedAspectValueView::Struct(value) = value.view()
    else {
        panic!("struct");
    };
    assert_eq!(
        value.get(&field_key("b")),
        Some(&AspectValue::String("winner".into()))
    );
    assert_eq!(value.get(&field_key("a")), None);
}

fn world() -> (crate::runtime::RelationalRuntime, AspectContract) {
    let shape = aspects()
        .struct_fields()
        .optional("a", ScalarAspectType::String)
        .optional("b", ScalarAspectType::String)
        .finish()
        .unwrap();
    let contract = aspects()
        .contract()
        .for_key(aspect_key("assessment"))
        .identified_by(AspectIdentity(0x917_5a11))
        .at_revision(aspects().vocabulary().revision(1))
        .struct_aspect(shape);
    (runtime_for(&contract), contract)
}

fn runtime_for(contract: &AspectContract) -> crate::runtime::RelationalRuntime {
    let mut schema = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    schema.entity_aspects.push(DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField {
            field: field_key("assessment"),
        },
        contract: contract.clone(),
    });
    schema.build_runtime()
}

fn set(
    entity_id: crate::identity::data::EntityId,
    contract: &AspectContract,
    fields: &[(&str, &str)],
) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
        ApplyEntityAspectPatchIntent {
            entity_id,
            aspect_patch: PortableRecordAspectPatch::new([
                PortableAspectPatchOperation::SetWhole {
                    basis: PortableAspectContractBasis::from_contract(contract),
                    value: ContractValidationInput::Struct(
                        StructAspectValue::new(fields.iter().map(|(key, value)| {
                            (field_key(key), AspectValue::String((*value).into()))
                        }))
                        .unwrap(),
                    ),
                },
            ]),
        },
    ))
}

fn commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    intent: MutationIntent,
) -> CommitResult {
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("initialize-aspect").push(intent),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}
