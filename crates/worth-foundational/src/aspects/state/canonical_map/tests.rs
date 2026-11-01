//! Independent prior BTreeMap grammar controls for canonical boxed state.
use super::*;
use crate::aspects::*;
use crate::values::{AspectValue, ScalarAspectType};
use worth_proof::TransitionOutcome;

#[derive(Serialize)]
struct LegacyMap {
    entries: BTreeMap<AspectKey, ContractValidatedAspectValue>,
}
#[derive(Serialize)]
struct LegacyState {
    aspects: LegacyMap,
}

fn scalar(name: &str, value: bool) -> ContractValidatedAspectValue {
    let contract = AspectContract::scalar(
        AspectKey::new(name).unwrap(),
        AspectIdentity(17),
        AspectContractRevision(3),
        ScalarAspectType::Bool,
    );
    let TransitionOutcome::Success(value) = validate_aspect_value(
        &contract,
        ContractValidationInput::Scalar(AspectValue::Bool(value)),
    ) else {
        panic!("actual scalar validation")
    };
    value.into_parts().into_parts().0
}

fn struct_value() -> ContractValidatedAspectValue {
    let field = FieldDeclaration::new(
        FieldKey::new("field").unwrap(),
        ScalarAspectType::Bool,
        FieldRequirement::Optional,
        AbsenceLaw::Optional,
        AspectEvolutionPolicy::AdditiveFieldsAllowed,
    )
    .unwrap();
    let contract = AspectContract::struct_aspect(
        AspectKey::new("nested").unwrap(),
        AspectIdentity(19),
        AspectContractRevision(2),
        StructAspectShape::new([field]).unwrap(),
    );
    let value =
        StructAspectValue::new([(FieldKey::new("field").unwrap(), AspectValue::Bool(true))])
            .unwrap();
    let TransitionOutcome::Success(value) =
        validate_aspect_value(&contract, ContractValidationInput::Struct(value))
    else {
        panic!("actual struct validation")
    };
    value.into_parts().into_parts().0
}

#[test]
fn canonical_boxed_state_preserves_prior_map_wire_named_compact_json_and_clone() {
    for input in [
        vec![],
        vec![scalar("z", false), struct_value(), scalar("a", true)],
    ] {
        let legacy = LegacyState {
            aspects: LegacyMap {
                entries: input
                    .iter()
                    .map(|value| (value.key().clone(), value.clone()))
                    .collect(),
            },
        };
        let map = CanonicalAspectStateMap::from_validated_entries(input).unwrap();
        let state = AuthoritativeRecordAspectState::from_canonical_map(map);
        assert_eq!(
            rmp_serde::to_vec_named(&state).unwrap(),
            rmp_serde::to_vec_named(&legacy).unwrap()
        );
        assert_eq!(
            rmp_serde::to_vec(&state).unwrap(),
            rmp_serde::to_vec(&legacy).unwrap()
        );
        assert_eq!(
            serde_json::to_vec(&state).unwrap(),
            serde_json::to_vec(&legacy).unwrap()
        );
        let cloned = state.clone();
        assert_eq!(cloned, state);
        assert_eq!(
            state
                .aspects()
                .entries()
                .map(|(key, _)| key.as_str())
                .collect::<Vec<_>>(),
            legacy
                .aspects
                .entries
                .keys()
                .map(AspectKey::as_str)
                .collect::<Vec<_>>()
        );
        for key in legacy.aspects.entries.keys() {
            assert_eq!(cloned.get(key), state.get(key));
            assert_ne!(
                cloned.get(key).unwrap().key().as_str().as_ptr(),
                state.get(key).unwrap().key().as_str().as_ptr(),
                "clone owns independent key data"
            );
        }
    }
}
