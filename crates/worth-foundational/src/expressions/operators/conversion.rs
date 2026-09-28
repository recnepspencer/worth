//! Explicit numeric conversions.
//!
//! `exact_cast` either preserves the exact value or denies: a value outside
//! the target's range is an overflow, and a value the target cannot hold
//! exactly (a fraction for an integer, an inexact binary or decimal digit) is
//! a domain denial. `rounded_cast` rounds once, nearest with ties to even,
//! into a binary float. Decimals never pass through a binary float.

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};
use crate::expressions::numeric::{round_ratio, BinaryFormat, Natural};
use crate::expressions::types::IntegerType;

use super::decimal::{normalized, Decimal};
use super::integer::fit;

/// A numeric value in transit through a cast.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Number {
    Integer(i128),
    Float32(f32),
    Float64(f64),
    Decimal(Decimal),
}

/// A cast target; admission has already checked the pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    Integer(IntegerType),
    Float32,
    Float64,
    Decimal,
}

fn overflow(reason: &'static str) -> ExpressionDenial {
    ExpressionDenial::new(ExpressionDenialDetail::ArithmeticOverflow(reason))
}

fn domain(reason: &'static str) -> ExpressionDenial {
    ExpressionDenial::new(ExpressionDenialDetail::ArithmeticDomain(reason))
}

const INEXACT: &str = "the value is not exactly representable in the target type";
const TWO_127: f64 = 170_141_183_460_469_231_731_687_303_715_884_105_728.0;
const COEFFICIENT_LIMIT: u128 = 100_000_000_000_000_000_000_000_000_000_000_000_000;

pub(crate) fn exact_cast(value: Number, target: Target) -> ExpressionResult<Number> {
    Ok(match (value, target) {
        (Number::Integer(value), Target::Integer(integer)) => {
            Number::Integer(fit(integer, Some(value))?)
        }
        (Number::Float32(value), Target::Integer(integer)) => {
            Number::Integer(float_to_integer(f64::from(value), integer)?)
        }
        (Number::Float64(value), Target::Integer(integer)) => {
            Number::Integer(float_to_integer(value, integer)?)
        }
        (Number::Decimal((coefficient, scale)), Target::Integer(integer)) => {
            // Normalized decimals carry no trailing zeros, so any scale is a
            // fraction.
            if scale > 0 {
                return Err(domain("the decimal has a fractional part"));
            }
            Number::Integer(fit(integer, Some(coefficient))?)
        }
        (Number::Integer(value), Target::Float64) => {
            let float = value as f64;
            exact_integral(float, value)?;
            Number::Float64(float)
        }
        (Number::Integer(value), Target::Float32) => {
            let float = value as f32;
            exact_integral(f64::from(float), value)?;
            Number::Float32(float)
        }
        (Number::Float32(value), Target::Float64) => Number::Float64(f64::from(value)),
        (Number::Float64(value), Target::Float32) => {
            let narrowed = value as f32;
            if !narrowed.is_finite() {
                return Err(overflow("the value is outside the Float32 range"));
            }
            if f64::from(narrowed) != value {
                return Err(domain(INEXACT));
            }
            Number::Float32(narrowed + 0.0)
        }
        (Number::Float32(value), Target::Decimal) => {
            Number::Decimal(float_to_decimal(f64::from(value))?)
        }
        (Number::Float64(value), Target::Decimal) => Number::Decimal(float_to_decimal(value)?),
        (Number::Integer(value), Target::Decimal) => Number::Decimal(
            normalized(value, 0)
                .ok_or_else(|| overflow("the integer exceeds 38 decimal digits"))?,
        ),
        (Number::Decimal(decimal), Target::Float64) => {
            Number::Float64(decimal_to_float(decimal, BinaryFormat::BINARY64)?)
        }
        (Number::Decimal(decimal), Target::Float32) => {
            Number::Float32(decimal_to_float(decimal, BinaryFormat::BINARY32)? as f32)
        }
        _ => unreachable!("admission allows exact casts only between numeric types"),
    })
}

pub(crate) fn rounded_cast(value: Number, target: Target) -> ExpressionResult<Number> {
    // `as` from an integer or binary64 rounds to nearest, ties to even.
    let (float32, float64) = match (value, target) {
        (Number::Integer(value), Target::Float32) => (Some(value as f32), None),
        (Number::Integer(value), Target::Float64) => (None, Some(value as f64)),
        (Number::Float64(value), Target::Float32) => (Some(value as f32), None),
        (Number::Decimal(decimal), Target::Float32) => (
            Some(round_decimal(decimal, BinaryFormat::BINARY32)? as f32),
            None,
        ),
        (Number::Decimal(decimal), Target::Float64) => {
            (None, Some(round_decimal(decimal, BinaryFormat::BINARY64)?))
        }
        _ => unreachable!("admission allows rounded casts only into binary floats"),
    };
    let finite = float32.is_none_or(f32::is_finite) && float64.is_none_or(f64::is_finite);
    if !finite {
        return Err(overflow("the rounded value is outside the target range"));
    }
    Ok(match (float32, float64) {
        (Some(value), _) => Number::Float32(value + 0.0),
        (_, Some(value)) => Number::Float64(value + 0.0),
        _ => unreachable!("one target is chosen"),
    })
}

