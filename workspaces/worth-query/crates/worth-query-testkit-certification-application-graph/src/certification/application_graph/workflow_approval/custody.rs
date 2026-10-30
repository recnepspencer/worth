//! Already-published approval facts are immutable at the native commit boundary.

pub use super::*;
pub use worth_relational::facade::transactions::{
    ConflictClass, InvariantViolationFields, TransactionCommitError,
};

pub fn assert_fact_custody_denial(error: TransactionCommitError) {
    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("expected a native invariant conflict, got {error:?}");
    };
    let ConflictClass::InvariantViolation {
        fields:
            InvariantViolationFields::CustomInvariantViolation {
                identity,
                refusal: _,
            },
        ..
    } = error.class
    else {
        panic!("expected a custom invariant denial, got {error:?}");
    };
    assert_eq!(
        identity.rule_id.as_str(),
        "worth-query.workflow.publication-immutability"
    );
}
