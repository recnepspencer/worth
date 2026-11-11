//! Actual field backing, not lookup-key capacity or a copied-value approximation.
use crate::aspects::*;
use crate::values::{AspectValue, InternedString, ScalarAspectType};
use worth_proof::TransitionOutcome;

#[test]
fn field_quotes_cover_stored_key_capacity_and_shared_text_through_tree_depths() {
    for count in [1usize, 3, 31, 65] {
        for clear_then_set in [false, true] {
            let keys: Vec<_> = (0..count)
                .map(|n| FieldKey::new(format!("f{n:04}")).unwrap())
                .collect();
            let shape = StructAspectShape::new(keys.iter().cloned().map(|key| {
                FieldDeclaration::new(
                    key,
                    ScalarAspectType::String,
                    FieldRequirement::Optional,
                    AbsenceLaw::Optional,
                    AspectEvolutionPolicy::AdditiveFieldsAllowed,
                )
                .unwrap()
            }))
            .unwrap();
            let contract = AspectContract::struct_aspect(
                AspectKey::new("data").unwrap(),
                AspectIdentity(7),
                AspectContractRevision(2),
                shape,
            );
            let fields = keys
                .iter()
                .enumerate()
                .map(|(n, key)| {
                    let mut spelling =
                        String::with_capacity(if n == 0 { 8192 } else { key.as_str().len() });
                    spelling.push_str(key.as_str());
                    (
                        FieldKey::new(spelling).unwrap(),
                        AspectValue::String(InternedString::Stored(
                            crate::facade::StoredValueText::new(format!("stored-{n}")),
                        )),
                    )
                })
                .collect();
            let fields = StructAspectValue::from_sorted_fields(fields).unwrap();
            let TransitionOutcome::Success(value) =
                validate_aspect_value(&contract, ContractValidationInput::Struct(fields))
            else {
                panic!("genuine fields")
            };
            let current = AuthoritativeRecordAspectState::from_validated_entries([value
                .into_parts()
                .into_parts()
                .0])
            .unwrap();
            let changed = keys[0].clone();
            let mask = AspectMask::new([CanonicalFieldPath::single(changed.clone())]);
            let replacement = AspectValue::String(InternedString::Stored(
                crate::facade::StoredValueText::new("replacement".to_owned()),
            ));
            let clears = if clear_then_set {
                vec![changed.clone()]
            } else {
                vec![]
            };
            let TransitionOutcome::Success(patch) = AuthoritativeRecordAspectPatch::field_level(
                &contract,
                &mask,
                [(changed.clone(), replacement.clone())],
                clears,
            ) else {
                panic!("genuine canonical field patch")
            };
            let bound = patch
                .application_retained_storage_bounds(Some(&current))
                .unwrap();
            let (result, peak) = crate::allocation_observation::peak(|| patch.apply_to(&current));
            let TransitionOutcome::Success(result) = result else {
                panic!("field update")
            };
            let result = result.into_parts().into_parts().0;
            assert!(peak <= bound.retained_bytes + bound.transient_bytes);
            assert!(result.aspects().retained_storage_bytes().unwrap() <= bound.retained_bytes);
            let ContractValidatedAspectValueView::Struct(previous) =
                current.get(contract.key()).unwrap().view()
            else {
                panic!("previous struct")
            };
            let ContractValidatedAspectValueView::Struct(next) =
                result.get(contract.key()).unwrap().view()
            else {
                panic!("successor struct")
            };
            assert_ne!(previous.get(&changed), Some(&replacement));
            assert_eq!(next.get(&changed), Some(&replacement));
            if let Some(unchanged) = keys.get(1) {
                assert!(std::ptr::eq(
                    previous.get(unchanged).unwrap(),
                    next.get(unchanged).unwrap()
                ));
            }
            drop(current);
            drop(patch);
            let ContractValidatedAspectValueView::Struct(next) =
                result.get(contract.key()).unwrap().view()
            else {
                panic!("retained successor")
            };
            assert_eq!(next.get(&changed), Some(&replacement));
        }
    }
}
