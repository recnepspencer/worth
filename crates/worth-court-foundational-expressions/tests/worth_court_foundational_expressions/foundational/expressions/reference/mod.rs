// An independent slow evaluator for the Int64/Bool subset. Seeded trees
// render to source; the production evaluator must agree with a direct
// recursive reading of the language rules on every value and denial family.
// Integer arithmetic here runs in `i128` and checks the Int64 range after
// each operation, so it shares no code or overflow strategy with the crate.
#[allow(unused_imports)]
use crate::expressions::ordinary_context::reference::*;
pub(crate) use {
    worth_foundational::expression_api::ExpressionDenialFamily as Family,
    worth_foundational::expression_api::ExpressionValue,
};

pub(crate) use crate::expressions::ordinary_context::evaluate;

#[test]
fn reference_boundaries_are_pinned() {
    let min = i64::MIN;
    let cases = [
        (
            format!("({min}) / (-1)"),
            Err(worth_foundational::expression_api::ExpressionDenialFamily::ArithmeticOverflow),
        ),
        (
            format!("({min}) % (-1)"),
            Ok(worth_foundational::expression_api::ExpressionValue::integer(0)),
        ),
        (
            format!("-({min})"),
            Err(worth_foundational::expression_api::ExpressionDenialFamily::ArithmeticOverflow),
        ),
        (
            "(-7) % 2".to_string(),
            Ok(worth_foundational::expression_api::ExpressionValue::integer(-1)),
        ),
        (
            "7 % (-2)".to_string(),
            Ok(worth_foundational::expression_api::ExpressionValue::integer(1)),
        ),
        (
            "(-7) / 2".to_string(),
            Ok(worth_foundational::expression_api::ExpressionValue::integer(-3)),
        ),
        (
            "false && (1 / 0 == 0)".to_string(),
            Ok(worth_foundational::expression_api::ExpressionValue::bool(
                false,
            )),
        ),
        (
            "(1 / 0 == 0) && false".to_string(),
            Err(worth_foundational::expression_api::ExpressionDenialFamily::DivisionByZero),
        ),
        (
            "(1 / 0) + (9223372036854775807 + 1)".to_string(),
            Err(worth_foundational::expression_api::ExpressionDenialFamily::DivisionByZero),
        ),
    ];
    for (source, expected) in cases {
        check(&source, expected);
    }
}
