#[allow(unused_imports)] use crate::certification::ordinary_context::canonicalization::digest_preparation::canonical_basis_builders::*;
use {serde_json::json};
use {worth_foundational::admit_authoritative_record_aspect_state, worth_foundational::lower_json_record_aspect_state, worth_foundational::prepare_aspect_contract_for_canonical_basis, worth_foundational::prepare_aspect_mask_for_canonical_basis, worth_foundational::prepare_aspect_patch_for_canonical_basis, worth_foundational::prepare_aspect_state_for_canonical_basis, worth_foundational::AspectContract, worth_foundational::AspectLocator, worth_foundational::AspectMask, worth_foundational::AspectValue, worth_foundational::AuthoritativeRecordAspectPatch, worth_foundational::BoundarySourceLocator, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalDecimal, worth_foundational::CanonicalFieldPath, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalizationRuleVersion, worth_foundational::JsonCompatibilityAspectInput, worth_foundational::LocatorAuthority, worth_foundational::ProjectionMask, worth_foundational::ScalarAspectType, worth_foundational::StructAspectValue};
use {worth_proof::TransitionOutcome};

use {crate::certification::ordinary_context::canonicalization::digest_preparation::readiness_fixtures::admitted_state, crate::certification::ordinary_context::canonicalization::digest_preparation::readiness_fixtures::task_summary_contract, crate::certification::ordinary_context::canonicalization::digest_preparation::readiness_fixtures::task_summary_contract_with_reversed_declaration_order};
use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::validated_scalar};

#[test]
fn contract_canonical_basis_is_ready_and_declaration_order_stable() {
    let left = match prepare_aspect_contract_for_canonical_basis(version(), task_summary_contract())
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("contract basis should be ready"),
    };
    let right = match prepare_aspect_contract_for_canonical_basis(
        version(),
        task_summary_contract_with_reversed_declaration_order(),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("contract basis should be ready"),
    };

    assert_eq!(
        left.payload().domain(),
        worth_foundational::CanonicalBasisDomain::AspectContract
    );
    assert_eq!(left.payload().entries(), right.payload().entries());
    assert!(left.payload().entries().iter().any(|entry| {
        entry.kind() == worth_foundational::CanonicalBasisEntryKind::Field
            && entry.locus()
                == &worth_foundational::CanonicalBasisLocus::Named(
                    "task.summary.field.done.value_type".into(),
                )
    }));
}

#[test]
fn mask_canonical_basis_preserves_mode_and_uses_ready_basis_artifact() {
    let mutation = AspectMask::<ProjectionMask>::new([
        worth_foundational::CanonicalFieldPath::single(field("zeta")),
        worth_foundational::CanonicalFieldPath::single(field("alpha")),
    ]);

    let ready =
        match prepare_aspect_mask_for_canonical_basis(version(), key("task.summary"), mutation) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("mask basis should be ready"),
        };

    assert_eq!(
        ready.payload().domain(),
        worth_foundational::CanonicalBasisDomain::AspectMask
    );
    assert_eq!(ready.payload().cost().entry_count(), 2);
    assert_eq!(
        ready.payload().entries()[0].locus(),
        &worth_foundational::CanonicalBasisLocus::Named(
            "task.summary.projection.field.alpha".into()
        )
    );
    assert_eq!(
        ready.basis().basis().value().as_str(),
        ready.payload().version().as_str()
    );
}

#[test]
fn state_canonical_basis_keeps_numeric_widths_distinct() {
    let signed = validated_scalar(
        "number",
        1,
        worth_foundational::ScalarAspectType::Int64,
        worth_foundational::AspectValue::Int64(1),
    );
    let unsigned = validated_scalar(
        "number",
        1,
        worth_foundational::ScalarAspectType::UInt64,
        worth_foundational::AspectValue::UInt64(1),
    );
    let signed_state = admitted_state([signed]);
    let unsigned_state = admitted_state([unsigned]);

    let signed_ready = match prepare_aspect_state_for_canonical_basis(version(), signed_state) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("signed state basis should be ready"),
    };
    let unsigned_ready = match prepare_aspect_state_for_canonical_basis(version(), unsigned_state) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("unsigned state basis should be ready"),
    };

    assert_ne!(
        signed_ready.payload().entries(),
        unsigned_ready.payload().entries()
    );
    assert!(signed_ready.payload().entries().iter().any(|entry| {
        entry.value()
            == &worth_foundational::CanonicalBasisValue::SignedInteger {
                width: worth_foundational::CanonicalIntegerWidth::Bits64,
                value: 1,
            }
    }));
    assert!(unsigned_ready.payload().entries().iter().any(|entry| {
        entry.value()
            == &worth_foundational::CanonicalBasisValue::UnsignedInteger {
                width: worth_foundational::CanonicalIntegerWidth::Bits64,
                value: 1,
            }
    }));
}

