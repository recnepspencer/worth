//! Independent allocator observations across persistent-tree topology changes.
use crate::aspects::*;
use crate::values::{AspectValue, ScalarAspectType};
use worth_proof::TransitionOutcome;

fn contract(key: String) -> AspectContract {
    AspectContract::scalar(
        AspectKey::new(key).unwrap(),
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
        panic!("canonical scalar")
    };
    value.into_parts().into_parts().0
}

#[test]
fn retained_storage_and_delta_scratch_cover_observed_splits_merges_and_long_keys() {
    for count in [0, 1, 31, 32, 63, 64, 65, 127, 256] {
        let contracts: Vec<_> = (0..count).map(|n| contract(format!("k{n:04}"))).collect();
        let state =
            AuthoritativeRecordAspectState::from_validated_entries(contracts.iter().map(admitted))
                .unwrap();
        let added = contract(format!("z{}", "x".repeat(4096)));
        let mut sets = vec![admitted(&added)];
        if let Some(first) = contracts.first() {
            sets.push(admitted(first));
        }
        sets.sort_by(|a, b| a.key().cmp(b.key()));
        let clears: Vec<_> = contracts.iter().skip(1).step_by(2).cloned().collect();
        let patch = AuthoritativeRecordAspectPatch {
            whole_aspect_sets: sets.into_boxed_slice(),
            whole_aspect_clears: clears.into_boxed_slice(),
            field_patches: Box::new([]),
        };
        let bound = patch
            .application_retained_storage_bounds(Some(&state))
            .unwrap();
        let (result, peak) = crate::allocation_observation::peak(|| patch.apply_to(&state));
        let TransitionOutcome::Success(result) = result else {
            panic!("valid delta")
        };
        let result = result.into_parts().into_parts().0;
        assert!(
            peak <= bound.retained_bytes + bound.transient_bytes,
            "count={count}, actual={peak}, bound={bound:?}"
        );
        assert!(result.aspects().retained_storage_bytes().unwrap() <= bound.retained_bytes);
        // The successor remains meaningful after every predecessor and input drops.
        drop(state);
        drop(patch);
        assert!(result.get(added.key()).is_some());
        let clear = AuthoritativeRecordAspectPatch {
            whole_aspect_sets: Box::new([]),
            whole_aspect_clears: vec![added.clone()].into_boxed_slice(),
            field_patches: Box::new([]),
        };
        let bound = clear
            .application_retained_storage_bounds(Some(&result))
            .unwrap();
        let (next, peak) = crate::allocation_observation::peak(|| clear.apply_to(&result));
        assert!(peak <= bound.retained_bytes + bound.transient_bytes);
        let TransitionOutcome::Success(next) = next else {
            panic!("valid deletion")
        };
        let next = next.into_parts().into_parts().0;
        assert!(next.get(added.key()).is_none());
        assert_eq!(
            next.aspects().maximum_key_width(),
            if next.aspects().is_empty() { 0 } else { 5 }
        );
    }
}

#[test]
fn sorted_field_and_duplicate_decode_bounds_cover_actual_tree_allocation() {
    for count in [0, 1, 3, 31, 64, 127] {
        let fields: Box<[_]> = (0..count)
            .map(|n| {
                (
                    FieldKey::new(format!("f{n:04}")).unwrap(),
                    AspectValue::UInt64(n as u64),
                )
            })
            .collect();
        let bound = StructAspectValue::sorted_construction_storage_bytes(count).unwrap();
        let (value, peak) =
            crate::allocation_observation::peak(|| StructAspectValue::from_sorted_fields(fields));
        let value = value.unwrap();
        assert_eq!(peak, bound, "one exact Node/key/value Arc per moved field");
        assert_eq!(value.len(), count);
    }
    let wire = r#"{"fields":{"a":{"UInt64":1},"a":{"UInt64":2},"z":{"Bool":true}}}"#;
    let bound = StructAspectValue::decode_storage_bounds(3).unwrap();
    let (decoded, peak) =
        crate::allocation_observation::peak(|| serde_json::from_str::<StructAspectValue>(wire));
    let decoded = decoded.unwrap();
    // Three one-byte decoded keys are the only nested heap allocations.
    assert!(peak <= bound.retained_fields_bytes + bound.transient_decode_bytes + 3);
    assert_eq!(
        decoded.get(&FieldKey::new("a").unwrap()),
        Some(&AspectValue::UInt64(2))
    );
    assert_eq!(decoded.len(), 2);
}

#[test]
fn independent_successor_retention_covers_observed_backing_after_predecessor_drop() {
    for count in [0, 1, 31, 65, 127] {
        let ((successor, retained), _) = crate::allocation_observation::peak(|| {
            let predecessor = AuthoritativeRecordAspectState::from_validated_entries(
                (0..count).map(|n| admitted(&contract(format!("key-{n:04}")))),
            )
            .unwrap();
            let successor = predecessor.clone();
            drop(predecessor);
            let retained = crate::allocation_observation::live_bytes();
            (successor, retained)
        });
        let bound = AuthoritativeRecordAspectPatch::empty()
            .application_retained_storage_bounds(Some(&successor))
            .unwrap();
        assert!(retained > 0, "even an empty state owns persistent roots");
        assert!(
            retained <= bound.retained_bytes,
            "count={count}, observed retained={retained}, bound={bound:?}"
        );
    }
}
