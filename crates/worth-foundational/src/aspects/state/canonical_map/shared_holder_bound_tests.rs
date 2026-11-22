//! Per-holder declared bounds and exact deduplicated allocation-visitor unions.
use crate::aspects::*;
use crate::values::{AspectValue, InternedString, ScalarAspectType};
use std::collections::BTreeMap;
use worth_proof::TransitionOutcome;

fn admitted(key: String) -> ContractValidatedAspectValue {
    let contract = AspectContract::scalar(
        AspectKey::new(key).unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::Bool,
    );
    let TransitionOutcome::Success(value) = validate_aspect_value(
        &contract,
        ContractValidationInput::Scalar(AspectValue::Bool(true)),
    ) else {
        panic!("canonical scalar")
    };
    value.into_parts().into_parts().0
}

#[derive(Default)]
struct UnionVisitor {
    allocations: BTreeMap<u64, u64>,
}

impl FoundationalAllocationVisitor for UnionVisitor {
    fn visit(&mut self, allocation: FoundationalAllocationObservation) -> bool {
        if let Some(previous) = self.allocations.insert(allocation.id, allocation.bytes) {
            assert_eq!(previous, allocation.bytes);
            false
        } else {
            true
        }
    }
}

#[test]
fn per_holder_bound_covers_backing_shared_by_two_holders_and_counts_it_twice() {
    for count in [0, 1, 31, 65, 127] {
        let ((first, second, live), _) = crate::allocation_observation::peak(|| {
            let first = AuthoritativeRecordAspectState::from_validated_entries(
                (0..count).map(|n| admitted(format!("key-{n:04}"))),
            )
            .unwrap();
            // The second holder shares the persistent roots and admitted values.
            let second = first.clone();
            let live = crate::allocation_observation::live_bytes();
            (first, second, live)
        });
        let bound = first.aspects().retained_storage_bytes().unwrap();
        assert_eq!(second.aspects().retained_storage_bytes(), Some(bound));
        if count == 0 {
            assert_eq!(live, 0);
            assert_eq!(bound, 0);
        } else {
            assert!(live > 0);
            assert!(
                live <= bound,
                "count={count}, observed shared backing={live}, holder bound={bound}"
            );
            assert!(live < bound.checked_mul(2).unwrap());
        }
        let mut union = UnionVisitor::default();
        first.visit_allocations(false, &mut union);
        second.visit_allocations(false, &mut union);
        let visited_union: u64 = union.allocations.values().sum();
        assert_eq!(
            visited_union, live as u64,
            "count={count}: visitor union must equal allocator-observed live bytes"
        );
        drop(first);
        assert_eq!(second.aspects().len(), count);
    }
}