#[test]
fn state_canonical_basis_matches_compatibility_equivalent_struct_truth() {
    let contract = task_summary_contract();
    let value = worth_foundational::StructAspectValue::new([
        (
            field("title"),
            worth_foundational::AspectValue::String("Ship it".into()),
        ),
        (field("done"), worth_foundational::AspectValue::Bool(true)),
    ])
    .expect("unique fields");
    let worth_proof::TransitionOutcome::Success(entry) =
        worth_foundational::validate_aspect_value(&contract, value.into())
    else {
        panic!("expected validation");
    };
    let worth_proof::TransitionOutcome::Success(state) =
        admit_authoritative_record_aspect_state([entry])
    else {
        panic!("expected admitted state");
    };

    let ready = match prepare_aspect_state_for_canonical_basis(version(), state) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("state basis should be ready"),
    };
    let json_state =
        lower_json_record_aspect_state([worth_foundational::JsonCompatibilityAspectInput::new(
            contract,
            worth_foundational::BoundarySourceLocator::aspect(
                worth_foundational::AspectLocator::new(
                    worth_foundational::LocatorAuthority::SupportOnly,
                    key("task.summary"),
                ),
            ),
            json!({ "done": true, "title": "Ship it" }),
        )]);
    let worth_proof::TransitionOutcome::Success(json_state) = json_state else {
        panic!("expected compatibility lowering");
    };
    let json_ready = match prepare_aspect_state_for_canonical_basis(version(), json_state) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("json-lowered state basis should be ready"),
    };

    assert_eq!(ready.payload().entries(), json_ready.payload().entries());
    assert!(ready.payload().entries().iter().any(|entry| {
        entry.locus()
            == &worth_foundational::CanonicalBasisLocus::Named("task.summary.field.done".into())
            && entry.value() == &worth_foundational::CanonicalBasisValue::Bool(true)
    }));
    assert!(ready.payload().entries().iter().any(|entry| {
        entry.locus()
            == &worth_foundational::CanonicalBasisLocus::Named("task.summary.field.title".into())
            && entry.value()
                == &worth_foundational::CanonicalBasisValue::ExactText("Ship it".into())
    }));
}

#[test]
fn state_canonical_basis_preserves_adjacent_value_families_without_text_prefixes() {
    let decimal = validated_scalar(
        "amount",
        1,
        worth_foundational::ScalarAspectType::Decimal,
        worth_foundational::AspectValue::Decimal(worth_foundational::CanonicalDecimal::new("1.0")),
    );
    let text = validated_scalar(
        "amount",
        1,
        worth_foundational::ScalarAspectType::String,
        worth_foundational::AspectValue::String("decimal:1.0".into()),
    );

    let decimal_ready =
        match prepare_aspect_state_for_canonical_basis(version(), admitted_state([decimal])) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("decimal state basis should be ready"),
        };
    let text_ready =
        match prepare_aspect_state_for_canonical_basis(version(), admitted_state([text])) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("text state basis should be ready"),
        };

    assert_ne!(
        decimal_ready.payload().entries(),
        text_ready.payload().entries()
    );
    assert!(decimal_ready.payload().entries().iter().any(|entry| {
        entry.value() == &worth_foundational::CanonicalBasisValue::DecimalText("1.0".into())
    }));
    assert!(text_ready.payload().entries().iter().any(|entry| {
        entry.value() == &worth_foundational::CanonicalBasisValue::ExactText("decimal:1.0".into())
    }));
}

#[test]
fn patch_canonical_basis_is_ordered_and_distinguishes_clear_from_set() {
    let count = validated_scalar(
        "count",
        1,
        worth_foundational::ScalarAspectType::Int64,
        worth_foundational::AspectValue::Int64(2),
    );
    let name = validated_scalar(
        "name",
        2,
        worth_foundational::ScalarAspectType::String,
        worth_foundational::AspectValue::String("Ada".into()),
    );
    let done_contract = worth_foundational::AspectContract::scalar(
        key("done"),
        identity(3),
        revision(1),
        worth_foundational::ScalarAspectType::Bool,
    );
    let worth_proof::TransitionOutcome::Success(left_patch) =
        worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect(
            [count.clone(), name.clone()],
            [done_contract.clone()],
        )
    else {
        panic!("expected left patch");
    };
    let worth_proof::TransitionOutcome::Success(right_patch) =
        worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect(
            [name, count],
            [done_contract],
        )
    else {
        panic!("expected right patch");
    };

    let left = match prepare_aspect_patch_for_canonical_basis(version(), &left_patch) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("left patch basis should be ready"),
    };
    let right = match prepare_aspect_patch_for_canonical_basis(version(), &right_patch) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("right patch basis should be ready"),
    };

    assert_eq!(
        left.payload().domain(),
        worth_foundational::CanonicalBasisDomain::AuthoritativePatch
    );
    assert_eq!(left.payload().entries(), right.payload().entries());
    assert!(left.payload().entries().iter().any(|entry| {
        entry.locus() == &worth_foundational::CanonicalBasisLocus::Named("done.whole.clear".into())
            && entry.value() == &worth_foundational::CanonicalBasisValue::ExactText("clear".into())
    }));
    assert!(left.payload().entries().iter().any(|entry| {
        entry.locus()
            == &worth_foundational::CanonicalBasisLocus::Named("count.whole.set.value".into())
            && entry.value()
                == &worth_foundational::CanonicalBasisValue::SignedInteger {
                    width: worth_foundational::CanonicalIntegerWidth::Bits64,
                    value: 2,
                }
    }));
}

#[test]
fn empty_patch_canonical_basis_is_explicit_noop_evidence() {
    let ready = match prepare_aspect_patch_for_canonical_basis(
        version(),
        &worth_foundational::AuthoritativeRecordAspectPatch::empty(),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("empty patch basis should be ready"),
    };

    assert_eq!(
        ready.payload().domain(),
        worth_foundational::CanonicalBasisDomain::AuthoritativePatch
    );
    assert_eq!(ready.payload().cost().entry_count(), 1);
    assert!(ready.payload().entries().iter().any(|entry| {
        entry.locus() == &worth_foundational::CanonicalBasisLocus::Named("patch.noop".into())
            && entry.value() == &worth_foundational::CanonicalBasisValue::Null
    }));
}
