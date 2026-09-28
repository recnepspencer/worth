//! Runtime unit conversion for `quantity(value, unit)` and
//! `magnitude(quantity, unit)` over nonliteral values.
//!
//! Both scale the exact binary value by the unit's exact rational scale and
//! round once to binary64, the same contract literal folding uses.

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};
use crate::expressions::numeric::{scaled_ratio, Natural};
use crate::expressions::types::units::UnitScale;

use super::conversion::odd_decompose;

/// The magnitude of a canonical SI `quantity` expressed in a unit with `scale`.
pub(crate) fn in_unit(quantity: f64, scale: UnitScale) -> ExpressionResult<f64> {
    let inverse = UnitScale {
        numerator: scale.denominator,
        denominator: scale.numerator,
        degree_power: -scale.degree_power,
    };
    to_canonical(quantity, inverse)
}

/// The canonical SI magnitude of `value` authored in a unit with `scale`.
pub(crate) fn to_canonical(value: f64, scale: UnitScale) -> ExpressionResult<f64> {
    if value == 0.0 {
        return Ok(0.0);
    }
    let (mantissa, exponent) = odd_decompose(value.abs());
    let one = Natural::from_u128(1);
    let mantissa = Natural::from_u128(u128::from(mantissa));
    let (numerator, denominator) = if exponent >= 0 {
        (mantissa.shl(exponent as u64), one)
    } else {
        (mantissa, one.shl(u64::from(exponent.unsigned_abs())))
    };
    let magnitude = scaled_ratio(numerator, denominator, scale).ok_or_else(|| {
        ExpressionDenial::new(ExpressionDenialDetail::ArithmeticOverflow(
            "the scaled quantity is not finite",
        ))
    })?;
    Ok(if value < 0.0 {
        -magnitude + 0.0
    } else {
        magnitude
    })
}

#[cfg(test)]
mod tests {
    use super::{in_unit, to_canonical};
    use crate::expressions::types::units::{catalog_unit, DEGREE_IN_RADIANS};

    #[test]
    fn scales_once_through_exact_unit_ratios() {
        let millimeter = catalog_unit("mm").unwrap().scale;
        assert_eq!(to_canonical(1.5, millimeter), Ok(0.0015));
        assert_eq!(to_canonical(-900.0, millimeter), Ok(-0.9));
        assert_eq!(in_unit(0.9, millimeter), Ok(900.0));
        let inch = catalog_unit("in").unwrap().scale;
        assert_eq!(to_canonical(1.0, inch), Ok(0.0254));
        assert_eq!(in_unit(0.0254, inch), Ok(1.0));
        let degree = catalog_unit("deg").unwrap().scale;
        assert_eq!(to_canonical(1.0, degree), Ok(DEGREE_IN_RADIANS));
        assert_eq!(in_unit(DEGREE_IN_RADIANS, degree), Ok(1.0));
        assert_eq!(to_canonical(0.0, degree), Ok(0.0));
        let kilometer = catalog_unit("km").unwrap().scale;
        assert!(to_canonical(f64::MAX, kilometer).is_err());
    }
}
