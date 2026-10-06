//! Checked base-10 decimals: at most 38 significant digits and scale 0..=18.
//!
//! `+`, `-`, and `*` are exact or deny. Division and quantization round only
//! with an explicit mode, once. Binary floats never mediate decimal
//! arithmetic, comparison, or conversion.

use std::cmp::Ordering;

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};
use crate::expressions::numeric::Natural;

/// A normalized decimal: `coefficient * 10^-scale` with no redundant zeros.
pub(crate) type Decimal = (i128, u8);

/// Coefficients stay below `10^38`.
const COEFFICIENT_LIMIT: u128 = 100_000_000_000_000_000_000_000_000_000_000_000_000;
const MAX_SCALE: u32 = 18;

/// Explicit rounding, in `Rounding` variant declaration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rounding {
    NearestEven,
    TowardZero,
    TowardPositive,
    TowardNegative,
}

impl Rounding {
    pub(crate) fn from_variant(index: u32) -> Self {
        match index {
            0 => Self::NearestEven,
            1 => Self::TowardZero,
            2 => Self::TowardPositive,
            _ => Self::TowardNegative,
        }
    }
}

fn overflow() -> ExpressionDenial {
    ExpressionDenial::new(ExpressionDenialDetail::ArithmeticOverflow(
        "decimal result exceeds 38 digits or scale 18",
    ))
}

/// Strips redundant trailing zeros; `None` outside the digit or scale bound.
pub(crate) fn normalized(mut coefficient: i128, mut scale: u8) -> Option<Decimal> {
    if coefficient == 0 {
        return Some((0, 0));
    }
    while scale > 0 && coefficient % 10 == 0 {
        coefficient /= 10;
        scale -= 1;
    }
    (coefficient.unsigned_abs() < COEFFICIENT_LIMIT && u32::from(scale) <= MAX_SCALE)
        .then_some((coefficient, scale))
}

/// An exact signed magnitude at an implied decimal scale.
struct Wide {
    negative: bool,
    magnitude: Natural,
}

fn ten_pow(exponent: u32) -> Natural {
    Natural::from_u128(10).pow(exponent)
}

fn widen(coefficient: i128, by: u32) -> Wide {
    Wide {
        negative: coefficient < 0,
        magnitude: Natural::from_u128(coefficient.unsigned_abs()).mul(&ten_pow(by)),
    }
}

fn signed_add(a: Wide, b: Wide) -> Wide {
    if a.negative == b.negative {
        return Wide {
            negative: a.negative,
            magnitude: a.magnitude.add(&b.magnitude),
        };
    }
    let (mut larger, smaller) = if a.magnitude >= b.magnitude {
        (a, b)
    } else {
        (b, a)
    };
    larger.magnitude.sub_assign(&smaller.magnitude);
    larger
}

/// Strips trailing zeros from an exact result, then checks the bounds.
fn narrow(wide: Wide, mut scale: u32) -> ExpressionResult<Decimal> {
    let ten = Natural::from_u128(10);
    let mut magnitude = wide.magnitude;
    if magnitude.is_zero() {
        return Ok((0, 0));
    }
    while scale > 0 {
        let (quotient, remainder) = magnitude.divmod(&ten);
        if !remainder.is_zero() {
            break;
        }
        magnitude = quotient;
        scale -= 1;
    }
    let value = magnitude
        .to_u128()
        .filter(|value| *value < COEFFICIENT_LIMIT && scale <= MAX_SCALE)
        .ok_or_else(overflow)?;
    let value = value as i128;
    Ok((if wide.negative { -value } else { value }, scale as u8))
}

fn aligned((ca, sa): Decimal, (cb, sb): Decimal) -> (Wide, Wide, u32) {
    let scale = u32::from(sa.max(sb));
    (
        widen(ca, scale - u32::from(sa)),
        widen(cb, scale - u32::from(sb)),
        scale,
    )
}

pub(crate) fn add(a: Decimal, b: Decimal) -> ExpressionResult<Decimal> {
    let (a, b, scale) = aligned(a, b);
    narrow(signed_add(a, b), scale)
}

pub(crate) fn subtract(a: Decimal, (cb, sb): Decimal) -> ExpressionResult<Decimal> {
    add(a, (-cb, sb))
}

pub(crate) fn multiply((ca, sa): Decimal, (cb, sb): Decimal) -> ExpressionResult<Decimal> {
    let product = Wide {
        negative: (ca < 0) != (cb < 0),
        magnitude: Natural::from_u128(ca.unsigned_abs())
            .mul(&Natural::from_u128(cb.unsigned_abs())),
    };
    narrow(product, u32::from(sa) + u32::from(sb))
}

pub(crate) fn negate((coefficient, scale): Decimal) -> Decimal {
    (-coefficient, scale)
}

pub(crate) fn abs((coefficient, scale): Decimal) -> Decimal {
    (coefficient.abs(), scale)
}

