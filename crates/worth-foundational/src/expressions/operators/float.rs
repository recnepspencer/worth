//! Finite IEEE binary floats and Float64-magnitude quantities.
//!
//! Each declared operation rounds once, nearest with ties to even, with
//! gradual underflow; nothing fuses, reassociates, or flushes to zero. NaN
//! and infinities never enter or leave: overflow and division by zero deny,
//! and every zero result is positive zero.

use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};
use crate::expressions::numeric::Natural;
use crate::expressions::syntax::ast::BinaryOp;

/// The binary float formats. Their basic operations and square root are
/// correctly rounded on every supported target.
pub(crate) trait Binary:
    Copy
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    const ZERO: Self;
    fn is_finite(self) -> bool;
    fn sqrt(self) -> Self;
}

impl Binary for f32 {
    const ZERO: Self = 0.0;
    fn is_finite(self) -> bool {
        f32::is_finite(self)
    }
    fn sqrt(self) -> Self {
        f32::sqrt(self)
    }
}

impl Binary for f64 {
    const ZERO: Self = 0.0;
    fn is_finite(self) -> bool {
        f64::is_finite(self)
    }
    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }
}

/// Denies a nonfinite result and normalizes zero to positive zero.
pub(crate) fn finish<F: Binary>(value: F) -> ExpressionResult<F> {
    if value.is_finite() {
        Ok(value + F::ZERO)
    } else {
        Err(ExpressionDenial::new(
            ExpressionDenialDetail::ArithmeticOverflow("float result is not finite"),
        ))
    }
}

pub(crate) fn arithmetic<F: Binary>(op: BinaryOp, a: F, b: F) -> ExpressionResult<F> {
    let value = match op {
        BinaryOp::Add => a + b,
        BinaryOp::Subtract => a - b,
        BinaryOp::Multiply => a * b,
        BinaryOp::Divide if b == F::ZERO => {
            return Err(ExpressionDenial::new(
                ExpressionDenialDetail::DivisionByZero,
            ));
        }
        BinaryOp::Divide => a / b,
        _ => unreachable!("admission gives floats no remainder"),
    };
    finish(value)
}

pub(crate) fn negate<F: Binary>(value: F) -> F {
    -value + F::ZERO
}

pub(crate) fn abs<F: Binary>(value: F) -> F {
    if value < F::ZERO {
        -value
    } else {
        value
    }
}

/// Correctly rounded square root; a negative operand denies.
pub(crate) fn sqrt<F: Binary>(value: F) -> ExpressionResult<F> {
    if value < F::ZERO {
        return Err(ExpressionDenial::new(
            ExpressionDenialDetail::ArithmeticDomain("sqrt of a negative value"),
        ));
    }
    finish(value.sqrt())
}

/// `|value| * 2^1074` exactly, with the sign.
fn exact(value: f64) -> (bool, Natural) {
    let bits = value.to_bits();
    let biased = (bits >> 52) & 0x7ff;
    let fraction = bits & ((1 << 52) - 1);
    let (mantissa, shift) = if biased == 0 {
        (fraction, 0)
    } else {
        (fraction | (1 << 52), biased - 1)
    };
    (
        bits >> 63 == 1,
        Natural::from_u128(u128::from(mantissa)).shl(shift),
    )
}

/// `|a - b| <= max(abs_tol, rel_tol * max(|a|, |b|))`, evaluated exactly on
/// the finite binary operands so no intermediate can overflow or round.
pub(crate) fn near(a: f64, b: f64, absolute: f64, relative: f64) -> ExpressionResult<bool> {
    if absolute < 0.0 || relative < 0.0 {
        return Err(ExpressionDenial::new(
            ExpressionDenialDetail::ArithmeticDomain("near tolerances are nonnegative"),
        ));
    }
    let (a_negative, a) = exact(a);
    let (b_negative, b) = exact(b);
    let difference = if a_negative == b_negative {
        let (mut larger, smaller) = if a >= b {
            (a.clone(), &b)
        } else {
            (b.clone(), &a)
        };
        larger.sub_assign(smaller);
        larger
    } else {
        a.add(&b)
    };
    let largest = if a >= b { a } else { b };
    // Both sides are scaled to 2^-2148 so products of two exact values fit.
    let left = difference.shl(1074);
    let absolute = exact(absolute).1.shl(1074);
    let relative = exact(relative).1.mul(&largest);
    Ok(left <= absolute.max(relative))
}

#[cfg(test)]
mod tests {
    use super::{arithmetic, near, negate, sqrt};
    use crate::expressions::syntax::ast::BinaryOp;

    #[test]
    fn float_contracts_deny_nonfinite_results() {
        assert_eq!(arithmetic(BinaryOp::Add, 0.1_f64, 0.2), Ok(0.1 + 0.2));
        assert!(arithmetic(BinaryOp::Divide, 0.0_f64, 0.0).is_err());
        assert!(arithmetic(BinaryOp::Multiply, f64::MAX, 2.0).is_err());
        let product = arithmetic(BinaryOp::Multiply, -0.0_f64, 1.0).unwrap();
        assert_eq!(product.to_bits(), 0.0_f64.to_bits());
        assert_eq!(negate(0.0_f64).to_bits(), 0.0_f64.to_bits());
        let tiny = arithmetic(BinaryOp::Divide, f64::MIN_POSITIVE, 4.0).unwrap();
        assert!(tiny > 0.0 && tiny < f64::MIN_POSITIVE, "gradual underflow");
        assert!(sqrt(-1.0_f64).is_err());
        assert_eq!(sqrt(2.0_f32), Ok(2.0_f32.sqrt()));
    }

    #[test]
    fn near_compares_exact_values_without_overflow() {
        assert_eq!(near(1.0, 1.0 + 1e-12, 1e-9, 0.0), Ok(true));
        assert_eq!(near(1.0, 1.1, 0.0, 0.01), Ok(false));
        assert_eq!(near(100.0, 101.0, 0.0, 0.01), Ok(true));
        // |MAX - -MAX| overflows binary64; the exact comparison does not.
        assert_eq!(near(f64::MAX, -f64::MAX, f64::MAX, 0.0), Ok(false));
        assert_eq!(near(f64::MAX, -f64::MAX, 0.0, 2.0), Ok(true));
        assert!(near(1.0, 1.0, -1.0, 0.0).is_err());
    }
}
