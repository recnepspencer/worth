//! Typed numeric operations over evaluation values. Admission fixed every
//! operand type, so each operation dispatches on its node's type alone.

use crate::expressions::denial::ExpressionResult;
use crate::expressions::functions::Builtin;
use crate::expressions::operators::conversion::{
    bits_to_integer, exact_cast, rounded_cast, Number, Target,
};
use crate::expressions::operators::{decimal, float, integer};
use crate::expressions::syntax::ast::BinaryOp;
use crate::expressions::types::ExpressionType;

use super::value::{ExpressionValue, Repr};

pub(super) fn float32(value: f32) -> ExpressionValue {
    ExpressionValue(Repr::Float32(value.to_bits()))
}

pub(super) fn float64(value: f64) -> ExpressionValue {
    ExpressionValue(Repr::Float64(value.to_bits()))
}

pub(super) fn quantity(value: f64) -> ExpressionValue {
    ExpressionValue(Repr::Quantity(value.to_bits()))
}

pub(super) fn decimal_value((coefficient, scale): decimal::Decimal) -> ExpressionValue {
    ExpressionValue(Repr::Decimal { coefficient, scale })
}

pub(super) fn integer_of(value: &ExpressionValue) -> i128 {
    value
        .as_integer()
        .expect("admission typed this operand as an integer")
}

pub(super) fn decimal_of(value: &ExpressionValue) -> decimal::Decimal {
    value
        .as_decimal()
        .expect("admission typed this operand as a decimal")
}

fn float32_of(value: &ExpressionValue) -> f32 {
    value
        .as_f32()
        .expect("admission typed this operand as Float32")
}

/// A Float64 value or a quantity's canonical magnitude.
pub(super) fn magnitude_of(value: &ExpressionValue) -> f64 {
    value
        .as_f64()
        .or_else(|| value.as_quantity())
        .expect("admission typed this operand as Float64 or a quantity")
}

/// The additive identity `sum` folds from.
pub(super) fn zero(ty: &ExpressionType) -> ExpressionValue {
    match ty {
        ExpressionType::Integer(_) => ExpressionValue::integer(0),
        ExpressionType::Float32 => float32(0.0),
        ExpressionType::Float64 => float64(0.0),
        ExpressionType::Decimal => decimal_value((0, 0)),
        ExpressionType::Quantity(_) => quantity(0.0),
        _ => unreachable!("admission sums only numeric lists"),
    }
}

/// `a op b` for an arithmetic operator whose result has type `ty`.
pub(super) fn arithmetic(
    op: BinaryOp,
    ty: &ExpressionType,
    a: &ExpressionValue,
    b: &ExpressionValue,
) -> ExpressionResult<ExpressionValue> {
    Ok(match ty {
        ExpressionType::Integer(width) => {
            let value = integer::arithmetic(op, *width, integer_of(a), integer_of(b))?;
            ExpressionValue::integer(value)
        }
        ExpressionType::Float32 => float32(float::arithmetic(op, float32_of(a), float32_of(b))?),
        ExpressionType::Float64 => {
            float64(float::arithmetic(op, magnitude_of(a), magnitude_of(b))?)
        }
        ExpressionType::Quantity(_) => {
            quantity(float::arithmetic(op, magnitude_of(a), magnitude_of(b))?)
        }
        ExpressionType::Decimal => {
            let (a, b) = (decimal_of(a), decimal_of(b));
            decimal_value(match op {
                BinaryOp::Add => decimal::add(a, b)?,
                BinaryOp::Subtract => decimal::subtract(a, b)?,
                BinaryOp::Multiply => decimal::multiply(a, b)?,
                _ => unreachable!("admission gives decimals no other arithmetic operator"),
            })
        }
        _ => unreachable!("admission types arithmetic only over numbers"),
    })
}

pub(super) fn negate(
    ty: &ExpressionType,
    value: &ExpressionValue,
) -> ExpressionResult<ExpressionValue> {
    Ok(match ty {
        ExpressionType::Integer(width) => {
            ExpressionValue::integer(integer::negate(*width, integer_of(value))?)
        }
        ExpressionType::Float32 => float32(float::negate(float32_of(value))),
        ExpressionType::Float64 => float64(float::negate(magnitude_of(value))),
        ExpressionType::Quantity(_) => quantity(float::negate(magnitude_of(value))),
        ExpressionType::Decimal => decimal_value(decimal::negate(decimal_of(value))),
        _ => unreachable!("admission negates only signed numbers"),
    })
}

pub(super) fn abs(
    ty: &ExpressionType,
    value: &ExpressionValue,
) -> ExpressionResult<ExpressionValue> {
    Ok(match ty {
        ExpressionType::Integer(width) => {
            ExpressionValue::integer(integer::abs(*width, integer_of(value))?)
        }
        ExpressionType::Float32 => float32(float::abs(float32_of(value))),
        ExpressionType::Float64 => float64(float::abs(magnitude_of(value))),
        ExpressionType::Quantity(_) => quantity(float::abs(magnitude_of(value))),
        ExpressionType::Decimal => decimal_value(decimal::abs(decimal_of(value))),
        _ => unreachable!("admission takes abs only of signed numbers"),
    })
}

pub(super) fn sqrt(
    ty: &ExpressionType,
    value: &ExpressionValue,
) -> ExpressionResult<ExpressionValue> {
    Ok(match ty {
        ExpressionType::Float32 => float32(float::sqrt(float32_of(value))?),
        ExpressionType::Float64 => float64(float::sqrt(magnitude_of(value))?),
        ExpressionType::Quantity(_) => quantity(float::sqrt(magnitude_of(value))?),
        _ => unreachable!("admission takes square roots only of floats and quantities"),
    })
}

fn number(value: &ExpressionValue) -> Number {
    match &value.0 {
        Repr::Integer(value) => Number::Integer(*value),
        Repr::Float32(bits) => Number::Float32(f32::from_bits(*bits)),
        Repr::Float64(bits) => Number::Float64(f64::from_bits(*bits)),
        Repr::Decimal { coefficient, scale } => Number::Decimal((*coefficient, *scale)),
        _ => unreachable!("admission casts only numbers"),
    }
}

fn target(ty: &ExpressionType) -> Target {
    match ty {
        ExpressionType::Integer(width) => Target::Integer(*width),
        ExpressionType::Float32 => Target::Float32,
        ExpressionType::Float64 => Target::Float64,
        ExpressionType::Decimal => Target::Decimal,
        _ => unreachable!("admission casts only into numbers"),
    }
}

/// `exact_cast` or `rounded_cast` of `value` into `ty`.
pub(super) fn cast(
    builtin: Builtin,
    ty: &ExpressionType,
    value: &ExpressionValue,
) -> ExpressionResult<ExpressionValue> {
    if let (Some((_, limbs)), ExpressionType::Integer(width)) = (value.as_bits(), ty) {
        return Ok(ExpressionValue::integer(bits_to_integer(limbs, *width)?));
    }
    let converted = if builtin == Builtin::RoundedCast {
        rounded_cast(number(value), target(ty))?
    } else {
        exact_cast(number(value), target(ty))?
    };
    Ok(match converted {
        Number::Integer(value) => ExpressionValue::integer(value),
        Number::Float32(value) => float32(value),
        Number::Float64(value) => float64(value),
        Number::Decimal(value) => decimal_value(value),
    })
}