/// The unsigned value of a bus as a declared-width unsigned integer.
pub(crate) fn bits_to_integer(plane: &[u64], integer: IntegerType) -> ExpressionResult<i128> {
    let value = super::digital::to_unsigned(plane).and_then(|value| i128::try_from(value).ok());
    fit(integer, value)
}

/// A finite float's value as an integer when it is integral and below 2^127
/// in magnitude.
fn float_integral(value: f64) -> Option<i128> {
    (value.fract() == 0.0 && (-TWO_127..TWO_127).contains(&value)).then_some(value as i128)
}

fn float_to_integer(value: f64, integer: IntegerType) -> ExpressionResult<i128> {
    if value.fract() != 0.0 {
        return Err(domain("the float has a fractional part"));
    }
    fit(integer, float_integral(value))
}

/// Denies unless `float` holds exactly `value`.
fn exact_integral(float: f64, value: i128) -> ExpressionResult<()> {
    if !float.is_finite() {
        return Err(overflow("the value is outside the target range"));
    }
    match float_integral(float) {
        Some(exact) if exact == value => Ok(()),
        _ => Err(domain(INEXACT)),
    }
}

/// A binary float as a decimal: `m * 2^-k` is `m * 5^k * 10^-k`, exact at
/// scale `k` when `k <= 18`.
fn float_to_decimal(value: f64) -> ExpressionResult<Decimal> {
    if value == 0.0 {
        return Ok((0, 0));
    }
    let (mantissa, exponent) = odd_decompose(value.abs());
    let magnitude = if exponent >= 0 {
        let exponent = exponent as u32;
        if 64 - mantissa.leading_zeros() + exponent > 127 {
            return Err(overflow("the float exceeds 38 decimal digits"));
        }
        (u128::from(mantissa) << exponent, 0)
    } else {
        let scale = exponent.unsigned_abs();
        if scale > 18 {
            return Err(domain("the float needs more than 18 decimal places"));
        }
        // An odd mantissa times a power of five is odd: already normalized.
        (u128::from(mantissa) * 5_u128.pow(scale), scale as u8)
    };
    if magnitude.0 >= COEFFICIENT_LIMIT {
        return Err(overflow("the float exceeds 38 decimal digits"));
    }
    let coefficient = magnitude.0 as i128;
    let coefficient = if value < 0.0 {
        -coefficient
    } else {
        coefficient
    };
    Ok(normalized(coefficient, magnitude.1).expect("coefficient and scale are in range"))
}

/// `value = mantissa * 2^exponent` with an odd mantissa, for a positive finite
/// float, subnormals included.
pub(crate) fn odd_decompose(value: f64) -> (u64, i32) {
    let bits = value.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1 << 52) - 1);
    let (mantissa, exponent) = match biased {
        0 => (fraction, -1074),
        _ => (fraction | (1 << 52), biased - 1075),
    };
    let zeros = mantissa.trailing_zeros();
    (mantissa >> zeros, exponent + zeros as i32)
}

/// `coefficient * 10^-scale` exactly in `format`, or a domain denial.
fn decimal_to_float((coefficient, scale): Decimal, format: BinaryFormat) -> ExpressionResult<f64> {
    let rounded = round_decimal((coefficient, scale), format)?;
    // Exactness: `rounded * 10^scale == coefficient`, checked in integers.
    let (mantissa, exponent) = if rounded == 0.0 {
        (0, 0)
    } else {
        odd_decompose(rounded.abs())
    };
    let power = Natural::from_u128(10).pow(u32::from(scale));
    let mut left = Natural::from_u128(u128::from(mantissa)).mul(&power);
    let mut right = Natural::from_u128(coefficient.unsigned_abs());
    if exponent >= 0 {
        left = left.shl(exponent as u64);
    } else {
        right = right.shl(u64::from(exponent.unsigned_abs()));
    }
    if left != right {
        return Err(domain(INEXACT));
    }
    Ok(rounded)
}

/// `coefficient * 10^-scale` rounded once into `format`.
fn round_decimal((coefficient, scale): Decimal, format: BinaryFormat) -> ExpressionResult<f64> {
    let numerator = Natural::from_u128(coefficient.unsigned_abs());
    let denominator = Natural::from_u128(10).pow(u32::from(scale));
    let magnitude = round_ratio(&numerator, &denominator, format)
        .ok_or_else(|| overflow("the value is outside the target range"))?;
    Ok(if coefficient < 0 {
        -magnitude + 0.0
    } else {
        magnitude
    })
}

#[cfg(test)]
#[path = "conversion_tests.rs"]
mod tests;
