use super::*;
use crate::aspects::{CanonicalAspectStateMap, PortableRecordAspectPatch};

#[test]
fn owned_patch_storage_bounds_cover_actual_backings_and_checked_overflow() {
    assert_eq!(
        PortableRecordAspectPatch::new([]).readmission_storage_bounds(),
        Some(AspectPatchStorageBounds::default())
    );
    // c65d042399: an empty persistent map and its derived width index
    // each own a root. An absent optional state still allocates neither.
    assert!(CanonicalAspectStateMap::construction_storage_bytes(0).unwrap() > 0);
    for count in [1, 2, 3, 4, 7, 8, 9, 65] {
        let bounds =
            AuthoritativeRecordAspectPatch::construction_storage_bounds(count, 0, 0).unwrap();
        let actual = Layout::array::<ContractValidatedAspectValue>(count)
            .unwrap()
            .size();
        assert_eq!(bounds.retained_bytes, actual);
        assert!(
            bounds.transient_bytes >= actual,
            "Vec/Box overlap is funded"
        );
        let field =
            FieldLevelAspectPatch::construction_storage_bounds(count, count, count).unwrap();
        let pairs = Layout::array::<(FieldKey, AspectValue)>(count)
            .unwrap()
            .size();
        let keys = Layout::array::<FieldKey>(count).unwrap().size();
        let paths = Layout::array::<CanonicalFieldPath>(count).unwrap().size();
        assert_eq!(field.retained_bytes, pairs + keys + paths + keys);
        assert!(field.transient_bytes >= pairs + keys + paths + keys);
    }
    assert!(
        AuthoritativeRecordAspectPatch::construction_storage_bounds(usize::MAX, 0, 0).is_none()
    );
    assert!(FieldLevelAspectPatch::construction_storage_bounds(0, usize::MAX, 0).is_none());
    assert!(FieldLevelAspectPatch::construction_storage_bounds(0, 0, usize::MAX).is_none());
    assert!(CanonicalAspectStateMap::construction_storage_bytes(usize::MAX).is_none());
}

#[test]
fn application_allocation_bounds_follow_selected_output_and_cloned_string_lengths() {
    use crate::aspects::*;
    use crate::values::{InternedString, ScalarAspectType};
    use worth_proof::TransitionOutcome;
    let key = AspectKey::new("data").unwrap();
    let contract = AspectContract::scalar(
        key.clone(),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );
    let mut spare = String::with_capacity(4096);
    spare.push('x');
    let TransitionOutcome::Success(value) = validate_aspect_value(
        &contract,
        ContractValidationInput::Scalar(AspectValue::String(InternedString::Raw(spare))),
    ) else {
        panic!("scalar")
    };
    let current =
        AuthoritativeRecordAspectState::from_validated_entries([value.into_parts().into_parts().0])
            .unwrap();
    let patch = AuthoritativeRecordAspectPatch::empty();
    let bounds = patch
        .application_retained_storage_bounds(Some(&current))
        .unwrap();
    let TransitionOutcome::Success(result) = patch.apply_to(&current) else {
        panic!("copy current")
    };
    let actual = result.into_parts().into_parts().0;
    assert!(bounds.retained_bytes >= actual.aspects().retained_storage_bytes().unwrap());
    // c65d042399 shares rather than trims spare capacity. Independent
    // successor custody funds that reachable backing after the source retires.
    assert!(bounds.retained_bytes >= current.owned_allocation_capacity_bytes());
    assert!(std::ptr::eq(
        actual.get(&key).unwrap(),
        current.get(&key).unwrap()
    ));
}

#[test]
fn application_allocation_bounds_count_struct_winners_once_with_sets_after_clears() {
    use crate::aspects::*;
    use crate::values::{InternedString, ScalarAspectType};
    use worth_proof::TransitionOutcome;
    let field = |name| FieldKey::new(name).unwrap();
    let text = |value: &str| AspectValue::String(InternedString::Raw(value.to_owned()));
    let contract = AspectContract::struct_aspect(
        AspectKey::new("data").unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        StructAspectShape::new(["a", "b"].map(|name| {
            FieldDeclaration::new(
                field(name),
                ScalarAspectType::String,
                FieldRequirement::Optional,
                AbsenceLaw::Optional,
                AspectEvolutionPolicy::AdditiveFieldsAllowed,
            )
            .unwrap()
        }))
        .unwrap(),
    );
    let value = StructAspectValue::new([
        (field("a"), text(&"discard".repeat(1024))),
        (field("b"), text("keep")),
    ])
    .unwrap();
    let TransitionOutcome::Success(value) =
        validate_aspect_value(&contract, ContractValidationInput::Struct(value))
    else {
        panic!("struct")
    };
    let current =
        AuthoritativeRecordAspectState::from_validated_entries([value.into_parts().into_parts().0])
            .unwrap();
    let mask = AspectMask::new([CanonicalFieldPath::single(field("a"))]);
    let TransitionOutcome::Success(patch) = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &mask,
        [(field("a"), text("new"))],
        [field("a")],
    ) else {
        panic!("genuine set and clear")
    };
    let bounds = patch
        .application_retained_storage_bounds(Some(&current))
        .unwrap();
    let TransitionOutcome::Success(result) = patch.apply_to(&current) else {
        panic!("apply")
    };
    let actual = result.into_parts().into_parts().0;
    assert!(bounds.retained_bytes >= actual.aspects().retained_storage_bytes().unwrap());
    assert!(actual.owned_allocation_capacity_bytes() < current.owned_allocation_capacity_bytes());
    let ContractValidatedAspectValueView::Struct(value) =
        actual.get(contract.key()).unwrap().view()
    else {
        panic!("struct")
    };
    assert_eq!(value.get(&field("a")), Some(&text("new")));
    assert_eq!(value.get(&field("b")), Some(&text("keep")));
}