/// Exact numeric order.
pub(crate) fn compare(a: Decimal, b: Decimal) -> Ordering {
    let (a, b, _) = aligned(a, b);
    let a_negative = a.negative && !a.magnitude.is_zero();
    let b_negative = b.negative && !b.magnitude.is_zero();
    match (a_negative, b_negative) {
        (false, true) => Ordering::Greater,
        (true, false) => Ordering::Less,
        (false, false) => a.magnitude.cmp(&b.magnitude),
        (true, true) => b.magnitude.cmp(&a.magnitude),
    }
}

fn check_scale(scale: i128) -> ExpressionResult<u32> {
    u32::try_from(scale)
        .ok()
        .filter(|scale| *scale <= MAX_SCALE)
        .ok_or_else(|| {
            ExpressionDenial::new(ExpressionDenialDetail::ArithmeticDomain(
                "decimal scale is outside 0..=18",
            ))
        })
}

/// `a / b` rounded once to `scale` digits by `mode`.
pub(crate) fn divide(
    (ca, sa): Decimal,
    (cb, sb): Decimal,
    scale: i128,
    mode: Rounding,
) -> ExpressionResult<Decimal> {
    let scale = check_scale(scale)?;
    if cb == 0 {
        return Err(ExpressionDenial::new(
            ExpressionDenialDetail::DivisionByZero,
        ));
    }
    let numerator = Natural::from_u128(ca.unsigned_abs()).mul(&ten_pow(u32::from(sb) + scale));
    let denominator = Natural::from_u128(cb.unsigned_abs()).mul(&ten_pow(u32::from(sa)));
    let negative = (ca < 0) != (cb < 0);
    let (quotient, remainder) = numerator.divmod(&denominator);
    let magnitude = round(quotient, &remainder, &denominator, negative, mode);
    narrow(
        Wide {
            negative,
            magnitude,
        },
        scale,
    )
}

/// `a` rounded once to at most `scale` digits by `mode`.
pub(crate) fn quantize(
    (ca, sa): Decimal,
    scale: i128,
    mode: Rounding,
) -> ExpressionResult<Decimal> {
    let scale = check_scale(scale)?;
    if u32::from(sa) <= scale {
        return Ok((ca, sa));
    }
    let divisor = ten_pow(u32::from(sa) - scale);
    let negative = ca < 0;
    let (quotient, remainder) = Natural::from_u128(ca.unsigned_abs()).divmod(&divisor);
    let magnitude = round(quotient, &remainder, &divisor, negative, mode);
    narrow(
        Wide {
            negative,
            magnitude,
        },
        scale,
    )
}

/// Rounds the magnitude `quotient + remainder / divisor` of a value whose
/// sign is `negative`.
fn round(
    quotient: Natural,
    remainder: &Natural,
    divisor: &Natural,
    negative: bool,
    mode: Rounding,
) -> Natural {
    if remainder.is_zero() {
        return quotient;
    }
    let away = match mode {
        Rounding::NearestEven => {
            let twice = remainder.shl(1);
            twice > *divisor || (twice == *divisor && quotient.is_odd())
        }
        Rounding::TowardZero => false,
        Rounding::TowardPositive => !negative,
        Rounding::TowardNegative => negative,
    };
    if away {
        quotient.add(&Natural::from_u128(1))
    } else {
        quotient
    }
}

#[cfg(test)]
mod tests {
    use super::{add, compare, divide, multiply, normalized, quantize, subtract, Rounding};
    use std::cmp::Ordering;

    #[test]
    fn arithmetic_is_exact_and_normalized() {
        assert_eq!(add((125, 2), (375, 2)), Ok((5, 0)));
        assert_eq!(subtract((1, 1), (1, 1)), Ok((0, 0)));
        assert_eq!(multiply((15, 1), (2, 0)), Ok((3, 0)));
        assert_eq!(normalized(1200, 2), Some((12, 0)));
        let big = 10_i128.pow(37) * 9;
        assert!(add((big, 0), (big, 0)).is_err(), "39 digits deny");
        assert!(multiply((1, 18), (1, 1)).is_err(), "scale 19 denies");
        assert_eq!(compare((-1, 0), (-5, 1)), Ordering::Less);
        assert_eq!(compare((5, 1), (50, 2)), Ordering::Equal);
    }

    #[test]
    fn division_and_quantization_round_once_by_mode() {
        let third = divide((1, 0), (3, 0), 4, Rounding::NearestEven);
        assert_eq!(third, Ok((3333, 4)));
        assert_eq!(
            divide((-2, 0), (3, 0), 0, Rounding::TowardNegative),
            Ok((-1, 0))
        );
        assert_eq!(quantize((25, 1), 0, Rounding::NearestEven), Ok((2, 0)));
        assert_eq!(quantize((35, 1), 0, Rounding::NearestEven), Ok((4, 0)));
        assert_eq!(quantize((-25, 1), 0, Rounding::TowardPositive), Ok((-2, 0)));
        assert_eq!(quantize((-25, 1), 0, Rounding::TowardZero), Ok((-2, 0)));
        assert!(divide((1, 0), (0, 0), 2, Rounding::TowardZero).is_err());
        assert!(quantize((1, 0), 19, Rounding::TowardZero).is_err());
    }
}
