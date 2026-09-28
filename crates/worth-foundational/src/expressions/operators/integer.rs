//! Declared-width integers. Every result is checked against its width;
//! nothing wraps. Division truncates toward zero and the remainder takes the
//! dividend's sign.

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};
use crate::expressions::syntax::ast::BinaryOp;
use crate::expressions::types::IntegerType;

pub(crate) fn fit(integer: IntegerType, value: Option<i128>) -> ExpressionResult<i128> {
    value
        .filter(|value| (integer.min()..=integer.max()).contains(value))
        .ok_or_else(|| {
            ExpressionDenial::new(ExpressionDenialDetail::ArithmeticOverflow(
                "integer result is outside its declared width",
            ))
        })
}

pub(crate) fn arithmetic(
    op: BinaryOp,
    integer: IntegerType,
    a: i128,
    b: i128,
) -> ExpressionResult<i128> {
    let value = match op {
        BinaryOp::Add => a.checked_add(b),
        BinaryOp::Subtract => a.checked_sub(b),
        BinaryOp::Multiply => a.checked_mul(b),
        BinaryOp::Divide | BinaryOp::Remainder if b == 0 => {
            return Err(ExpressionDenial::new(
                ExpressionDenialDetail::DivisionByZero,
            ));
        }
        BinaryOp::Divide => a.checked_div(b),
        BinaryOp::Remainder => a.checked_rem(b),
        _ => unreachable!("admission types only arithmetic operators here"),
    };
    fit(integer, value)
}

pub(crate) fn negate(integer: IntegerType, value: i128) -> ExpressionResult<i128> {
    fit(integer, value.checked_neg())
}

pub(crate) fn abs(integer: IntegerType, value: i128) -> ExpressionResult<i128> {
    fit(integer, value.checked_abs())
}

#[cfg(test)]
mod tests {
    use super::{abs, arithmetic, negate};
    use crate::expressions::syntax::ast::BinaryOp;
    use crate::expressions::types::IntegerType;

    #[test]
    fn integer_contracts_are_checked() {
        let int8 = IntegerType::Int8;
        assert_eq!(arithmetic(BinaryOp::Add, int8, 100, 27), Ok(127));
        assert!(arithmetic(BinaryOp::Add, int8, 100, 28).is_err());
        assert!(arithmetic(BinaryOp::Divide, int8, -128, -1).is_err());
        assert_eq!(arithmetic(BinaryOp::Remainder, int8, -128, -1), Ok(0));
        assert_eq!(arithmetic(BinaryOp::Divide, int8, -7, 2), Ok(-3));
        assert_eq!(arithmetic(BinaryOp::Remainder, int8, -7, 2), Ok(-1));
        assert!(arithmetic(BinaryOp::Remainder, int8, 1, 0).is_err());
        assert!(negate(int8, -128).is_err());
        assert!(abs(int8, -128).is_err());
        let u64 = IntegerType::UInt64;
        assert!(arithmetic(BinaryOp::Subtract, u64, 0, 1).is_err());
        let max = u64.max();
        assert!(arithmetic(BinaryOp::Multiply, u64, max, max).is_err());
    }
}
