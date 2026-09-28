use super::{bits_to_integer, exact_cast, rounded_cast, Number, Target};
use crate::expressions::denial::ExpressionDenialFamily as Family;
use crate::expressions::types::IntegerType;

fn exact(value: Number, target: Target) -> Result<Number, Family> {
    exact_cast(value, target).map_err(|denial| denial.family())
}

fn rounded(value: Number, target: Target) -> Result<Number, Family> {
    rounded_cast(value, target).map_err(|denial| denial.family())
}

const INT8: Target = Target::Integer(IntegerType::Int8);
const INT64: Target = Target::Integer(IntegerType::Int64);

#[test]
fn exact_casts_into_integers_check_integrality_then_range() {
    assert_eq!(exact(Number::Float64(-3.0), INT8), Ok(Number::Integer(-3)));
    assert_eq!(
        exact(Number::Float64(2.5), INT64),
        Err(Family::ArithmeticDomain)
    );
    assert_eq!(
        exact(Number::Float64(1e30), INT64),
        Err(Family::ArithmeticOverflow)
    );
    assert_eq!(
        exact(Number::Float32(128.0), INT8),
        Err(Family::ArithmeticOverflow)
    );
    assert_eq!(exact(Number::Decimal((7, 0)), INT8), Ok(Number::Integer(7)));
    assert_eq!(
        exact(Number::Decimal((25, 1)), INT8),
        Err(Family::ArithmeticDomain)
    );
    assert_eq!(
        exact(Number::Decimal((300, 0)), INT8),
        Err(Family::ArithmeticOverflow)
    );
    assert_eq!(
        exact(Number::Integer(-1), Target::Integer(IntegerType::UInt8)),
        Err(Family::ArithmeticOverflow)
    );
}

#[test]
fn exact_casts_into_floats_deny_lost_precision() {
    let big = 1_i128 << 53;
    assert_eq!(
        exact(Number::Integer(big), Target::Float64),
        Ok(Number::Float64(big as f64))
    );
    assert_eq!(
        exact(Number::Integer(big + 1), Target::Float64),
        Err(Family::ArithmeticDomain)
    );
    assert_eq!(
        exact(Number::Integer(1 << 24), Target::Float32),
        Ok(Number::Float32(16_777_216.0))
    );
    assert_eq!(
        exact(Number::Integer((1 << 24) + 1), Target::Float32),
        Err(Family::ArithmeticDomain)
    );
    assert_eq!(
        exact(Number::Float64(0.5), Target::Float32),
        Ok(Number::Float32(0.5))
    );
    assert_eq!(
        exact(Number::Float64(0.1), Target::Float32),
        Err(Family::ArithmeticDomain)
    );
    assert_eq!(
        exact(Number::Float64(1e300), Target::Float32),
        Err(Family::ArithmeticOverflow)
    );
    assert_eq!(
        exact(Number::Float32(0.1), Target::Float64),
        Ok(Number::Float64(f64::from(0.1_f32)))
    );
    assert_eq!(
        exact(Number::Decimal((25, 1)), Target::Float64),
        Ok(Number::Float64(2.5))
    );
    assert_eq!(
        exact(Number::Decimal((-5, 1)), Target::Float32),
        Ok(Number::Float32(-0.5))
    );
    assert_eq!(
        exact(Number::Decimal((1, 1)), Target::Float64),
        Err(Family::ArithmeticDomain)
    );
    assert_eq!(
        exact(Number::Decimal((1, 1)), Target::Float32),
        Err(Family::ArithmeticDomain)
    );
    let beyond_float32 = (1_i128 << 24) + 1;
    assert_eq!(
        exact(Number::Decimal((beyond_float32, 0)), Target::Float32),
        Err(Family::ArithmeticDomain)
    );
}

#[test]
fn exact_casts_into_decimals_keep_every_digit() {
    assert_eq!(
        exact(Number::Float64(0.375), Target::Decimal),
        Ok(Number::Decimal((375, 3)))
    );
    assert_eq!(
        exact(Number::Float64(-0.0), Target::Decimal),
        Ok(Number::Decimal((0, 0)))
    );
    assert_eq!(
        exact(Number::Float32(1024.0), Target::Decimal),
        Ok(Number::Decimal((1024, 0)))
    );
    // 0.1 is m * 2^-56 in binary64, which needs 56 decimal places.
    assert_eq!(
        exact(Number::Float64(0.1), Target::Decimal),
        Err(Family::ArithmeticDomain)
    );
    assert_eq!(
        exact(Number::Float64(f64::from_bits(1)), Target::Decimal),
        Err(Family::ArithmeticDomain)
    );
    // The binary64 nearest 1e38 is just below 10^38, so it still fits.
    assert_eq!(
        exact(Number::Float64(1e38), Target::Decimal),
        Ok(Number::Decimal((
            99_999_999_999_999_997_748_809_823_456_034_029_568,
            0
        )))
    );
    assert_eq!(
        exact(Number::Float64(1e39), Target::Decimal),
        Err(Family::ArithmeticOverflow)
    );
    assert_eq!(
        exact(Number::Integer(-42), Target::Decimal),
        Ok(Number::Decimal((-42, 0)))
    );
    assert_eq!(
        exact(Number::Integer(1200), Target::Decimal),
        Ok(Number::Decimal((1200, 0)))
    );
}

#[test]
fn rounded_casts_round_once_nearest_even() {
    assert_eq!(
        rounded(Number::Decimal((1, 1)), Target::Float64),
        Ok(Number::Float64(0.1))
    );
    assert_eq!(
        rounded(Number::Decimal((1, 1)), Target::Float32),
        Ok(Number::Float32(0.1))
    );
    assert_eq!(
        rounded(Number::Decimal((-1, 18)), Target::Float32),
        Ok(Number::Float32(-1e-18))
    );
    // 2^60 + 2^36 + 1 is just above a Float32 midpoint; rounding through
    // binary64 would land on the midpoint and tie down to 2^60.
    let value = (1_i128 << 60) + (1 << 36) + 1;
    let up = ((1_u64 << 60) + (1 << 37)) as f32;
    assert_eq!(
        rounded(Number::Decimal((value, 0)), Target::Float32),
        Ok(Number::Float32(up))
    );
    assert_eq!(
        rounded(Number::Integer(value), Target::Float32),
        Ok(Number::Float32(up))
    );
    assert_eq!(
        rounded(Number::Integer((1 << 53) + 1), Target::Float64),
        Ok(Number::Float64((1_u64 << 53) as f64))
    );
    assert_eq!(
        rounded(Number::Float64(0.1), Target::Float32),
        Ok(Number::Float32(0.1))
    );
    assert_eq!(
        rounded(Number::Float64(1e300), Target::Float32),
        Err(Family::ArithmeticOverflow)
    );
    assert_eq!(
        rounded(Number::Float64(-1e-300), Target::Float32),
        Ok(Number::Float32(0.0))
    );
}

#[test]
fn buses_convert_to_unsigned_integers_by_range() {
    assert_eq!(bits_to_integer(&[200], IntegerType::UInt8), Ok(200));
    assert!(bits_to_integer(&[300], IntegerType::UInt8).is_err());
    assert_eq!(
        bits_to_integer(&[u64::MAX, 0], IntegerType::UInt64),
        Ok(i128::from(u64::MAX))
    );
    assert!(bits_to_integer(&[0, 1], IntegerType::UInt64).is_err());
}
