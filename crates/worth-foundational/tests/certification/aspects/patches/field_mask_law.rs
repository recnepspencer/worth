use worth_foundational::{
    AspectContract, AspectMask, AspectValue, AuthoritativePatchConstructionDenial,
    AuthoritativeRecordAspectPatch, CanonicalFieldPath, MutationMask, ScalarAspectType,
};
use worth_proof::TransitionOutcome;

use super::patch_fixtures::{admitted_state, task_summary_contract, validated_task_summary};
use crate::certification::foundational_vocabulary::{field, identity, key, revision};

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
        CanonicalFieldPath::single(field("done")),
        CanonicalFieldPath::single(field("note")),
    ]);

    let patch = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &mask,
        [(field("done"), AspectValue::Bool(true))],
        [field("note")],
    );
    let TransitionOutcome::Success(patch) = patch else {
        panic!("expected field patch construction to succeed");
    };
    let TransitionOutcome::Success(next_state) = patch.apply_to(state.payload()) else {
        panic!("expected field patch application to succeed");
    };

    let task = next_state
        .payload()
        .get(&key("task.summary"))
        .expect("task remains admitted");
    let worth_foundational::ContractValidatedAspectValueView::Struct(value) = task.view() else {
        panic!("task remains struct");
    };

    assert_eq!(value.get(&field("done")), Some(&AspectValue::Bool(true)));
    assert_eq!(value.get(&field("note")), None);
}

#[test]
fn field_level_patch_rejects_scalar_contracts_and_required_field_clears() {
    let scalar_contract = AspectContract::scalar(
        key("count"),
        identity(9),
        revision(1),
        ScalarAspectType::Int64,
    );
    let scalar_mask = AspectMask::<MutationMask>::new([CanonicalFieldPath::single(field("count"))]);

    let scalar_denial = AuthoritativeRecordAspectPatch::field_level(
        &scalar_contract,
        &scalar_mask,
        [(field("count"), AspectValue::Int64(1))],
        [],
    );
    assert_eq!(
        scalar_denial,
        TransitionOutcome::Denied(
            AuthoritativePatchConstructionDenial::FieldPatchRequiresStructAspect
        )
    );

    let contract = task_summary_contract();
    let required_clear = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &AspectMask::<MutationMask>::new([CanonicalFieldPath::single(field("title"))]),
        [],
        [field("title")],
    );
    assert_eq!(
        required_clear,
        TransitionOutcome::Denied(
            AuthoritativePatchConstructionDenial::RequiredFieldClearDenied(field("title"))
        )
    );
}

#[test]
fn field_level_patch_rejects_unselected_mask_fields_and_duplicate_sets() {
    let contract = task_summary_contract();
    let note_only_mask =
        AspectMask::<MutationMask>::new([CanonicalFieldPath::single(field("note"))]);

    let unselected_field = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &note_only_mask,
        [(field("done"), AspectValue::Bool(true))],
        [],
    );
    assert_eq!(
        unselected_field,
        TransitionOutcome::Denied(
            AuthoritativePatchConstructionDenial::FieldNotSelectedByMutationMask(field("done"))
        )
    );

    let duplicate_set = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &AspectMask::<MutationMask>::new([CanonicalFieldPath::single(field("note"))]),
        [
            (field("note"), AspectValue::String("first".into())),
            (field("note"), AspectValue::String("second".into())),
        ],
        [],
    );
    assert_eq!(
        duplicate_set,
        TransitionOutcome::Denied(AuthoritativePatchConstructionDenial::DuplicateFieldSet(
            field("note")
        ))
    );
}

#[test]
fn field_level_patch_requires_explicit_field_mask() {
    let contract = task_summary_contract();

    let outcome = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &AspectMask::<MutationMask>::whole_aspect(),
        [(field("done"), AspectValue::Bool(true))],
        [],
    );

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(
            AuthoritativePatchConstructionDenial::FieldPatchRequiresFieldMask
        )
    );
}

#[test]
fn field_level_patch_rejects_empty_field_mutation_requests() {
    let contract = task_summary_contract();

    let outcome = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &AspectMask::<MutationMask>::new([CanonicalFieldPath::single(field("done"))]),
        [],
        [],
    );

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(AuthoritativePatchConstructionDenial::EmptyFieldPatch)
    );
}
