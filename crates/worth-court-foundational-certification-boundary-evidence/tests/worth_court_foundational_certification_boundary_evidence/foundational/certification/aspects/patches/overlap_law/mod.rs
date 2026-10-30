use {worth_foundational::AspectMask, worth_foundational::MutationMask};

use {
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::task_summary_contract,
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::validated_task_summary,
};
use {
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key,
};

#[test]
fn whole_and_field_patch_overlap_is_rejected_as_ambiguous() {
    let contract = task_summary_contract();
    let worth_proof::TransitionOutcome::Success(whole) =
        worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect(
            [validated_task_summary(&contract, "Ship it", false, None)],
            [],
        )
    else {
        panic!("expected whole-aspect patch construction to succeed");
    };
    let worth_proof::TransitionOutcome::Success(field_patch) =
        worth_foundational::AuthoritativeRecordAspectPatch::field_level(
            &contract,
            &AspectMask::<MutationMask>::new([worth_foundational::CanonicalFieldPath::single(
                field("done"),
            )]),
            [(field("done"), worth_foundational::AspectValue::Bool(true))],
            [],
        )
    else {
        panic!("expected field patch construction to succeed");
    };

    let combined = worth_foundational::AuthoritativeRecordAspectPatch::combine(whole, field_patch);

    assert_eq!(
        combined,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::AuthoritativePatchConstructionDenial::AmbiguousWholeAndFieldPatch(
                key("task.summary")
            )
        )
    );
}

#[test]
fn duplicate_field_patch_combination_is_rejected() {
    let contract = task_summary_contract();
    let mask = AspectMask::<MutationMask>::new([worth_foundational::CanonicalFieldPath::single(
        field("done"),
    )]);
    let worth_proof::TransitionOutcome::Success(first) =
        worth_foundational::AuthoritativeRecordAspectPatch::field_level(
            &contract,
            &mask,
            [(field("done"), worth_foundational::AspectValue::Bool(true))],
            [],
        )
    else {
        panic!("expected first field patch construction to succeed");
    };
    let worth_proof::TransitionOutcome::Success(second) =
        worth_foundational::AuthoritativeRecordAspectPatch::field_level(
            &contract,
            &mask,
            [(field("done"), worth_foundational::AspectValue::Bool(false))],
            [],
        )
    else {
        panic!("expected second field patch construction to succeed");
    };

    let combined = worth_foundational::AuthoritativeRecordAspectPatch::combine(first, second);

    assert_eq!(
        combined,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::AuthoritativePatchConstructionDenial::DuplicateFieldPatch(key(
                "task.summary"
            ))
        )
    );
}

#[test]
fn disjoint_field_patches_for_one_aspect_combine_without_losing_masks() {
    let contract = task_summary_contract();
    let title_mask =
        AspectMask::<MutationMask>::new([worth_foundational::CanonicalFieldPath::single(field(
            "title",
        ))]);
    let done_mask =
        AspectMask::<MutationMask>::new([worth_foundational::CanonicalFieldPath::single(field(
            "done",
        ))]);
    let worth_proof::TransitionOutcome::Success(title) =
        worth_foundational::AuthoritativeRecordAspectPatch::field_level(
            &contract,
            &title_mask,
            [(
                field("title"),
                worth_foundational::AspectValue::String("Ship it".into()),
            )],
            [],
        )
    else {
        panic!("expected title field patch construction to succeed");
    };
    let worth_proof::TransitionOutcome::Success(done) =
        worth_foundational::AuthoritativeRecordAspectPatch::field_level(
            &contract,
            &done_mask,
            [(field("done"), worth_foundational::AspectValue::Bool(true))],
            [],
        )
    else {
        panic!("expected done field patch construction to succeed");
    };

    let worth_proof::TransitionOutcome::Success(combined) =
        worth_foundational::AuthoritativeRecordAspectPatch::combine(title, done)
    else {
        panic!("disjoint fields in one aspect should combine");
    };
    let (_, fields) = combined.field_patches().next().expect("combined fields");
    assert_eq!(fields.field_sets().count(), 2);
    assert_eq!(fields.mask().paths().len(), 2);
}
