use super::*;
use crate::aspects::*;
use crate::values::{AspectValue, ScalarAspectType};
use std::cell::Cell;

fn scalar(name: &str) -> AspectContract {
    AspectContract::scalar(
        AspectKey::new(name).unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::Bool,
    )
}

fn set(contract: &AspectContract, value: AspectValue) -> PortableAspectPatchOperation {
    PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
        value: ContractValidationInput::Scalar(value),
    }
}

#[test]
fn owned_readmission_keeps_original_error_order_and_checks_duplicate_before_lookup() {
    let a = scalar("a");
    let b = scalar("b");
    let calls = Cell::new(0);
    let lookup = |key: &AspectKey| {
        calls.set(calls.get() + 1);
        if key == a.key() {
            Some(a.clone())
        } else {
            Some(b.clone())
        }
    };
    let duplicate = PortableAspectPatchOperation::ClearWhole {
        basis: PortableAspectContractBasis::new(
            a.key().clone(),
            AspectIdentity(99),
            AspectContractRevision(99),
        ),
    };
    let outcome = readmit_portable_record_aspect_patch(
        PortableRecordAspectPatch::new([set(&a, AspectValue::Bool(true)), duplicate.clone()]),
        PortablePatchReadmissionPurpose::RecordMutation,
        &lookup,
    );
    assert!(
        matches!(outcome, TransitionOutcome::Denied(PortableAspectReadmissionDenial::DuplicateAspectOperation(key)) if key == *a.key())
    );
    assert_eq!(calls.get(), 1, "duplicate lookup must never occur");
    calls.set(0);
    let outcome = readmit_portable_record_aspect_patch(
        PortableRecordAspectPatch::new([
            set(&a, AspectValue::Bool(true)),
            set(&b, AspectValue::UInt64(3)),
            duplicate,
        ]),
        PortablePatchReadmissionPurpose::RecordMutation,
        &lookup,
    );
    assert!(
        matches!(outcome, TransitionOutcome::Denied(PortableAspectReadmissionDenial::ValueValidation { key, .. }) if key == *b.key())
    );
    assert_eq!(
        calls.get(),
        2,
        "an earlier validation denial wins over a later duplicate"
    );
}

#[test]
fn owned_readmission_sorts_disjoint_operations_once_and_applies_same_values() {
    let a = scalar("a");
    let z = scalar("z");
    let input = PortableRecordAspectPatch::new([
        set(&z, AspectValue::Bool(false)),
        set(&a, AspectValue::Bool(true)),
    ]);
    let bound = input.readmission_storage_bounds().unwrap();
    let lookup = |key: &AspectKey| {
        if key == a.key() {
            Some(a.clone())
        } else {
            Some(z.clone())
        }
    };
    let TransitionOutcome::Success(patch) = readmit_portable_record_aspect_patch(
        input,
        PortablePatchReadmissionPurpose::RecordCreation,
        &lookup,
    ) else {
        panic!("valid owned patch")
    };
    assert_eq!(
        patch
            .whole_aspect_sets()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        ["a", "z"]
    );
    assert_eq!(
        bound.retained_bytes,
        2 * std::mem::size_of::<ContractValidatedAspectValue>()
    );
    let application = patch.application_retained_storage_bounds(None).unwrap();
    let TransitionOutcome::Success(Some(state)) = patch.apply_to_optional(None) else {
        panic!("valid creation")
    };
    let state = state.into_parts().into_parts().0;
    assert_eq!(state.aspects().len(), 2);
    // c65d042399: authoritative tree roots differ from authored patch rosters.
    assert!(application.retained_bytes >= state.aspects().retained_storage_bytes().unwrap());
    assert!(matches!(
        state.get(a.key()).unwrap().view(),
        ContractValidatedAspectValueView::Scalar(AspectValue::Bool(true))
    ));
}
