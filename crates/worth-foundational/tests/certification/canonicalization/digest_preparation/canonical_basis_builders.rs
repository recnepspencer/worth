use serde_json::json;
use worth_foundational::{
    admit_authoritative_record_aspect_state, lower_json_record_aspect_state,
    prepare_aspect_contract_for_canonical_basis, prepare_aspect_mask_for_canonical_basis,
    prepare_aspect_patch_for_canonical_basis, prepare_aspect_state_for_canonical_basis,
    AspectContract, AspectLocator, AspectMask, AspectValue, AuthoritativeRecordAspectPatch,
    BoundarySourceLocator, CanonicalBasisDomain, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisValue, CanonicalDecimal, CanonicalFieldPath, CanonicalIntegerWidth,
    CanonicalizationRuleVersion, JsonCompatibilityAspectInput, LocatorAuthority, ProjectionMask,
    ScalarAspectType, StructAspectValue,
};
use worth_proof::TransitionOutcome;

use super::readiness_fixtures::{
    admitted_state, task_summary_contract, task_summary_contract_with_reversed_declaration_order,
};
use crate::certification::foundational_vocabulary::{
    field, identity, key, revision, validated_scalar,
};

fn version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new("m2.surface-basis").expect("valid version")
}

#[test]
fn contract_canonical_basis_is_ready_and_declaration_order_stable() {
    let left = match prepare_aspect_contract_for_canonical_basis(version(), task_summary_contract())
    {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("contract basis should be ready"),
    };
    let right = match prepare_aspect_contract_for_canonical_basis(
        version(),
        task_summary_contract_with_reversed_declaration_order(),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("contract basis should be ready"),
    };

    assert_eq!(
        left.payload().domain(),
        CanonicalBasisDomain::AspectContract
    );
    assert_eq!(left.payload().entries(), right.payload().entries());
    assert!(left.payload().entries().iter().any(|entry| {
        entry.kind() == CanonicalBasisEntryKind::Field
            && entry.locus()
                == &CanonicalBasisLocus::Named("task.summary.field.done.value_type".into())
    }));
}

#[test]
fn mask_canonical_basis_preserves_mode_and_uses_ready_basis_artifact() {
    let mutation = AspectMask::<ProjectionMask>::new([
        CanonicalFieldPath::single(field("zeta")),
        CanonicalFieldPath::single(field("alpha")),
    ]);

    let ready =
        match prepare_aspect_mask_for_canonical_basis(version(), key("task.summary"), mutation) {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("mask basis should be ready"),
        };

    assert_eq!(ready.payload().domain(), CanonicalBasisDomain::AspectMask);
    assert_eq!(ready.payload().cost().entry_count(), 2);
    assert_eq!(
        ready.payload().entries()[0].locus(),
        &CanonicalBasisLocus::Named("task.summary.projection.field.alpha".into())
    );
    assert_eq!(
        ready.basis().basis().value().as_str(),
        ready.payload().version().as_str()
    );
}

#[test]
fn state_canonical_basis_keeps_numeric_widths_distinct() {
    let signed = validated_scalar("number", 1, ScalarAspectType::Int64, AspectValue::Int64(1));
    let unsigned = validated_scalar(
        "number",
        1,
        ScalarAspectType::UInt64,
        AspectValue::UInt64(1),
    );
    let signed_state = admitted_state([signed]);
    let unsigned_state = admitted_state([unsigned]);

    let signed_ready = match prepare_aspect_state_for_canonical_basis(version(), signed_state) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("signed state basis should be ready"),
    };
    let unsigned_ready = match prepare_aspect_state_for_canonical_basis(version(), unsigned_state) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("unsigned state basis should be ready"),
    };

    assert_ne!(
        signed_ready.payload().entries(),
        unsigned_ready.payload().entries()
    );
    assert!(signed_ready.payload().entries().iter().any(|entry| {
        entry.value()
            == &CanonicalBasisValue::SignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: 1,
            }
    }));
    assert!(unsigned_ready.payload().entries().iter().any(|entry| {
        entry.value()
            == &CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: 1,
            }
    }));
}

