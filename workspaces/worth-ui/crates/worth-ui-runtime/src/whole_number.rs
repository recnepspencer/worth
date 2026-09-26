//! Whole float values read as the integers they name.
//!
//! Rust has no checked conversion from a float to an integer: `as` rounds
//! toward zero, saturates at the target's bounds, and reads NaN as zero, so
//! a fraction, an overflow, or a NaN would each pass as some other integer.
//! Every such read crosses here instead, and anything that is not a whole
//! value inside the target's range is refused. Callers round first, the way
//! their own meaning asks.

/// `value` as a `u16` when it is a whole number `u16` counts.
pub(crate) fn whole_u16(value: f64) -> Option<u16> {
    whole_u32(value).and_then(|value| u16::try_from(value).ok())
}

/// `value` as a `u32` when it is a whole number `u32` counts.
pub(crate) fn whole_u32(value: f64) -> Option<u32> {
    if !(value.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(&value)) {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a whole value within u32's range converts exactly"
    )]
    let whole = value as u32;
    Some(whole)
}

/// `value` as an `i64` when it is a whole number `i64` counts.
pub(crate) fn whole_i64(value: f64) -> Option<i64> {
    // 2^63 is one past the largest `i64`, and the smallest `f64` above
    // `i64::MAX`; -2^63 is `i64::MIN` itself.
    const BOUND: f64 = 9_223_372_036_854_775_808.0;
    if !(value.fract() == 0.0 && (-BOUND..BOUND).contains(&value)) {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a whole value within i64's range converts exactly"
    )]
    let whole = value as i64;
    Some(whole)
}

#[cfg(test)]
#[path = "whole_number_tests.rs"]
mod tests;
