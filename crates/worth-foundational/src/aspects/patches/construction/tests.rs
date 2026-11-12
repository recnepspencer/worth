use crate::aspects::*;
use crate::values::{AspectValue, ScalarAspectType};
use std::cell::Cell;
use worth_proof::TransitionOutcome;

fn field(name: &str) -> FieldKey {
    FieldKey::new(name).unwrap()
}
fn contract() -> AspectContract {
    let fields = [
        ("a", FieldRequirement::Optional),
        ("b", FieldRequirement::Optional),
        ("required", FieldRequirement::Required),
    ]
    .map(|(name, requirement)| {
        FieldDeclaration::new(
            field(name),
            ScalarAspectType::Bool,
            requirement,
            if requirement == FieldRequirement::Required {
                AbsenceLaw::Required
            } else {
                AbsenceLaw::Optional
            },
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap()
    });
    AspectContract::struct_aspect(
        AspectKey::new("record").unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        StructAspectShape::new(fields).unwrap(),
    )
}
fn mask() -> AspectMask<MutationMask> {
    AspectMask::new(["a", "b", "required"].map(|key| CanonicalFieldPath::single(field(key))))
}

#[test]
fn field_owned_and_lazy_entries_preserve_type_duplicate_and_required_clear_priority() {
    let contract = contract();
    let mask = mask();
    let sets = vec![
        (field("a"), AspectValue::Bool(true)),
        (field("a"), AspectValue::UInt64(4)),
    ];
    let TransitionOutcome::Denied(expected) =
        AuthoritativeRecordAspectPatch::field_level(&contract, &mask, sets.clone(), [])
    else {
        panic!("type denial")
    };
    assert!(matches!(
        expected,
        AuthoritativePatchConstructionDenial::FieldTypeMismatch { .. }
    ));
    assert_eq!(
        FieldLevelAspectPatch::from_owned_fields(&contract, &mask, sets, vec![]).unwrap_err(),
        expected
    );
    let sets = vec![(field("required"), AspectValue::Bool(true))];
    let expected =
        AuthoritativePatchConstructionDenial::RequiredFieldClearDenied(field("required"));
    assert_eq!(
        FieldLevelAspectPatch::from_owned_fields(
            &contract,
            &mask,
            sets.clone(),
            vec![field("required")]
        )
        .unwrap_err(),
        expected
    );
    assert!(
        matches!(AuthoritativeRecordAspectPatch::field_level(&contract, &mask, sets, [field("required")]), TransitionOutcome::Denied(denial) if denial == expected)
    );
}

#[test]
fn generic_whole_constructor_stops_at_first_duplicate_before_requesting_sets() {
    let contract = contract();
    let calls = Cell::new(0);
    let clears = std::iter::from_fn(|| {
        calls.set(calls.get() + 1);
        assert!(calls.get() <= 2, "duplicate must stop the lazy iterator");
        Some(contract.clone())
    });
    let sets = std::iter::from_fn(|| -> Option<ContractValidatedAspectArtifact> {
        panic!("clears deny before sets are requested")
    });
    assert!(matches!(
        AuthoritativeRecordAspectPatch::whole_aspect(sets, clears),
        TransitionOutcome::Denied(
            AuthoritativePatchConstructionDenial::DuplicateWholeAspectClear(_)
        )
    ));
    assert_eq!(calls.get(), 2);
}

#[test]
fn canonical_field_combination_keeps_set_precedence_and_refuses_overlapping_fields() {
    let contract = contract();
    let mask = mask();
    let TransitionOutcome::Success(left) = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &mask,
        [(field("a"), AspectValue::Bool(true))],
        [field("a")],
    ) else {
        panic!("set wins optional clear")
    };
    let field_patch = left.field_patches().next().unwrap().1;
    assert_eq!(field_patch.field_sets().count(), 1);
    assert_eq!(field_patch.field_clears().count(), 0);
    let TransitionOutcome::Success(right) = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &mask,
        [(field("b"), AspectValue::Bool(false))],
        [],
    ) else {
        panic!("disjoint right")
    };
    let TransitionOutcome::Success(combined) =
        AuthoritativeRecordAspectPatch::combine(left.clone(), right)
    else {
        panic!("disjoint field merge")
    };
    assert_eq!(
        combined
            .field_patches()
            .next()
            .unwrap()
            .1
            .field_sets()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert!(matches!(
        AuthoritativeRecordAspectPatch::combine(left.clone(), left),
        TransitionOutcome::Denied(AuthoritativePatchConstructionDenial::DuplicateFieldPatch(_))
    ));
}

#[test]
fn owned_field_roster_matches_ordinary_construction_and_moves_first_duplicate_clear_key() {
    let contract = contract();
    let mask = mask();
    let first = field("a");
    let original_key = first.as_str().as_ptr();
    let sets = vec![(field("b"), AspectValue::Bool(false))];
    let owned = FieldLevelAspectPatch::from_owned_fields(
        &contract,
        &mask,
        sets.clone(),
        vec![first, field("a")],
    )
    .unwrap();
    let TransitionOutcome::Success(ordinary) = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &mask,
        sets,
        [field("a"), field("a")],
    ) else {
        panic!("ordinary field admission")
    };
    assert_eq!(&owned, ordinary.field_patches().next().unwrap().1);
    assert_eq!(
        owned.field_clears().next().unwrap().as_str().as_ptr(),
        original_key
    );
}

#[test]
fn whole_constructor_retains_contract_overlap_refusal_before_set_overrides_clear() {
    let before = contract();
    let AspectShape::Struct(shape) = before.shape() else {
        unreachable!()
    };
    let after = AspectContract::struct_aspect(
        before.key().clone(),
        before.identity(),
        AspectContractRevision(2),
        shape.clone(),
    );
    let input = StructAspectValue::new([(field("required"), AspectValue::Bool(true))]).unwrap();
    let TransitionOutcome::Success(set) =
        validate_aspect_value(&after, ContractValidationInput::Struct(input))
    else {
        panic!("valid replacement contract value")
    };
    assert!(matches!(
        AuthoritativeRecordAspectPatch::whole_aspect([set], [before]),
        TransitionOutcome::Denied(
            AuthoritativePatchConstructionDenial::OverlappingWholeAspectContractMismatch(_)
        )
    ));
}