#[test]
fn state_canonical_basis_matches_compatibility_equivalent_struct_truth() {
    let contract = task_summary_contract();
    let value = StructAspectValue::new([
        (field("title"), AspectValue::String("Ship it".into())),
        (field("done"), AspectValue::Bool(true)),
    ])
    .expect("unique fields");
    let TransitionOutcome::Success(entry) =
        worth_foundational::validate_aspect_value(&contract, value.into())
    else {
        panic!("expected validation");
    };
    let TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state([entry]) else {
        panic!("expected admitted state");
    };

    let ready = match prepare_aspect_state_for_canonical_basis(version(), state) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("state basis should be ready"),
    };
    let json_state = lower_json_record_aspect_state([JsonCompatibilityAspectInput::new(
        contract,
        BoundarySourceLocator::aspect(AspectLocator::new(
            LocatorAuthority::SupportOnly,
            key("task.summary"),
        )),
        json!({ "done": true, "title": "Ship it" }),
    )]);
    let TransitionOutcome::Success(json_state) = json_state else {
        panic!("expected compatibility lowering");
    };
    let json_ready = match prepare_aspect_state_for_canonical_basis(version(), json_state) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("json-lowered state basis should be ready"),
    };

    assert_eq!(ready.payload().entries(), json_ready.payload().entries());
    assert!(ready.payload().entries().iter().any(|entry| {
        entry.locus() == &CanonicalBasisLocus::Named("task.summary.field.done".into())
            && entry.value() == &CanonicalBasisValue::Bool(true)
    }));
    assert!(ready.payload().entries().iter().any(|entry| {
        entry.locus() == &CanonicalBasisLocus::Named("task.summary.field.title".into())
            && entry.value() == &CanonicalBasisValue::ExactText("Ship it".into())
    }));
}

#[test]
fn state_canonical_basis_preserves_adjacent_value_families_without_text_prefixes() {
    let decimal = validated_scalar(
        "amount",
        1,
        ScalarAspectType::Decimal,
        AspectValue::Decimal(CanonicalDecimal::new("1.0")),
    );
    let text = validated_scalar(
        "amount",
        1,
        ScalarAspectType::String,
        AspectValue::String("decimal:1.0".into()),
    );

    let decimal_ready =
        match prepare_aspect_state_for_canonical_basis(version(), admitted_state([decimal])) {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("decimal state basis should be ready"),
        };
    let text_ready =
        match prepare_aspect_state_for_canonical_basis(version(), admitted_state([text])) {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("text state basis should be ready"),
        };

    assert_ne!(
        decimal_ready.payload().entries(),
        text_ready.payload().entries()
    );
    assert!(decimal_ready
        .payload()
        .entries()
        .iter()
        .any(|entry| { entry.value() == &CanonicalBasisValue::DecimalText("1.0".into()) }));
    assert!(text_ready
        .payload()
        .entries()
        .iter()
        .any(|entry| { entry.value() == &CanonicalBasisValue::ExactText("decimal:1.0".into()) }));
}

#[test]
fn patch_canonical_basis_is_ordered_and_distinguishes_clear_from_set() {
    let count = validated_scalar("count", 1, ScalarAspectType::Int64, AspectValue::Int64(2));
    let name = validated_scalar(
        "name",
        2,
        ScalarAspectType::String,
        AspectValue::String("Ada".into()),
    );
    let done_contract = AspectContract::scalar(
        key("done"),
        identity(3),
        revision(1),
        ScalarAspectType::Bool,
    );
    let TransitionOutcome::Success(left_patch) = AuthoritativeRecordAspectPatch::whole_aspect(
        [count.clone(), name.clone()],
        [done_contract.clone()],
    ) else {
        panic!("expected left patch");
    };
    let TransitionOutcome::Success(right_patch) =
        AuthoritativeRecordAspectPatch::whole_aspect([name, count], [done_contract])
    else {
        panic!("expected right patch");
    };

    let left = match prepare_aspect_patch_for_canonical_basis(version(), &left_patch) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("left patch basis should be ready"),
    };
    let right = match prepare_aspect_patch_for_canonical_basis(version(), &right_patch) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("right patch basis should be ready"),
    };

    assert_eq!(
        left.payload().domain(),
        CanonicalBasisDomain::AuthoritativePatch
    );
    assert_eq!(left.payload().entries(), right.payload().entries());
    assert!(left.payload().entries().iter().any(|entry| {
        entry.locus() == &CanonicalBasisLocus::Named("done.whole.clear".into())
            && entry.value() == &CanonicalBasisValue::ExactText("clear".into())
    }));
    assert!(left.payload().entries().iter().any(|entry| {
        entry.locus() == &CanonicalBasisLocus::Named("count.whole.set.value".into())
            && entry.value()
                == &CanonicalBasisValue::SignedInteger {
                    width: CanonicalIntegerWidth::Bits64,
                    value: 2,
                }
    }));
}

#[test]
fn empty_patch_canonical_basis_is_explicit_noop_evidence() {
    let ready = match prepare_aspect_patch_for_canonical_basis(
        version(),
        &AuthoritativeRecordAspectPatch::empty(),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("empty patch basis should be ready"),
    };

    assert_eq!(
        ready.payload().domain(),
        CanonicalBasisDomain::AuthoritativePatch
    );
    assert_eq!(ready.payload().cost().entry_count(), 1);
    assert!(ready.payload().entries().iter().any(|entry| {
        entry.locus() == &CanonicalBasisLocus::Named("patch.noop".into())
            && entry.value() == &CanonicalBasisValue::Null
    }));
}
