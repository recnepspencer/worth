use {worth_foundational::AspectMask, worth_foundational::MutationMask};

use {
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::admitted_state,
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::task_summary_contract,
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::validated_task_summary,
};
use {
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision,
};

#[test]
fn field_level_patch_uses_struct_contract_and_mutation_mask() {
    let contract = task_summary_contract();
    let state = admitted_state([validated_task_summary(
        &contract,
        "Ship it",
        false,
        Some("draft"),
    )]);
    let mask = AspectMask::<MutationMask>::new([
        worth_foundational::CanonicalFieldPath::single(field("done")),
        worth_foundational::CanonicalFieldPath::single(field("note")),
    ]);

    let patch = worth_foundational::AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &mask,
        [(field("done"), worth_foundational::AspectValue::Bool(true))],
        [field("note")],
    );
    let worth_proof::TransitionOutcome::Success(patch) = patch else {
        panic!("expected field patch construction to succeed");
    };
    let worth_proof::TransitionOutcome::Success(next_state) = patch.apply_to(state.payload())
    else {
        panic!("expected field patch application to succeed");
    };

    let task = next_state
        .payload()
        .get(&key("task.summary"))
        .expect("task remains admitted");
    let worth_foundational::ContractValidatedAspectValueView::Struct(value) = task.view() else {
        panic!("task remains struct");
    };

    assert_eq!(
        value.get(&field("done")),
        Some(&worth_foundational::AspectValue::Bool(true))
    );
    assert_eq!(value.get(&field("note")), None);
}

#[test]
fn field_level_patch_rejects_scalar_contracts_and_required_field_clears() {
    let scalar_contract = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(9),
        revision(1),
        worth_foundational::ScalarAspectType::Int64,
    );
    let scalar_mask =
        AspectMask::<MutationMask>::new([worth_foundational::CanonicalFieldPath::single(field(
            "count",
        ))]);

    let scalar_denial = worth_foundational::AuthoritativeRecordAspectPatch::field_level(
        &scalar_contract,
        &scalar_mask,
        [(field("count"), worth_foundational::AspectValue::Int64(1))],
        [],
    );
    assert_eq!(
        scalar_denial,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::AuthoritativePatchConstructionDenial::FieldPatchRequiresStructAspect
        )
    );

    let contract = task_summary_contract();
    let required_clear = worth_foundational::AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &AspectMask::<MutationMask>::new([worth_foundational::CanonicalFieldPath::single(field(
            "title",
        ))]),
        [],
        [field("title")],
    );
    assert_eq!(
        required_clear,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::AuthoritativePatchConstructionDenial::RequiredFieldClearDenied(
                field("title")
            )
        )
    );
}

#[test]
fn field_level_patch_rejects_unselected_mask_fields_and_duplicate_sets() {
    let contract = task_summary_contract();
    let note_only_mask =
        AspectMask::<MutationMask>::new([worth_foundational::CanonicalFieldPath::single(field(
            "note",
        ))]);

    let unselected_field = worth_foundational::AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &note_only_mask,
        [(field("done"), worth_foundational::AspectValue::Bool(true))],
        [],
    );
    assert_eq!(
        unselected_field,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::AuthoritativePatchConstructionDenial::FieldNotSelectedByMutationMask(field("done"))
        )
    );

    let duplicate_set = worth_foundational::AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &AspectMask::<MutationMask>::new([worth_foundational::CanonicalFieldPath::single(field(
            "note",
        ))]),
        [
            (
                field("note"),
                worth_foundational::AspectValue::String("first".into()),
            ),
            (
                field("note"),
                worth_foundational::AspectValue::String("second".into()),
            ),
        ],
        [],
    );
    assert_eq!(
        duplicate_set,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::AuthoritativePatchConstructionDenial::DuplicateFieldSet(field(
                "note"
            ))
        )
    );
}

#[test]
fn field_level_patch_requires_explicit_field_mask() {
    let contract = task_summary_contract();

    let outcome = worth_foundational::AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &AspectMask::<MutationMask>::whole_aspect(),
        [(field("done"), worth_foundational::AspectValue::Bool(true))],
        [],
    );

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::AuthoritativePatchConstructionDenial::FieldPatchRequiresFieldMask
        )
    );
}

#[test]
fn field_level_patch_rejects_empty_field_mutation_requests() {
    let contract = task_summary_contract();

    let outcome = worth_foundational::AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &AspectMask::<MutationMask>::new([worth_foundational::CanonicalFieldPath::single(field(
            "done",
        ))]),
        [],
        [],
    );

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::AuthoritativePatchConstructionDenial::EmptyFieldPatch
        )
    );
}
