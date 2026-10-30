// Evaluation semantics against fixed expected values. Every expected value
// is written out by hand from the language rules, never read back from the
// evaluator.
pub(crate) use {
    worth_foundational::expression_api::ExpressionDenialDetail,
    worth_foundational::expression_api::ExpressionDenialFamily as Family,
    worth_foundational::expression_api::ExpressionOccurrence,
    worth_foundational::expression_api::ExpressionValue,
};

pub(crate) use {
    crate::expressions::ordinary_context::evaluation_denial,
    crate::expressions::ordinary_context::float, crate::expressions::ordinary_context::meters,
    crate::expressions::ordinary_context::value,
};

pub(crate) fn int(value: i128) -> ExpressionValue {
    worth_foundational::expression_api::ExpressionValue::integer(value)
}

pub(crate) fn text(value: &str) -> ExpressionValue {
    worth_foundational::expression_api::ExpressionValue::string(value)
}

pub(crate) fn some(value: ExpressionValue) -> ExpressionValue {
    worth_foundational::expression_api::ExpressionValue::some(value)
}

/// A canonical decimal from its coefficient and scale.
pub(crate) fn decimal(coefficient: i128, scale: u8) -> ExpressionValue {
    worth_foundational::expression_api::ExpressionValue::decimal_parts(coefficient, scale)
        .expect("canonical fixture decimal")
}

pub(crate) fn ints(values: [i128; 3]) -> ExpressionValue {
    worth_foundational::expression_api::ExpressionValue::list(values.into_iter().map(int).collect())
}

pub(crate) fn family(source: &str) -> Family {
    evaluation_denial(source).family()
}
