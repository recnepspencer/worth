pub(crate) use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};
pub(crate) use worth_relational::facade::transactions::{
    planned_single_field_locator, CommitConflict, ConflictClass, InvariantViolationFields,
    TransactionCommitError,
};

pub(crate) fn assert_unique_conflict(error: TransactionCommitError, value: &str) {
    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("expected invariant conflict, got {error:?}");
    };
    let CommitConflict { class, .. } = error;
    let ConflictClass::InvariantViolation {
        fields:
            InvariantViolationFields::UniqueEntityField {
                field_locator,
                value: observed,
            },
        ..
    } = class
    else {
        panic!("expected typed unique field conflict, got {class:?}");
    };
    let expected = planned_single_field_locator(
        AspectKey::new("call_sign").expect("call-sign aspect"),
        FieldKey::new("call_sign").expect("call-sign field"),
    );
    assert_eq!(field_locator, expected);
    assert_eq!(
        observed,
        AspectValue::String(InternedString::Raw(value.into()))
    );
}
