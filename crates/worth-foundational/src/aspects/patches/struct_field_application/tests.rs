use super::*;
use crate::aspects::FieldKey;
use crate::aspects::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, AspectMask, AuthoritativeRecordAspectPatch, CanonicalFieldPath, FieldDeclaration,
    FieldRequirement, StructAspectShape,
};
use crate::values::AspectValue;
use crate::values::{InternedString, ScalarAspectType};
use worth_proof::TransitionOutcome;

fn key(name: &str) -> FieldKey {
    FieldKey::new(name).unwrap()
}

fn patch() -> AuthoritativeRecordAspectPatch {
    let fields = ["a", "b", "c", "d", "z"].map(|name| {
        FieldDeclaration::new(
            key(name),
            ScalarAspectType::String,
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap()
    });
    let contract = AspectContract::struct_aspect(
        AspectKey::new("record").unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        StructAspectShape::new(fields).unwrap(),
    );
    let mask = AspectMask::new(
        ["a", "b", "c", "d", "z"].map(|name| CanonicalFieldPath::single(key(name))),
    );
    let TransitionOutcome::Success(patch) = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &mask,
        [(key("b"), text("new-b")), (key("d"), text("new-d"))],
        [key("a"), key("b"), key("z")],
    ) else {
        panic!("the genuine field patch must be admitted")
    };
    patch
}

fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.to_owned()))
}

#[test]
fn field_application_merges_canonical_streams_with_sets_after_clears() {
    let current = StructAspectValue::new([
        (key("c"), text("keep-c")),
        (key("a"), text("remove-a")),
        (key("b"), text("old-b")),
    ])
    .unwrap();
    let patch = patch();
    let field_patch = patch.field_patches().next().unwrap().1;
    let bound = field_patch
        .struct_field_merge_allocation_bound(&current)
        .unwrap();
    let result = current.patched(field_patch.field_sets(), field_patch.field_clears());
    assert_eq!(
        result
            .fields()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        ["b", "c", "d"]
    );
    assert_eq!(result.get(&key("b")), Some(&text("new-b")));
    assert_eq!(result.get(&key("c")), Some(&text("keep-c")));
    assert!(bound >= result.owned_allocation_capacity_bytes());
    let validated = crate::aspects::validate_aspect_value(
        field_patch.contract(),
        crate::aspects::ContractValidationInput::Struct(current),
    );
    let TransitionOutcome::Success(validated) = validated else {
        panic!("valid initial state")
    };
    let state = crate::aspects::AuthoritativeRecordAspectState::from_validated_entries([validated
        .into_parts()
        .into_parts()
        .0])
    .unwrap();
    let TransitionOutcome::Success(applied) = patch.apply_to(&state) else {
        panic!("actual patch application")
    };
    let state = applied.into_parts().into_parts().0;
    let crate::aspects::ContractValidatedAspectValueView::Struct(actual) =
        state.get(field_patch.key()).unwrap().view()
    else {
        panic!("struct result")
    };
    assert_eq!(actual, &result);
}

#[test]
fn merge_quote_observes_actual_cloned_payload_capacities() {
    let patch = patch();
    let field_patch = patch.field_patches().next().unwrap().1;
    let small = StructAspectValue::new([(key("c"), text("keep"))]).unwrap();
    let mut large = String::with_capacity(512);
    large.push_str("keep");
    let large =
        StructAspectValue::new([(key("c"), AspectValue::String(InternedString::Raw(large)))])
            .unwrap();
    assert_eq!(small, large);
    assert!(
        field_patch
            .struct_field_merge_allocation_bound(&large)
            .unwrap()
            > field_patch
                .struct_field_merge_allocation_bound(&small)
                .unwrap()
    );
}
