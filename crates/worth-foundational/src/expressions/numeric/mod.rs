//! Exact numeric helpers: single nearest-even rounding of exact rationals.

mod natural;

pub(crate) use natural::Natural;

use super::syntax::literal::DecimalText;
use super::types::units::{UnitScale, DEGREE_IN_RADIANS};

/// Literal magnitudes past these bounds are denied rather than computed.
const MAX_LITERAL_DIGITS: usize = 800;
const MAX_LITERAL_EXPONENT: i32 = 2_000;
const MAX_DEGREE_POWER: u32 = 16;

/// Rounds `numerator / denominator` to binary64, nearest with ties to even,
/// with gradual underflow. `None` when the result overflows.
pub(crate) fn round_ratio(numerator: &Natural, denominator: &Natural) -> Option<f64> {
    if numerator.is_zero() {
        return Some(0.0);
    }
    let estimate = numerator.bit_len() as i64 - denominator.bit_len() as i64;
    let at_least_estimate = if estimate >= 0 {
        *numerator >= denominator.shl(estimate as u64)
    } else {
        numerator.shl(estimate.unsigned_abs()) >= *denominator
    };
    let exponent = if at_least_estimate {
        estimate
    } else {
        estimate - 1
    };
    if exponent > 1023 {
        return None;
    }
    let lsb = (exponent - 52).max(-1074);
    let (dividend, divisor) = if lsb < 0 {
        (numerator.shl(lsb.unsigned_abs()), denominator.clone())
    } else {
        (numerator.clone(), denominator.shl(lsb as u64))
    };
    let (mut quotient, remainder) = divide_small_quotient(dividend, &divisor);
    let twice = remainder.shl(1);
    if twice > divisor || (twice == divisor && quotient & 1 == 1) {
        quotient += 1;
    }
    let value = quotient as f64 * power_of_two(lsb as i32);
    value.is_finite().then_some(value)
}

/// Long division whose quotient is known to fit in 54 bits.
fn divide_small_quotient(mut remainder: Natural, divisor: &Natural) -> (u64, Natural) {
    let shift = remainder.bit_len() as i64 - divisor.bit_len() as i64;
    let mut quotient = 0_u64;
    for bit in (0..=shift.max(-1)).rev() {
        let shifted = divisor.shl(bit as u64);
        if remainder >= shifted {
            remainder.sub_assign(&shifted);
            quotient |= 1 << bit;
        }
    }
    (quotient, remainder)
}

/// `2^exponent` for `-1074 <= exponent <= 1023`.
fn power_of_two(exponent: i32) -> f64 {
    if exponent >= -1022 {
        f64::from_bits(((exponent + 1023) as u64) << 52)
    } else {
        f64::from_bits(1 << (exponent + 1074))
    }
}

/// The canonical SI magnitude of `±decimal` authored in a unit with `scale`,
/// rounded once. `None` when the literal is out of range or the result is
/// not finite.
pub(crate) fn scaled_decimal(
    decimal: &DecimalText,
    negative: bool,
    scale: UnitScale,
) -> Option<f64> {
    if decimal.digits().len() > MAX_LITERAL_DIGITS
        || decimal.exponent().unsigned_abs() > MAX_LITERAL_EXPONENT as u32
        || u32::from(scale.degree_power.unsigned_abs()) > MAX_DEGREE_POWER
    {
        return None;
    }
    let ten = Natural::from_u128(10);
    let exponent = decimal.exponent();
    let mut numerator = Natural::from_decimal(decimal.digits())?
        .mul(&ten.pow(exponent.max(0) as u32))
        .mul(&Natural::from_u128(scale.numerator));
    let mut denominator = ten
        .pow(exponent.min(0).unsigned_abs())
        .mul(&Natural::from_u128(scale.denominator));
    let (mantissa, binary_exponent) = decompose(DEGREE_IN_RADIANS);
    let power = u32::from(scale.degree_power.unsigned_abs());
    let mantissa = Natural::from_u128(u128::from(mantissa)).pow(power);
    let shift = u64::from(binary_exponent.unsigned_abs()) * u64::from(power);
    // The degree constant is below one, so its binary exponent is negative.
    if scale.degree_power > 0 {
        numerator = numerator.mul(&mantissa);
        denominator = denominator.shl(shift);
    } else {
        numerator = numerator.shl(shift);
        denominator = denominator.mul(&mantissa);
    }
    let magnitude = round_ratio(&numerator, &denominator)?;
    Some(if negative {
        -magnitude + 0.0
    } else {
        magnitude
    })
}

/// `value = mantissa * 2^exponent` for a positive normal binary64.
fn decompose(value: f64) -> (u64, i32) {
    let bits = value.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i32;
    ((bits & ((1 << 52) - 1)) | (1 << 52), biased - 1075)
}

#[cfg(test)]
mod tests {
    use super::{round_ratio, scaled_decimal, Natural};
    use crate::expressions::syntax::literal::DecimalText;
    use crate::expressions::types::units::{catalog_unit, DEGREE_IN_RADIANS};

    fn ratio(numerator: u128, denominator: u128) -> Option<f64> {
        round_ratio(
            &Natural::from_u128(numerator),
            &Natural::from_u128(denominator),
        )
    }

    #[test]
    fn rounds_ratios_nearest_even() {
        assert_eq!(ratio(1, 3), Some(1.0 / 3.0));
        assert_eq!(ratio(2, 3), Some(2.0 / 3.0));
        assert_eq!(ratio(1, 10), Some(0.1));
        assert_eq!(ratio(1 << 60, 1), Some((1_u64 << 60) as f64));
        // 2^53 + 1 is a tie between 2^53 and 2^53 + 2: ties go to even.
        assert_eq!(ratio((1 << 53) + 1, 1), Some((1_u64 << 53) as f64));
        assert_eq!(ratio((1 << 53) + 3, 1), Some(((1_u64 << 53) + 4) as f64));
    }

    #[test]
    fn keeps_gradual_underflow_and_denies_overflow() {
        let tiny = round_ratio(&Natural::from_u128(1), &Natural::from_u128(1).shl(1074));
        assert_eq!(tiny, Some(f64::from_bits(1)));
        let below_half = round_ratio(&Natural::from_u128(1), &Natural::from_u128(1).shl(1076));
        assert_eq!(below_half, Some(0.0));
        assert_eq!(
            round_ratio(&Natural::from_u128(1).shl(1024), &Natural::from_u128(1)),
            None
        );
    }

    #[test]
    fn scales_decimal_literals_with_one_rounding() {
        let millimeter = catalog_unit("mm").unwrap().scale;
        let decimal = DecimalText::parse("1.5").unwrap();
        assert_eq!(scaled_decimal(&decimal, false, millimeter), Some(0.0015));
        assert_eq!(scaled_decimal(&decimal, true, millimeter), Some(-0.0015));
        let inch = catalog_unit("in").unwrap().scale;
        let one = DecimalText::parse("1").unwrap();
        assert_eq!(scaled_decimal(&one, false, inch), Some(0.0254));
        let degree = catalog_unit("deg").unwrap().scale;
        assert_eq!(scaled_decimal(&one, false, degree), Some(DEGREE_IN_RADIANS));
        let ninety = DecimalText::parse("90").unwrap();
        // The exact product of 90 and the binary64 degree constant, rounded once.
        assert_eq!(
            scaled_decimal(&ninety, false, degree),
            Some(90.0 * DEGREE_IN_RADIANS)
        );
    }
}
