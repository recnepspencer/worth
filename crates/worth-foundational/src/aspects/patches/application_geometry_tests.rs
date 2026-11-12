//! Physical allocator regressions for the existing persistent application owner.
use super::*;
use crate::aspects::*;
use crate::values::{AspectValue, ScalarAspectType};
use worth_proof::TransitionOutcome;

fn contract(name: String) -> AspectContract {
    AspectContract::scalar(
        AspectKey::new(name).unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::Bool,
    )
}
fn admitted(contract: &AspectContract) -> ContractValidatedAspectValue {
    let TransitionOutcome::Success(value) = validate_aspect_value(
        contract,
        ContractValidationInput::Scalar(AspectValue::Bool(true)),
    ) else {
        panic!("actual admitted scalar")
    };
    value.into_parts().into_parts().0
}
fn patch(
    sets: Vec<ContractValidatedAspectValue>,
    clears: Vec<AspectContract>,
) -> AuthoritativeRecordAspectPatch {
    AuthoritativeRecordAspectPatch {
        whole_aspect_sets: sets.into_boxed_slice(),
        whole_aspect_clears: clears.into_boxed_slice(),
        field_patches: Box::new([]),
    }
}

#[test]
fn birth_quote_covers_actual_root_boundaries_without_leaf_cow_scratch() {
    for count in [0, 1, 31, 62, 63, 64, 65] {
        for distinct_widths in [false, true] {
            let contracts: Vec<_> = (0..count)
                .map(|n| {
                    let suffix = if distinct_widths {
                        "x".repeat(n)
                    } else {
                        String::new()
                    };
                    contract(format!("k{n:04}{suffix}"))
                })
                .collect();
            let patch = patch(contracts.iter().map(admitted).collect(), vec![]);
            let bound = patch.application_retained_storage_bounds(None).unwrap();
            let (result, peak) =
                crate::allocation_observation::peak(|| patch.apply_to_optional(None));
            let TransitionOutcome::Success(result) = result else {
                panic!("actual birth")
            };
            if count == 0 {
                assert!(result.is_none());
                assert_eq!(
                    (bound.retained_bytes, bound.transient_bytes, peak),
                    (0, 0, 0)
                );
                continue;
            }
            let result = result.expect("nonempty actual birth");
            let result = result.into_parts().into_parts().0;
            assert_eq!(result.aspects().len(), count);
            assert!(peak <= bound.retained_bytes + bound.transient_bytes);
            assert!(result.aspects().retained_storage_bytes().unwrap() <= bound.retained_bytes);
            if count <= 64 {
                assert_eq!(
                    bound.transient_bytes, 0,
                    "unique leaf construction is paid by retention"
                );
            }
            for contract in &contracts {
                assert_eq!(
                    result.get(contract.key()).unwrap().view(),
                    ContractValidatedAspectValueView::Scalar(&AspectValue::Bool(true))
                );
            }
        }
    }
}

#[test]
fn retained_leaf_batch_detaches_roots_once_and_preserves_predecessor_values() {
    for count in [0, 1, 31, 62] {
        let contracts: Vec<_> = (0..count).map(|n| contract(format!("k{n:04}"))).collect();
        let state =
            AuthoritativeRecordAspectState::from_validated_entries(contracts.iter().map(admitted))
                .unwrap();
        let added = contract("z-new".into());
        let mut sets = vec![];
        if let Some(first) = contracts.first() {
            sets.push(admitted(first));
        }
        sets.push(admitted(&added));
        let patch = patch(sets, vec![]);
        let bound = patch
            .application_retained_storage_bounds(Some(&state))
            .unwrap();
        let (result, peak) = crate::allocation_observation::peak(|| patch.apply_to(&state));
        let TransitionOutcome::Success(result) = result else {
            panic!("actual leaf edit")
        };
        let result = result.into_parts().into_parts().0;
        assert!(peak <= bound.retained_bytes + bound.transient_bytes);
        assert!(result.get(added.key()).is_some());
        assert!(state.get(added.key()).is_none());
        if let Some(unchanged) = contracts.get(1) {
            let (
                ContractValidatedAspectValueView::Scalar(before),
                ContractValidatedAspectValueView::Scalar(after),
            ) = (
                state.get(unchanged.key()).unwrap().view(),
                result.get(unchanged.key()).unwrap().view(),
            )
            else {
                panic!("the actual untouched admitted values are scalars")
            };
            assert!(std::ptr::eq(before, after));
        }
        drop(state);
        drop(patch);
        assert!(result.get(added.key()).is_some());
    }
}

