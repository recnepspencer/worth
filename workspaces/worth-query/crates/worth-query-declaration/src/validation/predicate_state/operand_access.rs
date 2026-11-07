use crate::authoring::WorthQueryPredicateOperand;
use crate::canonicalization::{
    CanonicalPredicateEntry, CanonicalPredicateOperand, CanonicalPredicateOperand::ScalarSet,
    CanonicalScalarSet,
};

pub(super) fn scalar_operand(predicate: &CanonicalPredicateEntry) -> &WorthQueryPredicateOperand {
    match &predicate.operand {
        CanonicalPredicateOperand::Scalar(value) => value,
        _ => unreachable!("scalar predicate expected scalar operand"),
    }
}

pub(super) fn comparison_scalar(
    predicate: &CanonicalPredicateEntry,
) -> &worth_foundational::facade::AspectValue {
    scalar_operand(predicate).as_native()
}

pub(super) fn string_scalar(predicate: &CanonicalPredicateEntry) -> &str {
    match scalar_operand(predicate).as_native() {
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::Stored(value),
        ) => value.as_str(),
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::Raw(value),
        ) => value,
        _ => unreachable!("string predicate expected string scalar"),
    }
}

pub(super) fn membership_values_set(predicate: &CanonicalPredicateEntry) -> &CanonicalScalarSet {
    match &predicate.operand {
        ScalarSet(values) => values,
        _ => unreachable!("membership predicate expected scalar set"),
    }
}

pub(super) fn membership_values(
    predicate: &CanonicalPredicateEntry,
) -> &[WorthQueryPredicateOperand] {
    membership_values_set(predicate).as_slice()
}