#[test]
fn visitor_union_and_delta_match_allocator_observed_live_bytes_across_nested_structs() {
    let ((before, after, live_union), _) = crate::allocation_observation::peak(|| {
        let field = |name: &str| FieldKey::new(name).unwrap();
        let struct_contract = AspectContract::struct_aspect(
            AspectKey::new("profile").unwrap(),
            AspectIdentity(2),
            AspectContractRevision(1),
            StructAspectShape::new(["name", "bio", "tag"].map(|name| {
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
        let initial_struct = StructAspectValue::new([
            (
                field("name"),
                AspectValue::String(InternedString::Raw("alice".into())),
            ),
            (
                field("bio"),
                AspectValue::String(InternedString::Stored(
                    crate::facade::StoredValueText::new("persisted bio text".into()),
                )),
            ),
            (
                field("tag"),
                AspectValue::String(InternedString::Raw("v1".into())),
            ),
        ])
        .unwrap();
        let TransitionOutcome::Success(struct_val) = validate_aspect_value(
            &struct_contract,
            ContractValidationInput::Struct(initial_struct),
        ) else {
            panic!("valid struct")
        };
        let mut entries: Vec<_> = (0..16).map(|n| admitted(format!("s-{n:02}"))).collect();
        entries.push(struct_val.into_parts().into_parts().0);
        let before = AuthoritativeRecordAspectState::from_validated_entries(entries).unwrap();
        let mask = AspectMask::new([CanonicalFieldPath::single(field("tag"))]);
        let TransitionOutcome::Success(patch) = AuthoritativeRecordAspectPatch::field_level(
            &struct_contract,
            &mask,
            [(
                field("tag"),
                AspectValue::String(InternedString::Raw("v2-updated".into())),
            )],
            [],
        ) else {
            panic!("valid field patch")
        };
        let TransitionOutcome::Success(after) = patch.apply_to(&before) else {
            panic!("patch applies")
        };
        let after = after.into_parts().into_parts().0;
        drop(patch);
        drop(mask);
        drop(struct_contract);
        let live_union = crate::allocation_observation::live_bytes();
        (before, after, live_union)
    });

    let mut combined = UnionVisitor::default();
    before.visit_allocations(false, &mut combined);
    after.visit_allocations(false, &mut combined);
    assert_eq!(
        combined.allocations.values().sum::<u64>(),
        live_union as u64,
        "exact physical visitor union must equal allocator-observed bytes across shared persistent trees"
    );

    let mut before_set = UnionVisitor::default();
    before.visit_allocations(false, &mut before_set);
    let mut after_set = UnionVisitor::default();
    after.visit_allocations(false, &mut after_set);
    let mut expected_new = after_set.allocations.clone();
    expected_new.retain(|id, _| !before_set.allocations.contains_key(id));

    let mut delta = UnionVisitor::default();
    after.visit_new_allocations(Some(&before), false, &mut delta);
    assert_eq!(delta.allocations, expected_new);
    // Distinct heap allocations (key String buffers, Arcs, value buffers) must
    // each have their own physical identity rather than folding into one Arc row.
    assert!(
        combined.allocations.len() > before.aspects().len() + after.aspects().len(),
        "each independently allocated buffer/Arc must emit its own physical row"
    );
}

#[test]
fn stored_text_clone_replacement_and_rational_emit_distinct_physical_ids_and_skip_shared_seal() {
    use crate::value_shape::StoredValueText;
    use crate::values::{CanonicalBigInt, CanonicalRational};

    let text_field = FieldKey::new("note").unwrap();
    let ratio_field = FieldKey::new("ratio").unwrap();
    let contract = AspectContract::struct_aspect(
        AspectKey::new("doc").unwrap(),
        AspectIdentity(3),
        AspectContractRevision(1),
        StructAspectShape::new([
            FieldDeclaration::new(
                text_field.clone(),
                ScalarAspectType::String,
                FieldRequirement::Optional,
                AbsenceLaw::Optional,
                AspectEvolutionPolicy::AdditiveFieldsAllowed,
            )
            .unwrap(),
            FieldDeclaration::new(
                ratio_field.clone(),
                ScalarAspectType::Rational,
                FieldRequirement::Optional,
                AbsenceLaw::Optional,
                AspectEvolutionPolicy::AdditiveFieldsAllowed,
            )
            .unwrap(),
        ])
        .unwrap(),
    );
    let original_stored = StoredValueText::new("shared-seal-text".to_owned());
    let cloned_stored = original_stored.clone();
    let ratio = CanonicalRational::new(
        CanonicalBigInt::new("12345678901234567890"),
        CanonicalBigInt::new("98765432109876543210"),
    )
    .unwrap();
    let initial = StructAspectValue::new([
        (
            text_field.clone(),
            AspectValue::String(InternedString::Stored(original_stored)),
        ),
        (ratio_field, AspectValue::Rational(ratio)),
    ])
    .unwrap();
    let TransitionOutcome::Success(validated) =
        validate_aspect_value(&contract, ContractValidationInput::Struct(initial))
    else {
        panic!("valid struct")
    };
    let before = AuthoritativeRecordAspectState::from_validated_entries([
        validated.into_parts().into_parts().0,
    ])
    .unwrap();
    let mask = AspectMask::new([CanonicalFieldPath::single(text_field.clone())]);
    let TransitionOutcome::Success(patch) = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &mask,
        [(
            text_field,
            AspectValue::String(InternedString::Stored(cloned_stored)),
        )],
        [],
    ) else {
        panic!("valid patch")
    };
    let TransitionOutcome::Success(after) = patch.apply_to(&before) else {
        panic!("patch applies")
    };
    let after = after.into_parts().into_parts().0;

    let mut before_set = UnionVisitor::default();
    before.visit_allocations(false, &mut before_set);
    let mut after_set = UnionVisitor::default();
    after.visit_allocations(false, &mut after_set);
    let mut expected_new = after_set.allocations.clone();
    expected_new.retain(|id, _| !before_set.allocations.contains_key(id));

    let mut delta = UnionVisitor::default();
    after.visit_new_allocations(Some(&before), false, &mut delta);
    assert_eq!(
        delta.allocations, expected_new,
        "cloned StoredValueText replacement must skip the already-retained proof Arc"
    );
}
