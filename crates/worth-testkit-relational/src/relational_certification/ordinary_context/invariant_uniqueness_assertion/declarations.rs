pub use {
    worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue,
    worth_foundational::facade::FieldKey, worth_foundational::facade::InternedString,
};
pub use {
    worth_relational::facade::transactions::planned_single_field_locator,
    worth_relational::facade::transactions::CommitConflict,
    worth_relational::facade::transactions::ConflictClass,
    worth_relational::facade::transactions::InvariantViolationFields,
    worth_relational::facade::transactions::TransactionCommitError,
};

pub fn assert_unique_conflict(error: TransactionCommitError, value: &str) {
    let worth_relational::facade::transactions::TransactionCommitError::Conflict { error, .. } =
        error
    else {
        panic!("expected invariant conflict, got {error:?}");
    };
    let CommitConflict { class, .. } = error;
    let worth_relational::facade::transactions::ConflictClass::InvariantViolation {
        fields:
            worth_relational::facade::transactions::InvariantViolationFields::UniqueEntityField {
                field_locator,
                value: observed,
            },
        ..
    } = class
    else {
        panic!("expected typed unique field conflict, got {class:?}");
    };
    let expected = planned_single_field_locator(
        worth_foundational::facade::AspectKey::new("call_sign").expect("call-sign aspect"),
        worth_foundational::facade::FieldKey::new("call_sign").expect("call-sign field"),
    );
    assert_eq!(field_locator, expected);
    assert_eq!(
        observed,
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::Raw(value.into())
        )
    );
}