#[test]
fn retained_history_at_sixty_three_and_sixty_four_entries_keeps_general_path_quote() {
    let contracts: Vec<_> = (0..65).map(|n| contract(format!("k{n:04}"))).collect();
    let values: Vec<_> = contracts.iter().map(admitted).collect();
    for count in [63, 64] {
        let ((direct, direct_bytes), _) = crate::allocation_observation::peak(|| {
            let direct = AuthoritativeRecordAspectState::from_validated_entries(
                values.iter().skip(1).take(count).cloned(),
            )
            .unwrap();
            let bytes = crate::allocation_observation::live_bytes();
            (direct, bytes)
        });
        let clears = if count == 63 {
            vec![contracts[0].clone(), contracts[64].clone()]
        } else {
            vec![contracts[0].clone()]
        };
        let clear = patch(vec![], clears);
        let ((historical, historical_bytes), _) = crate::allocation_observation::peak(|| {
            let previous =
                AuthoritativeRecordAspectState::from_validated_entries(values.iter().cloned())
                    .unwrap();
            let TransitionOutcome::Success(next) = clear.apply_to(&previous) else {
                panic!("actual removal after the first split")
            };
            let next = next.into_parts().into_parts().0;
            drop(previous);
            let bytes = crate::allocation_observation::live_bytes();
            (next, bytes)
        });
        assert_eq!(direct, historical);
        assert_eq!(direct.aspects().len(), count);
        assert!(
            historical_bytes > direct_bytes,
            "equal cardinality need not mean equal node topology"
        );
        let added = contract("z-added".into());
        let edit = patch(vec![admitted(&added)], vec![]);
        let a = edit
            .application_retained_storage_bounds(Some(&direct))
            .unwrap();
        let b = edit
            .application_retained_storage_bounds(Some(&historical))
            .unwrap();
        assert_eq!(
            (a.retained_bytes, a.transient_bytes),
            (b.retained_bytes, b.transient_bytes)
        );
        assert!(
            a.transient_bytes > 0,
            "neither unknown topology takes the leaf shortcut"
        );
        let (result, peak) = crate::allocation_observation::peak(|| edit.apply_to(&historical));
        assert!(matches!(result, TransitionOutcome::Success(_)));
        assert!(peak <= b.retained_bytes + b.transient_bytes);
    }
}

#[test]
fn optional_empty_output_drops_roots_while_required_empty_state_retains_them() {
    let contract = contract("data".into());
    let state =
        AuthoritativeRecordAspectState::from_validated_entries([admitted(&contract)]).unwrap();
    let clear = patch(vec![], vec![contract]);
    let bound = clear
        .application_retained_storage_bounds(Some(&state))
        .unwrap();
    let ((optional, live), peak) = crate::allocation_observation::peak(|| {
        let TransitionOutcome::Success(optional) = clear.apply_to_optional(Some(&state)) else {
            panic!("actual optional application")
        };
        (optional, crate::allocation_observation::live_bytes())
    });
    assert!(optional.is_none());
    assert_eq!(live, 0);
    assert!(peak <= bound.retained_bytes + bound.transient_bytes);
    let ((required, live), peak) = crate::allocation_observation::peak(|| {
        let TransitionOutcome::Success(required) = clear.apply_to(&state) else {
            panic!("actual required application")
        };
        (required, crate::allocation_observation::live_bytes())
    });
    let required = required.into_parts().into_parts().0;
    assert!(required.aspects().is_empty());
    assert!(live > 0);
    assert!(live <= bound.retained_bytes);
    assert!(peak <= bound.retained_bytes + bound.transient_bytes);
    assert!(state.get(&AspectKey::new("data").unwrap()).is_some());
}

#[test]
fn streamed_state_constructor_keeps_first_duplicate_refusal_and_releases_roots() {
    let contract = contract("data".into());
    let value = admitted(&contract);
    let consumed = std::cell::Cell::new(0);
    let (_, peak) = crate::allocation_observation::peak(|| {
        let input = std::iter::from_fn(|| {
            let index = consumed.get();
            consumed.set(index + 1);
            assert!(index < 2, "the first duplicate must stop the lazy source");
            Some(value.clone())
        });
        match CanonicalAspectStateMap::from_validated_entries(input) {
            Err(AuthoritativeStateAdmissionDenial::DuplicateAspectKey(key)) => {
                assert_eq!(key.as_str(), "data");
            }
            _ => panic!("actual first-duplicate refusal"),
        }
        assert_eq!(crate::allocation_observation::live_bytes(), 0);
    });
    assert_eq!(consumed.get(), 2);
    assert!(
        peak > 0,
        "the real constructor allocated and released its roots"
    );
}
