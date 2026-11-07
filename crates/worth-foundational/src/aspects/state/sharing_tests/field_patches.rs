//! Delta field application agrees with fresh admission and retains immutable observers.
use super::*;
use crate::aspects::{
    contracts::{AbsenceLaw, AspectShape},
    evolution::AspectEvolutionPolicy,
    masks::{AspectMask, MutationMask},
    structs::{
        CanonicalFieldPath, FieldDeclaration, FieldKey, FieldRequirement, StructAspectShape,
        StructAspectValue,
    },
    validation::{
        ContractValidatedAspectValueView as View, ContractValidationDenial,
        FieldValidationObservation,
    },
};
use std::collections::BTreeMap;

fn key(index: usize) -> FieldKey {
    FieldKey::new(format!("field-{index:05}")).unwrap()
}
fn contract(size: usize) -> AspectContract {
    AspectContract::struct_aspect(
        AspectKey::new("data").unwrap(),
        AspectIdentity(900),
        AspectContractRevision(1),
        StructAspectShape::new((0..size).map(|index| {
            FieldDeclaration::new(
                key(index),
                ScalarAspectType::Int64,
                if index == 0 {
                    FieldRequirement::Required
                } else {
                    FieldRequirement::Optional
                },
                if index == 0 {
                    AbsenceLaw::Required
                } else {
                    AbsenceLaw::Optional
                },
                AspectEvolutionPolicy::ExplicitBreakRequired,
            )
            .unwrap()
        }))
        .unwrap(),
    )
}
fn admitted(
    contract: &AspectContract,
    fields: BTreeMap<FieldKey, AspectValue>,
) -> crate::aspects::validation::ContractValidatedAspectValue {
    let TransitionOutcome::Success(entry) = validate_aspect_value(
        contract,
        ContractValidationInput::Struct(StructAspectValue::new(fields).unwrap()),
    ) else {
        panic!("valid fields")
    };
    entry.into_parts().into_parts().0
}
fn state(
    entry: crate::aspects::validation::ContractValidatedAspectValue,
) -> AuthoritativeRecordAspectState {
    AuthoritativeRecordAspectState::from_validated_entries([entry]).unwrap()
}
fn patch(
    contract: &AspectContract,
    changed: usize,
    value: i64,
    clear: Option<usize>,
) -> AuthoritativeRecordAspectPatch {
    let mask = AspectMask::<MutationMask>::new(
        std::iter::once(changed)
            .chain(clear)
            .map(|index| CanonicalFieldPath::single(key(index))),
    );
    let TransitionOutcome::Success(patch) = AuthoritativeRecordAspectPatch::field_level(
        contract,
        &mask,
        [(key(changed), AspectValue::Int64(value))],
        clear.map(key),
    ) else {
        panic!("valid field patch")
    };
    patch
}
fn struct_value(state: &AuthoritativeRecordAspectState) -> &StructAspectValue {
    let View::Struct(value) = state.get(&AspectKey::new("data").unwrap()).unwrap().view() else {
        panic!("struct")
    };
    value
}
#[test]
fn mixed_field_deltas_equal_fresh_admission_without_copying_untouched_values() {
    let contract = contract(128);
    let mut expected: BTreeMap<_, _> = (0..128)
        .map(|i| (key(i), AspectValue::Int64(i as i64)))
        .collect();
    let initial = state(admitted(&contract, expected.clone()));
    let mut current = initial.clone();
    for step in 0..16 {
        let before = current.clone();
        let changed = 1 + step % 4;
        let clear = 8 + step % 4;
        let patch = patch(&contract, changed, -(step as i64), Some(clear));
        let TransitionOutcome::Success(next) = patch.apply_to(&current) else {
            panic!("delta applies")
        };
        current = next.into_parts().into_parts().0;
        expected.insert(key(changed), AspectValue::Int64(-(step as i64)));
        expected.remove(&key(clear));
        let oracle = state(admitted(&contract, expected.clone()));
        assert_eq!(current, oracle);
        assert_eq!(
            current.owned_allocation_capacity_bytes(),
            oracle.owned_allocation_capacity_bytes()
        );
        assert_eq!(
            struct_value(&current).semantic_byte_width(),
            struct_value(&oracle).semantic_byte_width()
        );
        for (field, value) in struct_value(&before).fields() {
            if expected.contains_key(field) && field != &key(changed) {
                assert!(std::ptr::eq(
                    value,
                    struct_value(&current).get(field).unwrap()
                ));
            }
        }
        let wire = serde_json::to_value(struct_value(&current)).unwrap();
        assert_eq!(wire, serde_json::json!({ "fields": expected }));
        let reopened: StructAspectValue = serde_json::from_value(wire).unwrap();
        assert_eq!(&reopened, struct_value(&current));
        assert_eq!(
            reopened.owned_allocation_capacity_bytes(),
            struct_value(&current).owned_allocation_capacity_bytes()
        );
    }
    assert_eq!(
        struct_value(&initial).get(&key(1)),
        Some(&AspectValue::Int64(1))
    );
}
#[test]
fn a_new_definition_and_deliberately_damaged_values_require_full_validation() {
    let original = contract(8);
    let mut entry = admitted(
        &original,
        (0..8)
            .map(|i| (key(i), AspectValue::Int64(i as i64)))
            .collect(),
    );
    let prior = state(entry.clone());
    assert!(entry.corrupt_struct_field_for_test(&key(0), None));
    let damaged = state(entry);
    let TransitionOutcome::Denied(denial) = patch(&original, 1, -1, None).apply_to(&damaged) else {
        panic!("missing required field refuses")
    };
    assert_eq!(
        denial,
        crate::aspects::patches::AuthoritativePatchApplicationDenial::ContractValidationDenied(
            ContractValidationDenial::MissingRequiredField(key(0))
        )
    );
    assert!(struct_value(&prior).get(&key(0)).is_some());
    let rebound = contract(9);
    let TransitionOutcome::Success(result) = patch(&rebound, 8, 42, None).apply_to(&prior) else {
        panic!("genuine contract rebind validates")
    };
    assert_eq!(
        struct_value(result.payload()).get(&key(8)),
        Some(&AspectValue::Int64(42))
    );
    assert!(matches!(original.shape(), AspectShape::Struct(_)));
}
mod cost {
    use super::*;
    fn work(size: usize) -> usize {
        let contract = contract(size);
        let current = state(admitted(
            &contract,
            (0..size)
                .map(|i| (key(i), AspectValue::Int64(i as i64)))
                .collect(),
        ));
        let patch = patch(&contract, 1, -1, None);
        let observer = FieldValidationObservation::start();
        let TransitionOutcome::Success(_) = patch.apply_to(&current) else {
            panic!("valid delta")
        };
        observer.work()
    }
    #[test]
    fn one_field_patch_validation_is_flat_in_unchanged_fields() {
        let small = work(128);
        assert!(small > 0, "observe actual changed-field validation");
        for large in [work(256), work(512)] {
            assert!(
                (large as f64) / (small as f64) < 1.5,
                "full-field validation growth: {small} -> {large}"
            );
        }
    }
}
